//! The three packing-list layouts from a made-up data model: page counts and written text
//! (read back through `read.rs`), and the stop when an order has no item lines.

mod common;

use packing_pdf::packing_list::{
    Layout, PackingLine, PackingList, RunOrder, RunRows, SavedPdfSection, write,
};
use packing_pdf::read;

fn line(name: &str, variation: &str, quantity: i64, display: &str) -> PackingLine {
    PackingLine {
        name: name.to_string(),
        variation: variation.to_string(),
        quantity,
        display_name: display.to_string(),
    }
}

fn order(tracking: &str, lines: Vec<PackingLine>) -> RunOrder {
    RunOrder {
        tracking_id: tracking.to_string(),
        lines,
    }
}

/// 30 orders of one product plus a run whose display name wraps in the pick summary.
fn sample_list() -> PackingList {
    let sepatu = || {
        line(
            "Sepatu Standar Samping Motor",
            "",
            1,
            "Sepatu Standar Samping Motor",
        )
    };
    let big: Vec<RunOrder> = (0..30)
        .map(|i| order(&format!("JY{:010}", i + 1), vec![sepatu()]))
        .collect();
    let long = line(
        "Spion Scoopy New Gagang Hitam",
        "honda",
        1,
        "Spion Scoopy New Gagang Hitam — Dove, honda, Datar Panjang Sekali Lagi",
    );
    let small: Vec<RunOrder> = (0..2)
        .map(|i| order(&format!("TKP{:09}", i + 1), vec![long.clone()]))
        .collect();
    PackingList {
        day: "2026-10-06".to_string(),
        batch: Some(1),
        printed: "16:40".to_string(),
        sections: vec![SavedPdfSection {
            number: 1,
            code: "A".to_string(),
            name: "Sepatu".to_string(),
            runs: vec![
                RunRows {
                    number: 1,
                    orders: big,
                },
                RunRows {
                    number: 2,
                    orders: small,
                },
            ],
        }],
    }
}

fn compact(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

/// Write `layout`, read it back, and return (page count written, compact text of all pages).
fn write_and_read(
    pdfium: &pdfium_render::prelude::Pdfium,
    layout: Layout,
    list: &PackingList,
    tag: &str,
) -> (usize, String) {
    let out = common::temp_path(tag);
    let written = write(pdfium, &out, layout, list).expect("write packing list");
    let cancel = std::sync::atomic::AtomicBool::new(false);
    let pages = read::read_all(pdfium, std::slice::from_ref(&out), &cancel, &mut |_, _| {})
        .expect("read back");
    let text: String = pages
        .iter()
        .map(|(_, _, page)| page.text.clone())
        .collect::<Vec<_>>()
        .join("\n");
    let size = std::fs::metadata(&out).expect("file exists").len();
    eprintln!("packing list {layout} -> {written} pages, {size} bytes");
    let _ = std::fs::remove_file(&out);
    assert_eq!(written, pages.len(), "reported page count matches the file");
    (written, compact(&text))
}

#[test]
fn writes_every_layout_with_the_right_text() {
    let Some(pdfium) = common::bind() else {
        eprintln!("skipped: run pc/tools/get-pdfium.sh");
        return;
    };
    if !common::fonts_available() {
        eprintln!("skipped: Arial font not installed");
        return;
    }
    let list = sample_list();

    let (full_pages, full) = write_and_read(&pdfium, Layout::Full, &list, "full");
    let (_summary_pages, summary) = write_and_read(&pdfium, Layout::Summary, &list, "summary");
    let (pick_pages, pick) = write_and_read(&pdfium, Layout::Pick, &list, "pick");

    for text in [&full, &summary, &pick] {
        assert!(text.contains("Packinglist"), "title missing: {text:.80}");
        assert!(text.contains("2026-10-06"), "day missing");
        assert!(text.contains("batch1"), "batch number missing (07)");
        assert!(text.contains("1ASepatu"), "section heading missing");
        assert!(
            text.contains("Picksummary(unitstotakefromstock)"),
            "pick summary missing"
        );
        assert!(text.contains("page1/"), "page number missing");
    }
    // The run labels belong to the item sections, which the pick layout does not draw.
    assert!(full.contains("1-01"), "full run label missing");
    assert!(summary.contains("1-01"), "summary run label missing");
    // Only the full layout prints tracking IDs.
    assert!(full.contains("JY0000000001"), "full must show tracking IDs");
    assert!(
        !summary.contains("JY0000000001"),
        "summary must not show tracking IDs"
    );
    assert!(
        !pick.contains("JY0000000001"),
        "pick must not show tracking IDs"
    );
    // Pick is a single sheet; the long display name is wrapped, not cut.
    assert_eq!(pick_pages, 1, "pick layout fits one page here");
    assert!(pick.contains("PanjangSekaliLagi"), "long name must wrap");
    assert!(full_pages >= 1);
}

#[test]
fn whole_scope_names_the_number_range() {
    let Some(pdfium) = common::bind() else {
        eprintln!("skipped: run pc/tools/get-pdfium.sh");
        return;
    };
    if !common::fonts_available() {
        eprintln!("skipped: Arial font not installed");
        return;
    }
    let mut list = sample_list();
    list.sections.push(SavedPdfSection {
        number: 2,
        code: "B".to_string(),
        name: "Spion".to_string(),
        runs: list.sections[0].runs.clone(),
    });
    let (_pages, text) = write_and_read(&pdfium, Layout::Summary, &list, "whole");
    assert!(
        text.contains("savedPDFs1-2"),
        "range header missing: {text:.80}"
    );
}

#[test]
fn a_list_needs_item_data() {
    let Some(pdfium) = common::bind() else {
        eprintln!("skipped: run pc/tools/get-pdfium.sh");
        return;
    };
    if !common::fonts_available() {
        eprintln!("skipped: Arial font not installed");
        return;
    }
    // An order with no slip lines cannot be listed (03 › *Packing list (A4)*).
    let mut list = sample_list();
    list.sections[0].runs[0].orders[0].lines.clear();
    let out = common::temp_path("nodata");
    let error = write(&pdfium, &out, Layout::Full, &list).expect_err("must stop");
    assert!(error.to_string().contains("no item data"), "got: {error}");
    assert!(!out.exists(), "no file must be written");

    // No saved PDFs at all.
    let empty = PackingList {
        day: "2026-10-06".to_string(),
        batch: None,
        printed: "16:40".to_string(),
        sections: Vec::new(),
    };
    let out = common::temp_path("empty");
    assert!(write(&pdfium, &out, Layout::Pick, &empty).is_err());
    assert!(!out.exists());
}
