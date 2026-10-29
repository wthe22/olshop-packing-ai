//! What the day summary prints, as text, from the date's live batches: the header's counts, each
//! batch with its groups as its Day card shows them, the couriers, the day's picks and the day's
//! order contents. Plain functions, so they are tested without PDFium.

use std::collections::BTreeMap;

use packing_engine::orders::display_name;
use packing_engine::record::OrderContent;

use super::{DayBatch, DayFacts};
use crate::facts::{Row, Section, count};
use crate::summary_pdf::content::{
    ContentRow, counts as batch_counts, courier_rows, group_sections, picks as batch_picks,
};

/// The header's counts line: `729 orders · 801 units · 2 batches · 5 groups`.
pub(crate) fn header_counts(facts: &DayFacts<'_>) -> String {
    let each: Vec<_> = facts
        .batches
        .iter()
        .map(|batch| batch_counts(batch.record))
        .collect();
    let units: u32 = each.iter().map(|c| c.units).sum();
    [
        count(each.iter().map(|c| c.orders).sum(), "order", "orders"),
        count(units as usize, "unit", "units"),
        count(facts.batches.len(), "batch", "batches"),
        count(each.iter().map(|c| c.groups).sum(), "group", "groups"),
    ]
    .join(" · ")
}

/// Each batch, newest first, then the couriers of the whole day.
pub(crate) fn sections(facts: &DayFacts<'_>) -> Vec<Section> {
    facts
        .batches
        .iter()
        .map(batch_section)
        .chain([Section {
            heading: "Couriers".to_string(),
            right: None,
            rows: courier_rows(
                facts.batches.iter().flat_map(|batch| &batch.record.orders),
                facts.couriers,
            ),
        }])
        .collect()
}

/// A batch as its Day card: the batch, when it was saved and from how many files, its orders and
/// units at the right; then each group with its orders and order contents.
fn batch_section(batch: &DayBatch<'_>) -> Section {
    let record = batch.record;
    let counts = batch_counts(record);
    let amended = if record.amends { " · amended" } else { "" };
    Section {
        heading: format!(
            "Batch {} · saved {} · {}{amended}",
            record.batch,
            batch.time,
            count(record.inputs.len(), "file", "files")
        ),
        right: Some(format!(
            "{} · {}",
            count(counts.orders, "order", "orders"),
            count(counts.units as usize, "unit", "units")
        )),
        rows: group_sections(record)
            .into_iter()
            .map(|group| {
                Row::new(group.name)
                    .key(format!("{}-{}", record.batch, group.code))
                    .count(0, group.orders, "order", "orders")
                    .count(1, group.contents.len(), "content", "contents")
            })
            .collect(),
    }
}

/// The units to take from stock per item over every batch of the day.
pub(crate) fn picks(facts: &DayFacts<'_>) -> Vec<(i64, String)> {
    batch_picks(facts.batches.iter().map(|batch| batch.record))
}

/// An order content's lines as (display name, quantity).
type Lines = Vec<(String, u16)>;

/// Each order content of the day once, over every batch and group, numbered from 1, with its
/// orders: how many orders are one item, two of it, or a mix. The contents of one line come first,
/// by display name and then quantity, so an item's `×1` and `×2` stand together; the mixed ones
/// follow, by their lines.
pub(crate) fn order_contents(facts: &DayFacts<'_>) -> Vec<ContentRow> {
    let mut orders: BTreeMap<&OrderContent, usize> = BTreeMap::new();
    for record in facts.batches.iter().map(|batch| batch.record) {
        for order in &record.orders {
            *orders
                .entry(&record.contents[order.content as usize])
                .or_default() += 1;
        }
    }
    let mut contents: Vec<(Lines, usize)> = orders
        .into_iter()
        .map(|(content, orders)| {
            let lines = content
                .items
                .iter()
                .map(|item| (display_name(&item.name, &item.variant), item.quantity))
                .collect();
            (lines, orders)
        })
        .collect();
    contents.sort_by(|(a, _), (b, _)| (a.len() > 1, a).cmp(&(b.len() > 1, b)));
    contents
        .into_iter()
        .enumerate()
        .map(|(index, (lines, orders))| ContentRow {
            number: (index + 1).to_string(),
            items: lines
                .into_iter()
                .map(|(name, quantity)| format!("{name} ×{quantity}"))
                .collect(),
            orders,
        })
        .collect()
}

/// The order contents' bar's counts: `3 contents · 6 orders`.
pub(crate) fn contents_counts(contents: &[ContentRow]) -> String {
    format!(
        "{} · {}",
        count(contents.len(), "content", "contents"),
        count(
            contents.iter().map(|row| row.orders).sum(),
            "order",
            "orders"
        )
    )
}
