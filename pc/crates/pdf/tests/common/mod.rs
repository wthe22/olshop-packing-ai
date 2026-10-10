//! Shared helpers for the PDF integration tests.

#![allow(dead_code)]

use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

use pdfium_render::prelude::Pdfium;

/// The pinned `pdfium.dll` directory, or `None` when it has not been downloaded.
pub fn vendor_dir() -> Option<PathBuf> {
    let vendor = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor");
    vendor.join("pdfium.dll").exists().then_some(vendor)
}

/// PDFium is not thread-safe and the test harness runs tests on several threads.
static PDFIUM_LOCK: Mutex<()> = Mutex::new(());

/// PDFium for one test; holds the lock so no other test uses PDFium meanwhile.
pub struct Bound {
    pdfium: Pdfium,
    _lock: MutexGuard<'static, ()>,
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
    let lock = PDFIUM_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    Some(Bound {
        pdfium: packing_pdf::bind(&dir).expect("bind pdfium"),
        _lock: lock,
    })
}

/// A file in the shared `testdata/` folder.
pub fn testdata(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../testdata")
        .join(name)
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
