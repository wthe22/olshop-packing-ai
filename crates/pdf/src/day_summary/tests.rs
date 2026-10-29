use jiff::civil::date;
use packing_engine::ids::CourierCode;
use packing_engine::record::{BatchSaved, ItemPick, OrderContent, OutputFile};
use packing_engine::workflows::Courier;
use testkit::{TempDir, batch, sha, vendor_dir};

use super::content::{contents_counts, header_counts, order_contents, picks, sections};
use super::{DayBatch, DayFacts, write_day_summary};
use crate::facts::Row;
use crate::summary_pdf::content::ContentRow;

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

fn couriers() -> Vec<Courier> {
    vec![Courier {
        code: CourierCode::new("jnt"),
        name: "J&T Express".into(),
    }]
}

fn facts<'a>(
    first: &'a BatchSaved,
    second: &'a BatchSaved,
    couriers: &'a [Courier],
) -> DayFacts<'a> {
    DayFacts {
        workspace: "Toko Contoh",
        date: date(2026, 10, 7),
        batches: vec![
            DayBatch {
                record: second,
                time: "09:12".into(),
            },
            DayBatch {
                record: first,
                time: "07:40".into(),
            },
        ],
        printed: "17:05",
        couriers,
    }
}

#[test]
fn the_day_reads_its_batches_as_their_cards_and_its_couriers() {
    let (first, second) = records();
    let couriers = couriers();
    let day = facts(&first, &second, &couriers);
    assert_eq!(
        header_counts(&day),
        "6 orders · 8 units · 2 batches · 4 groups"
    );

    let group = |key: &str, name: &str, orders: usize| {
        Row::new(name)
            .key(key)
            .count(0, orders, "order", "orders")
            .count(1, 1, "content", "contents")
    };
    let shown = sections(&day);
    let bars: Vec<(&str, Option<&str>)> = shown
        .iter()
        .map(|s| (s.heading.as_str(), s.right.as_deref()))
        .collect();
    assert_eq!(
        bars,
        [
            (
                "Batch 2 · saved 09:12 · 1 file · amended",
                Some("3 orders · 4 units")
            ),
            ("Batch 1 · saved 07:40 · 1 file", Some("3 orders · 4 units")),
            ("Couriers", None),
            ("Inputs", None),
        ]
    );
    assert_eq!(
        shown[0].rows,
        [group("2-A", "Sepatu", 2), group("2-Z", "Lainnya", 1)]
    );
    assert_eq!(
        shown[2].rows,
        [
            Row::new("J&T Express").count(0, 5, "order", "orders"),
            Row::new("Unknown").count(0, 1, "order", "orders"),
        ]
    );
    let inputs = |batch: &str| {
        [
            Row::new("labels_1.pdf")
                .count(0, 4, "page", "pages")
                .tail("aaaaaaaaaaaa")
                .key(batch),
            Row::new("4 pages read = 3 written + 1 left out (1 repeats, 0 saved earlier)")
                .bold()
                .key(batch),
        ]
    };
    assert_eq!(
        shown[3].rows,
        [inputs("2"), inputs("1")].concat(),
        "each batch's inputs and ledger, newest batch first"
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
fn the_order_contents_are_the_days_most_orders_first_one_line_ones_before_mixed() {
    let (first, mut second) = records();
    let item = |name: &str, quantity| ItemPick {
        name: name.into(),
        variant: String::new(),
        quantity,
    };
    second.contents.push(OrderContent {
        items: vec![item("Kaca Spion", 1), item("Spion Beat", 1)],
    });
    second.contents.push(OrderContent {
        items: vec![item("Spion Beat", 1)],
    });
    second.orders[0].content = 2;
    second.orders[1].content = 3;
    let couriers = couriers();
    let day = facts(&first, &second, &couriers);

    let row = |number: &str, items: &[&str], orders| ContentRow {
        number: number.into(),
        items: items.iter().map(|item| item.to_string()).collect(),
        orders,
    };
    let contents = order_contents(&day);
    assert_eq!(
        contents,
        [
            row("1", &["Sepatu Standar Samping Motor — honda ×1"], 2),
            row("2", &["Spion Beat ×2"], 2),
            row("3", &["Spion Beat ×1"], 1),
            row("4", &["Kaca Spion ×1", "Spion Beat ×1"], 1),
        ],
        "the same content over both batches is one row; most orders first"
    );
    assert_eq!(contents_counts(&contents), "4 contents · 6 orders");
}

#[test]
fn the_batches_then_the_picks_then_the_order_contents_from_new_pages() {
    let Some(vendor) = vendor_dir() else {
        eprintln!("skipped: run tools/get-pdfium.sh");
        return;
    };
    let _guard = testkit::pdfium_lock();
    let pdfium = crate::bind(&vendor).expect("bind pdfium");
    let dir = TempDir::new("day-summary");
    let path = dir.path().join("day summary.pdf");
    let (first, second) = records();
    let couriers = couriers();
    let pages = write_day_summary(&pdfium, &path, &facts(&first, &second, &couriers))
        .expect("write the day summary");
    assert_eq!(pages, 3);
    let doc = pdfium.load_pdf_from_file(&path, None).expect("open it");
    let texts: Vec<String> = doc
        .pages()
        .iter()
        .map(|page| page.text().expect("the page's text").all())
        .collect();
    assert!(
        texts[0].contains("Day summary")
            && texts[0].contains("Couriers")
            && texts[0].contains("labels_1.pdf"),
        "{}",
        texts[0]
    );
    assert!(texts[1].contains("Pick summary"), "{}", texts[1]);
    assert!(texts[2].contains("Order contents"), "{}", texts[2]);
}
