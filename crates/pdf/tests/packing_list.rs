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

/// One text run read back from a written page: `(x, y, text)`, points from PDFium's origin.
type Run = (f64, f64, String);

/// Write `layout`, read it back, and return (page count written, compact text of all pages,
/// the runs of every page).
fn write_and_read(
    pdfium: &pdfium_render::prelude::Pdfium,
    layout: Layout,
    list: &PackingList,
    tag: &str,
) -> (usize, String, Vec<Vec<Run>>) {
    let out = common::temp_path(tag);
    let written = write(pdfium, &out, layout, list).expect("write packing list");
    let cancel = std::sync::atomic::AtomicBool::new(false);
    let pages = read::read_all(
        pdfium,
        std::slice::from_ref(&out),
        &cancel,
        packing_engine::label::courier_data(),
        &mut |_, _| {},
    )
    .expect("read back");
    let text: String = pages
        .iter()
        .map(|(_, _, page)| page.text.clone())
        .collect::<Vec<_>>()
        .join("\n");
    let runs: Vec<Vec<Run>> = pages
        .iter()
        .map(|(_, _, page)| {
            page.runs
                .iter()
                .map(|run| (run.x, run.y, run.text.clone()))
                .collect()
        })
        .collect();
    let size = std::fs::metadata(&out).expect("file exists").len();
    eprintln!("packing list {layout} -> {written} pages, {size} bytes");
    let _ = std::fs::remove_file(&out);
    assert_eq!(written, pages.len(), "reported page count matches the file");
    (written, compact(&text), runs)
}

#[test]
fn writes_every_layout_with_the_right_text() {
    let Some(pdfium) = common::bind() else {
        eprintln!("skipped: run tools/get-pdfium.sh");
        return;
    };
    if !common::fonts_available() {
        eprintln!("skipped: Arial font not installed");
        return;
    }
    let list = sample_list();

    let (full_pages, full, _) = write_and_read(&pdfium, Layout::Full, &list, "full");
    let (_summary_pages, summary, _) = write_and_read(&pdfium, Layout::Summary, &list, "summary");
    let (pick_pages, pick, _) = write_and_read(&pdfium, Layout::Pick, &list, "pick");

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
    // Review round 1: the multiplication sign must be a real `×` (U+00D7), drawn and in the text
    // layer. It is drawn by a CID copy of Arial, because PDFium's non-CID font path writes `ÿ`.
    assert!(
        full.contains("×1"),
        "the item line must print ×: {full:.120}"
    );
    assert!(summary.contains("×1"), "the item line must print ×");
    assert!(!full.contains('ÿ'), "no ÿ may reach the file: {full:.120}");
}

