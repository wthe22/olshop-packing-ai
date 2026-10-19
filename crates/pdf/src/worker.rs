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

use packing_engine::label::CourierTable;
use packing_engine::orders::PageItem;
use packing_engine::text::PageText;
use serde::Serialize;

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

/// Which part of a save is running (07 › *Check before saving*). The names are the window's
/// `save-progress` stage names.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum SaveStage {
    /// The saved PDFs and packing lists are being written.
    Writing,
    /// The written files are being read back and compared with the plan.
    Checking,
}

/// How far a running save has got: the stage and pages done of all pages.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SaveProgress {
    pub stage: SaveStage,
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
        /// The data folder's courier data (the band images' appearance decision; 07 › *Inputs*).
        couriers: CourierTable,
        reply: Sender<Result<Vec<PageItem>, PdfError>>,
    },
    /// Read the given files' pages back **without** replacing the current draft (the check
    /// before saving reads the written files; the label documents stay open, so a retry after
    /// *Save anyway* still has them).
    ReadBack {
        paths: Vec<PathBuf>,
        progress: Sender<SaveProgress>,
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
    /// The header-band image with byte hash `hash` on one page of one open file, as PNG bytes
    /// (the logo prompt shows the logo as printed).
    LogoPicture {
        file: usize,
        page: usize,
        hash: String,
        reply: Sender<Result<Option<Vec<u8>>, PdfError>>,
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
    /// `couriers` is the data folder's courier data.
    pub fn read(
        &self,
        cancel: Arc<AtomicBool>,
        progress: Sender<Progress>,
        couriers: CourierTable,
    ) -> Result<Vec<PageItem>, PdfError> {
        let (reply, rx) = mpsc::channel();
        self.send(Job::Read {
            cancel,
            progress,
            couriers,
            reply,
        })?;
        recv(&rx)
    }

    /// Read the given files' pages back without replacing the current draft (the check before
    /// saving reads the written files, so the label documents stay open).
    pub fn read_back(
        &self,
        paths: &[PathBuf],
        progress: Sender<SaveProgress>,
    ) -> Result<Vec<PageItem>, PdfError> {
        let (reply, rx) = mpsc::channel();
        self.send(Job::ReadBack {
            paths: paths.to_vec(),
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

    /// One page's header-band image with byte hash `hash`, as PNG bytes (the logo prompt).
    pub fn logo_picture(
        &self,
        file: usize,
        page: usize,
        hash: &str,
    ) -> Result<Option<Vec<u8>>, PdfError> {
        let (reply, rx) = mpsc::channel();
        self.send(Job::LogoPicture {
            file,
            page,
            hash: hash.to_string(),
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
                couriers,
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
                            &couriers,
                            &mut on_page,
                        )
                        .map(|()| pages)
                    }
                    None => Err(PdfError::Message("no labels are open".into())),
                };
                let _ = reply.send(result);
            }
            Job::ReadBack {
                paths,
                progress,
                reply,
            } => {
                let result = read::open_all(pdfium, &paths).and_then(|files| {
                    let total: usize = files.iter().map(|file| file.pages).sum();
                    let mut pages: Vec<PageItem> = Vec::with_capacity(total);
                    let mut done = 0usize;
                    for (file_index, file) in files.iter().enumerate() {
                        for page_index in 0..file.pages {
                            pages.push((file_index, page_index, file.page_text_plain(page_index)?));
                            done += 1;
                            let _ = progress.send(SaveProgress {
                                stage: SaveStage::Checking,
                                page: done,
                                pages: total,
                            });
                        }
                    }
                    Ok(pages)
                });
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
            Job::LogoPicture {
                file,
                page,
                hash,
                reply,
            } => {
                let result = match draft.as_ref().and_then(|files| files.get(file)) {
                    Some(source) => source.band_picture(page, &hash),
                    None => Err(PdfError::Message(format!(
                        "the logo prompt asks for file {file}, but only {} are open",
                        draft.as_ref().map_or(0, Vec::len)
                    ))),
                };
                let _ = reply.send(result);
            }
            Job::Shutdown => break,
        }
    }
}
