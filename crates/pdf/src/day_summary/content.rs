//! What the day summary prints, as text, from the date's live batches: the header's counts, each
//! batch with its groups as its Day card shows them, the couriers and the day's picks. Plain
//! functions, so they are tested without PDFium.

use super::{DayBatch, DayFacts};
use crate::facts::{Row, Section, count};
use crate::summary_pdf::content::{
    counts as batch_counts, courier_rows, group_sections, picks as batch_picks,
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
