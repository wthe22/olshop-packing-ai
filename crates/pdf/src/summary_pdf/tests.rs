use jiff::civil::date;
use packing_engine::record::{BatchSaved, OutputFile};
use testkit::{TempDir, batch, sha, vendor_dir};

use super::content::{
    ContentRow, Counts, GroupSection, Section, counts, facts, group_sections, picks,
};
use super::{SummaryFacts, write_summary};

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
        picks(&record),
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
                heading: "A  Sepatu".to_string(),
                orders: 2,
                units: 2,
                contents: vec![ContentRow {
                    items: vec!["Sepatu Standar Samping Motor — honda ×1".to_string()],
                    orders: 2,
                }],
            },
            GroupSection {
                heading: "Z  Lainnya".to_string(),
                orders: 1,
                units: 2,
                contents: vec![ContentRow {
                    items: vec!["Spion Beat ×2".to_string()],
                    orders: 1,
                }],
            },
        ]
    );
}

#[test]
fn the_facts_page_says_what_the_batch_is() {
    let section = |heading, lines: &[&str]| Section {
        heading,
        lines: lines.iter().map(|line| line.to_string()).collect(),
    };
    assert_eq!(
        facts(&record(), "0.2.0"),
        vec![
            section(
                "Groups",
                &[
                    "1-A Sepatu x2.pdf · 2 orders · 2 pages · 1 order contents",
                    "1-Z Lainnya x1.pdf · 1 orders · 1 pages · 1 order contents",
                ]
            ),
            section("Inputs", &["labels_1.pdf · 4 pages · aaaaaaaaaaaa"]),
            section(
                "Ledger",
                &["4 pages read = 3 written + 1 left out (1 repeats, 0 saved earlier)"]
            ),
            section("Instant orders", &["none"]),
            section(
                "Warnings",
                &["Slip total differs for JY0000000002: printed 3, lines add up to 2."]
            ),
            section(
                "Sort rule",
                &[
                    "default",
                    "A Sepatu · name contains \"sepatu\"",
                    "Z Lainnya · every order left"
                ]
            ),
            section(
                "Versions",
                &["schema 1.0 · Packing v0.2.0 · PDFium chromium/7881"]
            ),
        ]
    );
}

#[test]
fn the_facts_then_the_groups_then_the_picks_each_from_a_new_page() {
    let Some(vendor) = vendor_dir() else {
        eprintln!("skipped: run tools/get-pdfium.sh");
        return;
    };
    let _guard = testkit::pdfium_lock();
    let pdfium = crate::bind(&vendor).expect("bind pdfium");
    let dir = TempDir::new("summary-pages");
    let path = dir.path().join("1 summary.pdf");
    let record = record();
    let pages = write_summary(
        &pdfium,
        &path,
        &SummaryFacts {
            namespace: "Toko Contoh",
            date: date(2026, 10, 7),
            record: &record,
            printed: "09:12",
            app: "0.2.0",
        },
    )
    .expect("write the summary");
    assert_eq!(pages, 3);
    let doc = pdfium.load_pdf_from_file(&path, None).expect("open it");
    let texts: Vec<String> = doc
        .pages()
        .iter()
        .map(|page| page.text().expect("the page's text").all())
        .collect();
    assert_eq!(texts.len(), 3);
    assert!(texts[0].contains("Ledger"), "{}", texts[0]);
    assert!(
        texts[1].contains("Lainnya") && !texts[1].contains("Ledger"),
        "{}",
        texts[1]
    );
    assert!(texts[2].contains("Pick summary"), "{}", texts[2]);
}
