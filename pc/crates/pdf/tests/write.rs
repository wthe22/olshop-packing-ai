//! Copying pages (07 › *Testing*: copied pages keep their page count and text).

mod common;

use packing_pdf::{read, write};

#[test]
fn copies_pages_unchanged() {
    let Some(pdfium) = common::bind() else {
        eprintln!("skipped: run pc/tools/get-pdfium.sh");
        return;
    };
    let source = common::testdata("labels-slip.pdf");
    let files = read::open_all(&pdfium, std::slice::from_ref(&source)).expect("open source");
    assert_eq!(files[0].pages, 24);

    // First, second and last page (the last carries the slip table).
    let pages = vec![(0usize, 0usize), (0, 1), (0, 23)];
    let out = common::temp_path("copy");
    write::copy_pages(&pdfium, &files, &pages, &out).expect("copy pages");

    let copied = read::open(&pdfium, &out).expect("open the copy");
    assert_eq!(copied.pages, pages.len());
    for (index, &(file, page)) in pages.iter().enumerate() {
        let original = files[file].page_text(page).expect("original text");
        let copy = copied.page_text(index).expect("copy text");
        assert_eq!(original.text, copy.text, "page {index} text differs");
        assert_eq!(original.runs, copy.runs, "page {index} runs differ");
    }
    let size = std::fs::metadata(&out).expect("copy written").len();
    eprintln!("copied {} pages -> {} bytes", pages.len(), size);
    let _ = std::fs::remove_file(&out);
}

#[test]
fn keeps_any_page_order_and_shares_resources() {
    let Some(pdfium) = common::bind() else {
        eprintln!("skipped: run pc/tools/get-pdfium.sh");
        return;
    };
    let source = common::testdata("labels-slip.pdf");
    let files =
        read::open_all(&pdfium, &[source.clone(), source.clone()]).expect("open sources twice");
    // Every page of both files, interleaved and backwards.
    let pages: Vec<(usize, usize)> = (0..24).rev().flat_map(|p| [(1, p), (0, p)]).collect();
    let out = common::temp_path("copy-order");
    write::copy_pages(&pdfium, &files, &pages, &out).expect("copy pages");

    let copied = read::open(&pdfium, &out).expect("open the copy");
    assert_eq!(copied.pages, pages.len());
    for (index, &(file, page)) in pages.iter().enumerate() {
        let original = files[file].page_text(page).expect("original text");
        let copy = copied.page_text(index).expect("copy text");
        assert_eq!(original.text, copy.text, "page {index} text differs");
    }
    // Page-by-page import made every page carry its own font copy; one import keeps the
    // output near the size of the sources.
    let size = std::fs::metadata(&out).expect("copy written").len();
    let source_size = std::fs::metadata(&source).expect("source").len();
    assert!(
        size < 3 * source_size,
        "{size} bytes from 2 x {source_size}"
    );
    let _ = std::fs::remove_file(&out);
}
