//! PDF work for the packing app. This is the only crate that links PDFium; `packing-engine`
//! takes the text runs this crate produces and returns decisions.
//!
//! PDFium is not thread-safe: every PDFium call in the app must run on the one thread that
//! bound the library ([`worker::PdfWorker`]). The free functions in [`read`], [`write`] and
//! [`packing_list`] take a `&Pdfium` and do no thread hand-off themselves.

use std::fmt;
use std::path::{Path, PathBuf};

use pdfium_render::prelude::{Pdfium, PdfiumError};

/// The pinned PDFium release the program binds (`pc/tools/get-pdfium.sh` downloads the same tag);
/// shown in Settings › *About* (08 › *6. Settings*; D14: version + PDFium version only).
pub const PDFIUM_VERSION: &str = "chromium/7881";

pub mod batch;
pub mod packing_list;
pub mod read;
pub mod worker;
pub mod write;

/// Bind the pinned `pdfium.dll` found in `dll_dir`.
pub fn bind(dll_dir: &Path) -> Result<Pdfium, PdfError> {
    match Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(dll_dir)) {
        Ok(bindings) => Ok(Pdfium::new(bindings)),
        // pdfium-render keeps the bindings process-wide and refuses a second bind; `default()`
        // then reuses them (a new worker after the old one stopped, or several tests).
        Err(PdfiumError::PdfiumLibraryBindingsAlreadyInitialized) => Ok(Pdfium::default()),
        Err(e) => Err(PdfError::Bind(e.to_string())),
    }
}

/// A PDF job failed. Every variant carries something worth showing to the user.
#[derive(Debug)]
pub enum PdfError {
    /// `pdfium.dll` could not be loaded.
    Bind(String),
    /// A file could not be opened or read from disk.
    Io { path: PathBuf, message: String },
    /// A file could not be opened as a PDF (damaged, still downloading, wrong file).
    Open { path: PathBuf, message: String },
    /// One of the packed fonts (`C:\Windows\Fonts`) is missing or unreadable.
    Font { path: PathBuf, message: String },
    /// The read was cancelled by the user.
    Cancelled,
    /// A packing list could not be written (bad layout, nothing to list, item data missing).
    Message(String),
    /// An error reported by PDFium itself.
    Pdfium(String),
}

impl fmt::Display for PdfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PdfError::Bind(message) => write!(f, "cannot load pdfium.dll: {message}"),
            PdfError::Io { path, message } => {
                write!(f, "cannot read {}: {message}", path.display())
            }
            PdfError::Open { path, .. } => write!(
                f,
                "{} cannot be read as a PDF. It may be damaged or still downloading. \
                 Download it again.",
                path.display()
            ),
            PdfError::Font { path, message } => {
                write!(f, "font file not found: {} ({message})", path.display())
            }
            PdfError::Cancelled => write!(f, "reading cancelled"),
            PdfError::Message(message) => write!(f, "{message}"),
            PdfError::Pdfium(message) => write!(f, "PDF error: {message}"),
        }
    }
}

impl std::error::Error for PdfError {}

impl From<PdfiumError> for PdfError {
    fn from(error: PdfiumError) -> Self {
        PdfError::Pdfium(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfium_render::prelude::PdfPagePaperSize;

    /// The pinned `pdfium.dll` directory, or `None` when it has not been downloaded.
    pub(crate) fn vendor_dir() -> Option<PathBuf> {
        let vendor = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor");
        vendor.join("pdfium.dll").exists().then_some(vendor)
    }

    #[test]
    fn binds_pdfium_and_adds_a_page() {
        let Some(vendor) = vendor_dir() else {
            eprintln!("skipped: run pc/tools/get-pdfium.sh");
            return;
        };
        let pdfium = bind(&vendor).expect("bind pdfium");
        let mut doc = pdfium.create_new_pdf().expect("create new pdf");
        doc.pages_mut()
            .create_page_at_end(PdfPagePaperSize::a4())
            .expect("add an A4 page");
        assert_eq!(doc.pages().len(), 1);
    }
}
