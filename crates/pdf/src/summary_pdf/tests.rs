use jiff::civil::date;
use packing_engine::ids::CourierCode;
use packing_engine::record::{BatchSaved, OutputFile};
use packing_engine::workflows::Courier;
use pdfium_render::prelude::*;
use testkit::{TempDir, batch, sha, vendor_dir};

use super::content::{ContentRow, Counts, GroupSection, counts, facts, group_sections, picks};
use super::{SummaryFacts, write_summary};
use crate::facts::{Row, Section};

fn output(file: &str, group: Option<&str>, pages: u16) -> OutputFile {
    OutputFile {
        file: file.to_string(),
        group: group.map(str::to_string),
        pages,
        size: 1,
        sha256: sha('b'),
    }
}

/// The made-up batch: two orders of one content in group A, one in group Z, a repeat left out.
fn record() -> BatchSaved {
    batch(
        1,
        false,
        vec![
            output("1-A Sepatu x2.pdf", Some("A"), 2),
            output("1-Z Lainnya x1.pdf", Some("Z"), 1),
        ],
    )
}

fn couriers() -> Vec<Courier> {
    vec![Courier {
        code: CourierCode::new("jnt"),
        name: "J&T Express".into(),
    }]
}

#[test]
fn the_counts_and_the_picks_come_from_the_record() {
    let record = record();
    assert_eq!(
        counts(&record),
        Counts {
            orders: 3,
            units: 4,
            groups: 2
        }
    );
    assert_eq!(
        picks([&record]),
        vec![
            (2, "Sepatu Standar Samping Motor — honda".to_string()),
            (2, "Spion Beat".to_string()),
        ]
    );
}

#[test]
fn each_group_lists_its_order_contents_in_printing_order() {
    assert_eq!(
        group_sections(&record()),
        vec![
            GroupSection {
                code: "A".to_string(),
                name: "Sepatu".to_string(),
                orders: 2,
                units: 2,
                contents: vec![ContentRow {
                    number: "A-1".to_string(),
                    items: vec!["Sepatu Standar Samping Motor — honda ×1".to_string()],
                    orders: 2,
                }],
            },
            GroupSection {
                code: "Z".to_string(),
                name: "Lainnya".to_string(),
                orders: 1,
                units: 2,
                contents: vec![ContentRow {
                    number: "Z-1".to_string(),
                    items: vec!["Spion Beat ×2".to_string()],
                    orders: 1,
                }],
            },
        ]
    );
}

#[test]
fn the_facts_page_says_what_the_batch_is() {
    let section = |heading: &str, rows: Vec<Row>| Section {
        heading: heading.to_string(),
        right: None,
        rows,
    };
    let mut record = record();
    record.orders[2].courier = None;
    assert_eq!(
        facts(&record, "0.2.0", &couriers()),
        vec![
            section(
                "Couriers",
                vec![
                    Row::new("J&T Express").count(0, 2, "order", "orders"),
                    Row::new("Unknown").count(0, 1, "order", "orders"),
                ]
            ),
            section(
                "Warnings",
                vec![Row::new(
                    "Slip total differs for JY0000000002: printed 3, lines add up to 2."
                )]
            ),
            section(
                "Inputs",
                vec![
                    Row::new("labels_1.pdf")
                        .count(0, 4, "page", "pages")
                        .tail("aaaaaaaaaaaa"),
                    Row::new("4 pages read = 3 written + 1 left out (1 repeats, 0 saved earlier)")
                        .bold(),
                ]
            ),
            section(
                "Sort rule · default",
                vec![
                    Row::new("Sepatu · name contains \"sepatu\"").key("A"),
                    Row::new("Lainnya · every order left").key("Z"),
                ]
            ),
            section(
                "Versions",
                vec![Row::new(
                    "schema 1.0 · Packing v0.2.0 · PDFium chromium/7881"
                )]
            ),
        ]
    );
}

#[test]
fn a_courier_the_rules_no_longer_name_shows_its_code() {
    let rows = &facts(&record(), "0.2.0", &[])[0].rows;
    assert_eq!(rows, &[Row::new("jnt").count(0, 3, "order", "orders")]);
}

/// The summary of `record`, written with the vendored PDFium; `None` (the test skips) without it.
fn written(tag: &str, record: &BatchSaved, check: impl FnOnce(&Pdfium, &std::path::Path)) {
    let Some(vendor) = vendor_dir() else {
        eprintln!("skipped: run tools/get-pdfium.sh");
        return;
    };
    let _guard = testkit::pdfium_lock();
    let pdfium = crate::bind(&vendor).expect("bind pdfium");
    let dir = TempDir::new(tag);
    let path = dir.path().join("1 summary.pdf");
    let couriers = couriers();
    write_summary(
        &pdfium,
        &path,
        &SummaryFacts {
            workspace: "Toko Contoh",
            date: date(2026, 10, 7),
            record,
            printed: "09:12",
            app: "0.2.0",
            couriers: &couriers,
        },
    )
    .expect("write the summary");
    check(&pdfium, &path);
}

#[test]
fn the_facts_then_the_groups_then_the_picks_each_from_a_new_page() {
    written("summary-pages", &record(), |pdfium, path| {
        let doc = pdfium.load_pdf_from_file(path, None).expect("open it");
        let texts: Vec<String> = doc
            .pages()
            .iter()
            .map(|page| page.text().expect("the page's text").all())
            .collect();
        assert_eq!(texts.len(), 3);
        assert!(
            texts[0].contains("pages read") && texts[0].contains("J&T Express"),
            "{}",
            texts[0]
        );
        assert!(
            texts[1].contains("Z-1") && !texts[1].contains("pages read"),
            "{}",
            texts[1]
        );
        assert!(texts[2].contains("Pick summary"), "{}", texts[2]);
    });
}

/// The same item text drawn twice keeps the space before its `×`: measuring a text a second time
/// once read the origins of PDFium's merged copy and drew the `×` over the space.
#[test]
fn the_same_item_drawn_twice_is_drawn_alike() {
    let mut record = record();
    record.orders[2].content = 0;
    written("summary-same-item", &record, |pdfium, path| {
        let doc = pdfium.load_pdf_from_file(path, None).expect("open it");
        let page = doc.pages().get(1).expect("the group page");
        let times: Vec<f32> = page
            .objects()
            .iter()
            .filter(|object| {
                object
                    .as_text_object()
                    .is_some_and(|text| text.text() == "×")
            })
            .map(|object| object.bounds().expect("its box").left().value)
            .collect();
        assert_eq!(times.len(), 2, "one × per group");
        assert!(
            (times[0] - times[1]).abs() < 0.01,
            "the × of A-1 at {} pt, of Z-1 at {} pt",
            times[0],
            times[1]
        );
    });
}
