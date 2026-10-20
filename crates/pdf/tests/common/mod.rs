//! Shared helpers for the PDF integration tests.

#![allow(dead_code)]

use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use pdfium_render::prelude::Pdfium;
use testkit::vendor_dir;

/// PDFium for one test; holds the process-wide lock so no other test uses PDFium meanwhile.
pub struct Bound {
    pdfium: Pdfium,
    _lock: testkit::PdfiumGuard,
}

impl Deref for Bound {
    type Target = Pdfium;
    fn deref(&self) -> &Pdfium {
        &self.pdfium
    }
}

/// Bind PDFium, or `None` when the DLL is missing (the tests then skip).
pub fn bind() -> Option<Bound> {
    let dir = vendor_dir()?;
    let lock = testkit::pdfium_lock();
    Some(Bound {
        pdfium: packing_pdf::bind(&dir).expect("bind pdfium"),
        _lock: lock,
    })
}

/// Whether the packed Arial/Consola fonts are installed.
pub fn fonts_available() -> bool {
    Path::new(r"C:\Windows\Fonts\arial.ttf").exists()
}

/// A unique path under the system temp directory, for written test output.
pub fn temp_path(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!(
        "packing-pdf-{}-{tag}-{nanos}.pdf",
        std::process::id()
    ))
}
