//! The one thread that owns PDFium.
//!
//! PDFium is not thread-safe, so no PDFium object may leave this thread. The worker binds the
//! library once and runs the closures it is sent with the bound library; each closure opens what
//! it needs and closes it before it returns.

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

use pdfium_render::prelude::Pdfium;

use crate::PdfError;

/// How far a running read has got: pages done of all files' pages.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Progress {
    pub page: usize,
    pub pages: usize,
}

/// A unit of work for the worker thread.
enum Job {
    Run(Box<dyn FnOnce(&Pdfium) + Send>),
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

    /// Run `task` on the worker thread and return what it returns.
    pub fn run<R, F>(&self, task: F) -> Result<R, PdfError>
    where
        R: Send + 'static,
        F: FnOnce(&Pdfium) -> R + Send + 'static,
    {
        let (reply, rx) = mpsc::channel();
        self.tx
            .send(Job::Run(Box::new(move |pdfium| {
                let _ = reply.send(task(pdfium));
            })))
            .map_err(|_| stopped())?;
        rx.recv().map_err(|_| stopped())
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

fn stopped() -> PdfError {
    PdfError::Message("the PDF worker thread has stopped".into())
}

/// The worker loop: run jobs until shutdown.
fn serve(pdfium: &Pdfium, rx: &Receiver<Job>) {
    while let Ok(job) = rx.recv() {
        match job {
            Job::Run(task) => task(pdfium),
            Job::Shutdown => break,
        }
    }
}
