use std::collections::BTreeSet;

use jiff::civil::date;
use packing_engine::ids::{BatchId, OrderId};
use packing_engine::record::{BatchSaved, OutputFile, ReprintMade, ReprintedOrder};
use testkit::{TempDir, batch, sha, vendor_dir};

use super::content::{DayLine, LineKind, header_counts, picks, sections};
use super::{DayBatch, DayFacts, DayReprint, write_day_summary};

fn output(file: &str, group: Option<&str>) -> OutputFile {
    OutputFile {
        file: file.to_string(),
        group: group.map(str::to_string),
        pages: 1,
        size: 1,
        sha256: sha('b'),
    }
}

/// Two made-up batches of three orders each; batch 2 amended, one of its orders with no courier.
fn records() -> (BatchSaved, BatchSaved) {
    let groups = |n: u8| {
        vec![
            output(&format!("{n}-A Sepatu x2.pdf"), Some("A")),
            output(&format!("{n}-Z Lainnya x1.pdf"), Some("Z")),
            output(&format!("{n} summary.pdf"), None),
        ]
    };
    let first = batch(1, false, groups(1));
    let mut second = batch(2, true, groups(2));
    second.orders[2].courier = None;
    (first, second)
}

fn reprint() -> ReprintMade {
    ReprintMade {
        number: 1,
        file: "reprint 1 x1.pdf".into(),
        pages: 1,
        size: 1,
        sha256: sha('c'),
        orders: vec![ReprintedOrder {
            order_id: OrderId::new("580000000000000002"),
            date: date(2026, 10, 7),
            batch: BatchId::new(1),
            sha256: sha('a'),
            page_start: 2,
            page_end: 2,
        }],
    }
}

fn facts<'a>(
    first: &'a BatchSaved,
    second: &'a BatchSaved,
    printed: &'a BTreeSet<String>,
    reprints: Vec<DayReprint<'a>>,
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
        reprints,
        printed: "17:05",
    }
}

fn texts(lines: &[DayLine]) -> Vec<(LineKind, String, Option<String>)> {
    lines
        .iter()
        .map(|line| (line.kind, line.text.clone(), line.right.clone()))
        .collect()
}

#[test]
fn the_day_reads_its_couriers_batches_and_reprints_from_the_records() {
    let (first, second) = records();
    let printed: BTreeSet<String> = ["A".to_string()].into();
    let made = reprint();
    let reprints = vec![DayReprint {
        made: &made,
        time: "09:40".into(),
    }];
    let day = facts(&first, &second, &printed, reprints);
    assert_eq!(
        header_counts(&day),
        "6 orders · 8 units · 2 batches · 4 group PDFs · 1 reprint"
    );

    let plain = |text: &str, right: Option<&str>| {
        (LineKind::Plain, text.to_string(), right.map(str::to_string))
    };
    let head = |text: &str, right: &str| (LineKind::Head, text.to_string(), Some(right.into()));
    let part = |text: &str, right: Option<&str>| {
        (LineKind::Part, text.to_string(), right.map(str::to_string))
    };
    let shown = sections(&day);
    let headings: Vec<&str> = shown.iter().map(|s| s.heading).collect();
    assert_eq!(headings, ["Couriers", "Batches", "Reprints"]);
    assert_eq!(
        texts(&shown[0].lines),
        [
            plain("jnt", Some("5 orders")),
            plain("unknown", Some("1 order"))
        ]
    );
    assert_eq!(
        texts(&shown[1].lines),
        [
            head(
                "Batch 2 · saved 09:12 · sort rule default · amended",
                "3 orders · 4 units"
            ),
            part("2-A Sepatu", Some("2 orders | 1 content · printed")),
            part("2-Z Lainnya", Some("1 order | 1 content · not printed")),
            head(
                "Batch 1 · saved 07:40 · sort rule default",
                "3 orders · 4 units"
            ),
            part("1-A Sepatu", Some("2 orders | 1 content · printed")),
            part("1-Z Lainnya", Some("1 order | 1 content · not printed")),
        ]
    );
    assert_eq!(
        texts(&shown[2].lines),
        [
            head("reprint 1 x1.pdf · made 09:40", "1 order · 1 page"),
            part("580000000000000002 · saved 2026-10-07 in batch 1", None),
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

    let none = facts(&first, &second, &printed, Vec::new());
    assert_eq!(texts(&sections(&none)[2].lines), [plain("none", None)]);
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
    let pages = write_day_summary(
        &pdfium,
        &path,
        &facts(&first, &second, &printed, Vec::new()),
    )
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
