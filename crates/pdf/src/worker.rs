//! The one thread that owns PDFium (07 › *Commands between window and Rust*).
//!
//! PDFium is not thread-safe, so no PDFium object may leave this thread. The worker binds the
//! library once, keeps the opened label documents of the current draft until it is replaced or
//! the worker stops, and runs the small set of [`Job`]s it is sent; results go back over a
//! channel carried in the job.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

use pdfium_render::prelude::Pdfium;

use packing_engine::orders::PageItem;
use packing_engine::text::PageText;

use crate::PdfError;
use crate::packing_list::{self, Layout, PackingList};
use crate::read::{self, OpenedFile};
use crate::write;

/// How far a running read has got: pages done of all files' pages.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Progress {
    pub page: usize,
    pub pages: usize,
}

/// A unit of work for the worker thread.
pub enum Job {
    /// Open label files (replacing the current draft) and report each file's page count.
    Open {
        paths: Vec<PathBuf>,
        reply: Sender<Result<Vec<usize>, PdfError>>,
    },
    /// Read every page of the current draft, reporting progress and honouring a cancel flag.
    Read {
        cancel: Arc<AtomicBool>,
        progress: Sender<Progress>,
        reply: Sender<Result<Vec<PageItem>, PdfError>>,
    },
    /// Copy pages from the current draft into a new saved PDF.
    WritePages {
        pages: Vec<(usize, usize)>,
        out: PathBuf,
        reply: Sender<Result<(), PdfError>>,
    },
    /// Write a packing list.
    WritePackingList {
        layout: Layout,
        out: PathBuf,
        list: Box<PackingList>,
        reply: Sender<Result<usize, PdfError>>,
    },
    /// Stop the worker loop.
    Shutdown,
}

/// A handle to the PDFium worker thread.
pub struct PdfWorker {
    tx: Sender<Job>,
    join: Option<JoinHandle<()>>,
}

impl PdfWorker {
    /// Spawn the worker, binding `pdfium.dll` from `dll_dir` on the new thread. Fails when the
    /// library cannot be bound.
    pub fn spawn(dll_dir: PathBuf) -> Result<PdfWorker, PdfError> {
        let (tx, rx) = mpsc::channel();
        let (ready_tx, ready_rx) = mpsc::channel();
        let join = thread::spawn(move || match crate::bind(&dll_dir) {
            Ok(pdfium) => {
                let _ = ready_tx.send(Ok(()));
                serve(&pdfium, &rx);
            }
            Err(error) => {
                let _ = ready_tx.send(Err(error));
            }
        });
        ready_rx
            .recv()
            .map_err(|_| PdfError::Message("the PDF worker thread stopped at start".into()))??;
        Ok(PdfWorker {
            tx,
            join: Some(join),
        })
    }

    /// Open label files, replacing the current draft. Returns their page counts.
    pub fn open(&self, paths: &[PathBuf]) -> Result<Vec<usize>, PdfError> {
        let (reply, rx) = mpsc::channel();
        self.send(Job::Open {
            paths: paths.to_vec(),
            reply,
        })?;
        recv(&rx)
    }

    /// Read every page of the current draft. Progress is sent on `progress`; `cancel` stops it.
    pub fn read(
        &self,
        cancel: Arc<AtomicBool>,
        progress: Sender<Progress>,
    ) -> Result<Vec<PageItem>, PdfError> {
        let (reply, rx) = mpsc::channel();
        self.send(Job::Read {
            cancel,
            progress,
            reply,
        })?;
        recv(&rx)
    }

    /// Copy the given `(file index, page index)` pages of the current draft into `out`.
    pub fn write_pages(&self, pages: &[(usize, usize)], out: &Path) -> Result<(), PdfError> {
        let (reply, rx) = mpsc::channel();
        self.send(Job::WritePages {
            pages: pages.to_vec(),
            out: out.to_path_buf(),
            reply,
        })?;
        recv(&rx)
    }

    /// Write a packing list; returns the page count.
    pub fn write_packing_list(
        &self,
        layout: Layout,
        out: &Path,
        list: &PackingList,
    ) -> Result<usize, PdfError> {
        let (reply, rx) = mpsc::channel();
        self.send(Job::WritePackingList {
            layout,
            out: out.to_path_buf(),
            list: Box::new(list.clone()),
            reply,
        })?;
        recv(&rx)
    }

    fn send(&self, job: Job) -> Result<(), PdfError> {
        self.tx
            .send(job)
            .map_err(|_| PdfError::Message("the PDF worker thread has stopped".into()))
    }
}

impl Drop for PdfWorker {
    fn drop(&mut self) {
        let _ = self.tx.send(Job::Shutdown);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

fn recv<T>(rx: &Receiver<Result<T, PdfError>>) -> Result<T, PdfError> {
    rx.recv()
        .map_err(|_| PdfError::Message("the PDF worker thread has stopped".into()))?
}

/// The worker loop: bind once, keep the draft, run jobs until shutdown.
fn serve(pdfium: &Pdfium, rx: &Receiver<Job>) {
    let mut draft: Option<Vec<OpenedFile<'_>>> = None;
    while let Ok(job) = rx.recv() {
        match job {
            Job::Open { paths, reply } => {
                let result = read::open_all(pdfium, &paths).map(|files| {
                    let counts = files.iter().map(|file| file.pages).collect();
                    draft = Some(files);
                    counts
                });
                let _ = reply.send(result);
            }
            Job::Read {
                cancel,
                progress,
                reply,
            } => {
                let result = match &draft {
                    Some(files) => {
                        let mut pages: Vec<PageItem> = Vec::new();
                        let mut on_page =
                            |file: usize, page: usize, text: &PageText| -> Result<(), PdfError> {
                                pages.push((file, page, text.clone()));
                                Ok(())
                            };
                        read::for_each_page(
                            files,
                            &cancel,
                            &mut |done, total| {
                                let _ = progress.send(Progress {
                                    page: done,
                                    pages: total,
                                });
                            },
                            &mut on_page,
                        )
                        .map(|()| pages)
                    }
                    None => Err(PdfError::Message("no labels are open".into())),
                };
                let _ = reply.send(result);
            }
            Job::WritePages { pages, out, reply } => {
                let result = match &draft {
                    Some(files) => write::copy_pages(pdfium, files, &pages, &out),
                    None => Err(PdfError::Message("no labels are open".into())),
                };
                let _ = reply.send(result);
            }
            Job::WritePackingList {
                layout,
                out,
                list,
                reply,
            } => {
                let result = packing_list::write(pdfium, &out, layout, &list);
                let _ = reply.send(result);
            }
            Job::Shutdown => break,
        }
    }
}