#[test]
fn whole_scope_names_the_number_range() {
    let Some(pdfium) = common::bind() else {
        eprintln!("skipped: run tools/get-pdfium.sh");
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
    let (_pages, text, _) = write_and_read(&pdfium, Layout::Summary, &list, "whole");
    assert!(
        text.contains("savedPDFs1-2"),
        "range header missing: {text:.80}"
    );
}

#[test]
fn a_list_needs_item_data() {
    let Some(pdfium) = common::bind() else {
        eprintln!("skipped: run tools/get-pdfium.sh");
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

/// A list long enough to break a page in all three layouts: 110 runs of one order each.
fn long_list() -> PackingList {
    let runs: Vec<RunRows> = (1..=110u32)
        .map(|number| RunRows {
            number,
            orders: vec![order(
                &format!("JY{number:010}"),
                vec![line(
                    &format!("Produk Uji {number:03}"),
                    "honda",
                    1,
                    &format!("Produk Uji {number:03} — honda"),
                )],
            )],
        })
        .collect();
    PackingList {
        day: "2026-10-11".to_string(),
        batch: Some(1),
        printed: "13:46".to_string(),
        sections: vec![SavedPdfSection {
            number: 1,
            code: "A".to_string(),
            name: "Sepatu".to_string(),
            runs,
        }],
    }
}

/// Review round 1: PDFium's non-CID font path writes `×` and other characters as `ÿ` (a probe
/// turned `× ÷ é ® ™ ° ± ñ Ä ½ €` into `ÿ`). They are drawn by a CID copy of the same font instead
/// — and the text layer of the plain text around them must stay plain, because that font encodes
/// spaces as U+00A0, `-` as U+00AD and `·` as U+A78F.
#[test]
fn characters_pdfium_encodes_wrongly_are_drawn_and_still_read_back() {
    let Some(pdfium) = common::bind() else {
        eprintln!("skipped: run tools/get-pdfium.sh");
        return;
    };
    if !common::fonts_available() {
        eprintln!("skipped: Arial font not installed");
        return;
    }
    let mut list = sample_list();
    list.sections[0].runs[1].orders[0].lines[0] = line(
        "Kabel Ñ ² ® ñ ½° ±",
        "honda",
        2,
        "Kabel Ñ ² ® ñ ½° ± — honda",
    );
    let (pages, text, runs) = write_and_read(&pdfium, Layout::Full, &list, "cid");

    assert_eq!(pages, 1);
    assert!(
        text.contains("KabelÑ²®ñ½°±—honda"),
        "the name must print: {text:.200}"
    );
    assert!(text.contains("×2"), "the count must print: {text:.200}");
    assert!(!text.contains('ÿ'), "no ÿ may reach the file: {text:.200}");
    // Which characters may use the CID font at all is pinned by the unit test in the module: a
    // space must not, because that font maps it to U+00A0. Here the header's own separators are the
    // check that the plain text is still written as written.
    let header = runs[0]
        .iter()
        .map(|(_, _, text)| text.as_str())
        .find(|text| text.starts_with("Packing list"))
        .expect("the header must be the first text on the page");
    assert!(
        header.contains(" · "),
        "the header must keep its plain separators: {header:?}"
    );
}

/// Review round 1: the body started at the top margin, so the first row or bar of every page was
/// printed over the header. The header's three lines (title, page number, counts) are the only
/// text in the top 22 mm of a page; the body starts below them on the first page and after every
/// page break, in all three layouts.
#[test]
fn the_body_is_never_printed_over_the_header() {
    let Some(pdfium) = common::bind() else {
        eprintln!("skipped: run tools/get-pdfium.sh");
        return;
    };
    if !common::fonts_available() {
        eprintln!("skipped: Arial font not installed");
        return;
    }
    /// The header's title, page number and counts line.
    const HEADER_LINES: usize = 3;
    /// The counts line's baseline, 22 mm from the page top (`BODY_TOP` is 23 mm).
    const HEADER_BOTTOM_MM: f64 = 22.0;
    const PT_PER_MM: f64 = 72.0 / 25.4;
    const PAGE_H_MM: f64 = 297.0;

    let list = long_list();
    for layout in [Layout::Full, Layout::Summary, Layout::Pick] {
        let (pages, _text, runs) = write_and_read(&pdfium, layout, &list, "overlap");
        assert!(pages > 1, "{layout}: the sample must break a page");
        for (index, page) in runs.iter().enumerate() {
            let page_no = index + 1;
            assert!(
                page.len() > HEADER_LINES,
                "{layout} page {page_no}: no body text"
            );
            // PDFium reports a run's y from the top or from the bottom; the title sits above the
            // counts line either way, so the header itself says which.
            let (title, number, counts) = (&page[0], &page[1], &page[2]);
            assert!(
                title.2.starts_with("Packing list"),
                "{layout} page {page_no}: first run is {:?}",
                title.2
            );
            assert!(
                number.2.starts_with("page "),
                "{layout} page {page_no}: second run is {:?}",
                number.2
            );
            assert!(
                counts.2.contains("orders"),
                "{layout} page {page_no}: third run is {:?}",
                counts.2
            );
            let from_top = title.1 < counts.1;
            let from_page_top = |y: f64| {
                if from_top {
                    y / PT_PER_MM
                } else {
                    PAGE_H_MM - y / PT_PER_MM
                }
            };
            for (x, y, text) in page.iter().skip(HEADER_LINES) {
                assert!(
                    from_page_top(*y) >= HEADER_BOTTOM_MM,
                    "{layout} page {page_no}: {text:?} at x={x:.0} is {:.1} mm from the page top, \
                     printed over the header",
                    from_page_top(*y)
                );
            }
        }
    }
}
