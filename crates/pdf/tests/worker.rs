//! The PDFium worker thread: a task runs on it with the bound library, one after another.

use packing_pdf::read;
use packing_pdf::worker::PdfWorker;
use testkit::{testdata, vendor_dir};

#[test]
fn worker_runs_tasks_with_the_bound_library() {
    let Some(vendor) = vendor_dir() else {
        eprintln!("skipped: run tools/get-pdfium.sh");
        return;
    };
    let _guard = testkit::pdfium_lock();
    let worker = PdfWorker::spawn(vendor).expect("spawn worker");

    let path = testdata("labels-slip.pdf");
    let pages = worker
        .run(move |pdfium| read::open(pdfium, &path).map(|file| file.pages))
        .expect("the task ran")
        .expect("open the labels");
    assert_eq!(pages, 24);

    let missing = worker
        .run(|pdfium| read::open(pdfium, std::path::Path::new("no-such-file.pdf")).map(|_| ()))
        .expect("the task ran");
    assert!(missing.is_err(), "a task's own error comes back");
}
