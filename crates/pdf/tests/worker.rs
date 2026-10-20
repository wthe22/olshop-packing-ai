//! The PDFium worker thread: a read job reports progress and can be cancelled.

mod common;

use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;

use packing_pdf::worker::{PdfWorker, Progress};
use testkit::{testdata, vendor_dir};

#[test]
fn worker_reads_with_progress_and_stops_on_cancel() {
    let Some(vendor) = vendor_dir() else {
        eprintln!("skipped: run tools/get-pdfium.sh");
        return;
    };
    let _guard = testkit::pdfium_lock();
    let worker = PdfWorker::spawn(vendor).expect("spawn worker");

    let counts = worker
        .open(&[testdata("labels-slip.pdf")])
        .expect("open labels");
    assert_eq!(counts, vec![24]);

    // A full read reports progress on the way.
    let (progress_tx, progress_rx) = mpsc::channel();
    let pages = worker
        .read(
            Arc::new(AtomicBool::new(false)),
            progress_tx,
            packing_engine::label::courier_data().clone(),
        )
        .expect("read all pages");
    assert_eq!(pages.len(), 24);
    let progress: Vec<Progress> = progress_rx.try_iter().collect();
    assert_eq!(progress.first().map(|p| (p.page, p.pages)), Some((1, 24)));
    assert_eq!(progress.last().map(|p| (p.page, p.pages)), Some((24, 24)));

    // Cancelling before the first page stops the read.
    let (progress_tx, _progress_rx) = mpsc::channel();
    let error = worker
        .read(
            Arc::new(AtomicBool::new(true)),
            progress_tx,
            packing_engine::label::courier_data().clone(),
        )
        .expect_err("a cancelled read must stop");
    assert!(
        matches!(error, packing_pdf::PdfError::Cancelled),
        "expected Cancelled, got {error}"
    );
}
