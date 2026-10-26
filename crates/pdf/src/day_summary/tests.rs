use std::collections::BTreeSet;

use jiff::civil::date;
use packing_engine::record::{BatchSaved, OutputFile};
use testkit::{TempDir, batch, sha, vendor_dir};

use super::content::{header_counts, picks, sections};
use super::{DayBatch, DayFacts, write_day_summary};
use crate::facts::{Row, RowKind};

fn output(file: &str, group: Option<&str>, pages: u16) -> OutputFile {
    OutputFile {
        file: file.to_string(),
        group: group.map(str::to_string),
        pages,
        size: 1,
        sha256: sha('b'),
    }
}

/// Two made-up batches of three orders each; batch 2 amended, one of its orders with no courier.
fn records() -> (BatchSaved, BatchSaved) {
    let groups = |n: u8| {
        vec![
            output(&format!("{n}-A Sepatu x2.pdf"), Some("A"), 2),
            output(&format!("{n}-Z Lainnya x1.pdf"), Some("Z"), 1),
            output(&format!("{n} summary.pdf"), None, 3),
        ]
    };
    let first = batch(1, false, groups(1));
    let mut second = batch(2, true, groups(2));
    second.orders[2].courier = None;
    (first, second)
}

fn facts<'a>(
    first: &'a BatchSaved,
    second: &'a BatchSaved,
    printed: &'a BTreeSet<String>,
) -> DayFacts<'a> {
    DayFacts {
        workspace: "Toko Contoh",
        date: date(2026, 10, 7),
        batches: vec![
            DayBatch {
                record: second,
                time: "09:12".into(),
                printed,
            },
            DayBatch {
                record: first,
                time: "07:40".into(),
                printed,
            },
        ],
        printed: "17:05",
    }
}

#[test]
fn the_day_reads_its_batches_and_couriers_from_the_records() {
    let (first, second) = records();
    let printed: BTreeSet<String> = ["A".to_string()].into();
    let day = facts(&first, &second, &printed);
    assert_eq!(
        header_counts(&day),
        "6 orders · 8 units · 2 batches · 4 groups"
    );

    let head = |text: &str| {
        Row::new(RowKind::Head, text)
            .count(0, "3 orders")
            .count(1, "3 pages")
    };
    let part = |text: &str, orders: &str, pages: &str, printed: &str| {
        Row::new(RowKind::Part, text)
            .count(0, orders)
            .count(1, pages)
            .count(2, printed)
    };
    let shown = sections(&day);
    let headings: Vec<&str> = shown.iter().map(|s| s.heading).collect();
    assert_eq!(headings, ["Batches", "Couriers"]);
    assert_eq!(
        shown[0].rows,
        [
            head("Batch 2 · saved 09:12 · sort rule default · amended"),
            part("2-A Sepatu x2.pdf", "2 orders", "2 pages", "printed"),
            part("2-Z Lainnya x1.pdf", "1 order", "1 page", "not printed"),
            head("Batch 1 · saved 07:40 · sort rule default"),
            part("1-A Sepatu x2.pdf", "2 orders", "2 pages", "printed"),
            part("1-Z Lainnya x1.pdf", "1 order", "1 page", "not printed"),
        ]
    );
    assert_eq!(
        shown[1].rows,
        [
            Row::plain("jnt").count(0, "5 orders"),
            Row::plain("unknown").count(0, "1 order"),
        ]
    );
    assert_eq!(
        picks(&day),
        vec![
            (4, "Sepatu Standar Samping Motor — honda".to_string()),
            (4, "Spion Beat".to_string()),
        ],
        "the units of both batches"
    );
}

#[test]
fn the_facts_then_the_picks_from_a_new_page() {
    let Some(vendor) = vendor_dir() else {
        eprintln!("skipped: run tools/get-pdfium.sh");
        return;
    };
    let _guard = testkit::pdfium_lock();
    let pdfium = crate::bind(&vendor).expect("bind pdfium");
    let dir = TempDir::new("day-summary");
    let path = dir.path().join("day summary.pdf");
    let (first, second) = records();
    let printed = BTreeSet::new();
    let pages = write_day_summary(&pdfium, &path, &facts(&first, &second, &printed))
        .expect("write the day summary");
    assert_eq!(pages, 2);
    let doc = pdfium.load_pdf_from_file(&path, None).expect("open it");
    let texts: Vec<String> = doc
        .pages()
        .iter()
        .map(|page| page.text().expect("the page's text").all())
        .collect();
    assert!(
        texts[0].contains("Day summary") && texts[0].contains("Couriers"),
        "{}",
        texts[0]
    );
    assert!(texts[1].contains("Pick summary"), "{}", texts[1]);
}
