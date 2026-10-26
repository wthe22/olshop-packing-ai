//! The search index: every live order of every date of a namespace, found by a filter on one date
//! or all, a reverted batch gone, a damaged date left out and named.

use std::fs;

use jiff::civil::date;
use packing_engine::record::{BatchReverted, RecordBody};
use packing_engine::rules::parse_filter;
use packing_store::{DateFolder, Journal, SearchIndex};
use testkit::TempDir;

use crate::common::{APP, NAMESPACE, batch};

fn commit(data: &TempDir, day: i8, bodies: Vec<RecordBody>) -> Journal {
    let folder = DateFolder::new(data.path(), NAMESPACE, date(2026, 10, day));
    let mut journal = Journal::open(&folder).unwrap();
    journal.commit(bodies, APP).unwrap();
    journal
}

fn ids(index: &SearchIndex, filter: &str, day: Option<i8>) -> Vec<String> {
    let filter = parse_filter(filter).unwrap();
    index
        .search(&filter, day.map(|day| date(2026, 10, day)))
        .iter()
        .map(|order| format!("{} {}", order.date.day(), order.order.order_id))
        .collect()
}

#[test]
fn every_live_order_of_the_namespace_is_found_by_its_fields() {
    let data = TempDir::new("index");
    commit(
        &data,
        7,
        vec![RecordBody::BatchSaved(batch(1, false, Vec::new()))],
    );
    commit(
        &data,
        8,
        vec![RecordBody::BatchSaved(batch(1, false, Vec::new()))],
    );
    commit(
        &data,
        8,
        vec![RecordBody::BatchSaved(batch(2, false, Vec::new()))],
    );
    let mut index = SearchIndex::build(data.path(), NAMESPACE).unwrap();

    let cases = [
        (
            "order_id equals \"580000000000000002\"",
            None,
            vec![
                "7 580000000000000002",
                "8 580000000000000002",
                "8 580000000000000002",
            ],
        ),
        (
            "tracking_id starts_with \"jy0000000003\"",
            Some(7),
            vec!["7 580000000000000003"],
        ),
        (
            "name contains \"spion\" and line_quantity = 2",
            Some(7),
            vec!["7 580000000000000002"],
        ),
        (
            "seller_sku equals \"SELL-SEP-01\" and variation equals \"honda\"",
            Some(7),
            vec!["7 580000000000000001", "7 580000000000000003"],
        ),
        (
            "courier equals \"jnt\" and ship_by = \"2026-10-08\" and total_quantity = 1",
            Some(9),
            vec![],
        ),
        (
            "display_name equals \"Sepatu Standar Samping Motor — honda\"",
            Some(7),
            vec!["7 580000000000000001", "7 580000000000000003"],
        ),
    ];
    for (filter, day, want) in cases {
        assert_eq!(ids(&index, filter, day), want, "{filter}");
    }
    assert_eq!(ids(&index, "distinct_items >= 1", None).len(), 9);
    assert_eq!(index.counts().get(&date(2026, 10, 7)), Some(&3));
    assert_eq!(index.counts().get(&date(2026, 10, 8)), Some(&6));

    let found = &index.search(
        &parse_filter("order_id equals \"580000000000000002\"").unwrap(),
        Some(date(2026, 10, 7)),
    )[0];
    assert_eq!(
        (found.file.as_str(), found.input.as_str()),
        ("", "labels_1.pdf")
    );
    assert_eq!(found.items[0].name, "Spion Beat");

    let mut eighth =
        Journal::open(&DateFolder::new(data.path(), NAMESPACE, date(2026, 10, 8))).unwrap();
    eighth
        .commit(
            vec![RecordBody::BatchReverted(BatchReverted {
                batch: packing_engine::ids::BatchId::new(2),
            })],
            APP,
        )
        .unwrap();
    index.refresh(eighth.fold());
    assert_eq!(
        index.counts().get(&date(2026, 10, 8)),
        Some(&3),
        "batch 2 reverted"
    );
}

#[test]
fn a_damaged_date_is_left_out_and_named() {
    let data = TempDir::new("index-damaged");
    commit(
        &data,
        7,
        vec![RecordBody::BatchSaved(batch(1, false, Vec::new()))],
    );
    let journal = data
        .path()
        .join(NAMESPACE)
        .join("2026-10-07")
        .join("journal.jsonl");
    let text = fs::read_to_string(&journal).unwrap();
    fs::write(&journal, format!("0000000000000000 {{}}\n{text}")).unwrap();
    let index = SearchIndex::build(data.path(), NAMESPACE).unwrap();
    assert!(index.counts().is_empty());
    assert_eq!(
        index.damaged.keys().copied().collect::<Vec<_>>(),
        [date(2026, 10, 7)]
    );
}
