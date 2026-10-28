//! The search index: every live order of every date of a workspace, found by a filter on a range
//! of dates or all, a reverted batch gone, a damaged date left out and named, an order's reprints.

use std::fs;

use jiff::civil::{Date, date};
use packing_engine::ids::{BatchId, OrderId};
use packing_engine::record::{BatchReverted, RecordBody, ReprintMade, ReprintedOrder};
use packing_engine::rules::parse_filter;
use packing_store::{DateFolder, Journal, ReprintMark, SearchIndex};
use testkit::{TempDir, sha};

use crate::common::{APP, WORKSPACE, batch};

fn commit(data: &TempDir, day: i8, bodies: Vec<RecordBody>) -> Journal {
    let folder = DateFolder::new(data.path(), WORKSPACE, date(2026, 10, day));
    let mut journal = Journal::open(&folder).unwrap();
    journal.commit(bodies, APP).unwrap();
    journal
}

/// The orders found on the October dates `days` (both ends included), or on every date.
fn ids(index: &SearchIndex, filter: &str, days: Option<(i8, i8)>) -> Vec<String> {
    let filter = parse_filter(filter).unwrap();
    let dates = days.map_or(Date::MIN..=Date::MAX, |(from, to)| {
        date(2026, 10, from)..=date(2026, 10, to)
    });
    index
        .search(&filter, dates, &[])
        .iter()
        .map(|order| format!("{} {}", order.date.day(), order.order.order_id))
        .collect()
}

#[test]
fn every_live_order_of_the_workspace_is_found_by_its_fields() {
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
    let mut index = SearchIndex::build(data.path(), WORKSPACE).unwrap();

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
            "order_id equals \"580000000000000002\"",
            Some((8, 9)),
            vec!["8 580000000000000002", "8 580000000000000002"],
        ),
        (
            "order_id equals \"580000000000000002\"",
            Some((6, 7)),
            vec!["7 580000000000000002"],
        ),
        (
            "tracking_id starts_with \"jy0000000003\"",
            Some((7, 7)),
            vec!["7 580000000000000003"],
        ),
        (
            "name contains \"spion\" and line_quantity = 2",
            Some((7, 7)),
            vec!["7 580000000000000002"],
        ),
        (
            "seller_sku equals \"SELL-SEP-01\" and variation equals \"honda\"",
            Some((7, 7)),
            vec!["7 580000000000000001", "7 580000000000000003"],
        ),
        (
            "courier equals \"jnt\" and ship_by = \"2026-10-08\" and total_quantity = 1",
            Some((9, 9)),
            vec![],
        ),
        (
            "display_name equals \"Sepatu Standar Samping Motor — honda\"",
            Some((7, 7)),
            vec!["7 580000000000000001", "7 580000000000000003"],
        ),
    ];
    for (filter, day, want) in cases {
        assert_eq!(ids(&index, filter, day), want, "{filter}");
    }
    assert_eq!(ids(&index, "distinct_items >= 1", None).len(), 9);
    assert_eq!(index.counts().get(&date(2026, 10, 7)), Some(&3));
    assert_eq!(index.counts().get(&date(2026, 10, 8)), Some(&6));

    let seventh = |filter: &str| {
        index.search(
            &parse_filter(filter).unwrap(),
            date(2026, 10, 7)..=date(2026, 10, 7),
            &[],
        )
    };
    let found = seventh("order_id equals \"580000000000000002\"")[0];
    assert_eq!(
        (found.file.as_str(), found.input.as_str()),
        ("", "labels_1.pdf")
    );
    assert_eq!(found.items[0].name, "Spion Beat");
    let pages: Vec<_> = seventh("distinct_items >= 1")
        .iter()
        .map(|order| (order.order.group.as_str(), order.file_pages.clone()))
        .collect();
    assert_eq!(
        pages,
        [("A", 1..=1), ("A", 2..=2), ("Z", 1..=1)],
        "each order's pages in its group PDF follow the orders of its group before it"
    );

    let mut eighth =
        Journal::open(&DateFolder::new(data.path(), WORKSPACE, date(2026, 10, 8))).unwrap();
    eighth
        .commit(
            vec![RecordBody::BatchReverted(BatchReverted {
                batch: BatchId::new(2),
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
        .join(WORKSPACE)
        .join("2026-10-07")
        .join("journal.jsonl");
    let text = fs::read_to_string(&journal).unwrap();
    fs::write(&journal, format!("0000000000000000 {{}}\n{text}")).unwrap();
    let index = SearchIndex::build(data.path(), WORKSPACE).unwrap();
    assert!(index.counts().is_empty());
    assert_eq!(
        index.damaged.keys().copied().collect::<Vec<_>>(),
        [date(2026, 10, 7)]
    );
}

#[test]
fn an_order_printed_again_carries_each_reprint_and_is_found_by_its_date() {
    let data = TempDir::new("index-reprints");
    commit(
        &data,
        7,
        vec![RecordBody::BatchSaved(batch(1, false, Vec::new()))],
    );
    let printed = |number: u8, ids: &[&str]| {
        RecordBody::ReprintMade(ReprintMade {
            number,
            file: format!("reprint {number} x{}.pdf", ids.len()),
            pages: ids.len() as u16,
            size: 1,
            sha256: sha('c'),
            orders: ids
                .iter()
                .map(|id| ReprintedOrder {
                    order_id: OrderId::new(*id),
                    date: date(2026, 10, 7),
                    batch: BatchId::new(1),
                    sha256: sha('a'),
                    page_start: 1,
                    page_end: 1,
                })
                .collect(),
        })
    };
    commit(&data, 7, vec![printed(1, &["580000000000000001"])]);
    let mut index = SearchIndex::build(data.path(), WORKSPACE).unwrap();
    let marks = |index: &SearchIndex, id: &str| {
        let order = index
            .find(date(2026, 10, 7), &OrderId::new(id))
            .expect("saved on the 7th");
        index.reprints_of(order)
    };
    assert_eq!(
        marks(&index, "580000000000000001"),
        [ReprintMark {
            date: date(2026, 10, 7),
            number: 1
        }]
    );
    assert!(marks(&index, "580000000000000002").is_empty());
    assert!(
        index
            .find(date(2026, 10, 8), &OrderId::new("580000000000000001"))
            .is_none(),
        "not saved on the 8th"
    );

    let eighth = commit(
        &data,
        8,
        vec![printed(1, &["580000000000000001", "580000000000000002"])],
    );
    index.refresh(eighth.fold());
    assert_eq!(
        marks(&index, "580000000000000001"),
        [
            ReprintMark {
                date: date(2026, 10, 7),
                number: 1
            },
            ReprintMark {
                date: date(2026, 10, 8),
                number: 1
            }
        ],
        "a reprint of another date marks the order too"
    );
    assert_eq!(marks(&index, "580000000000000002").len(), 1);
}
