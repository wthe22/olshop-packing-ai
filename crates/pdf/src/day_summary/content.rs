//! What the day summary prints, as text, from the date's live batches: the header's counts, the
//! batches with their group PDFs, the couriers and the day's picks. Plain functions, so they are
//! tested without PDFium.

use super::DayFacts;
use crate::facts::{Row, RowKind, Section, count};
use crate::summary_pdf::content::{counts as batch_counts, courier_rows, picks as batch_picks};

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

/// The batches, then the couriers of the whole day.
pub(crate) fn sections(facts: &DayFacts<'_>) -> Vec<Section> {
    vec![
        Section {
            heading: "Batches",
            rows: batches(facts),
        },
        Section {
            heading: "Couriers",
            rows: courier_rows(facts.batches.iter().flat_map(|batch| &batch.record.orders)),
        },
    ]
}

/// Each batch, newest first, with its orders and pages; then its group PDFs with theirs and their
/// Printed tick.
fn batches(facts: &DayFacts<'_>) -> Vec<Row> {
    let mut rows = Vec::new();
    for batch in &facts.batches {
        let record = batch.record;
        let groups: Vec<_> = record
            .outputs
            .iter()
            .filter_map(|output| Some((output, output.group.as_ref()?)))
            .collect();
        let pages = groups
            .iter()
            .map(|(output, _)| usize::from(output.pages))
            .sum();
        let amended = if record.amends { " · amended" } else { "" };
        let head = format!(
            "Batch {} · saved {} · sort rule {}{amended}",
            record.batch, batch.time, record.sort_rule.name
        );
        rows.push(
            Row::new(RowKind::Head, head)
                .count(0, count(record.orders.len(), "order", "orders"))
                .count(1, count(pages, "page", "pages")),
        );
        for (output, code) in groups {
            let orders = record.orders.iter().filter(|o| &o.group == code).count();
            let printed = if batch.printed.contains(code) {
                "printed"
            } else {
                "not printed"
            };
            rows.push(
                Row::new(RowKind::Part, output.file.clone())
                    .count(0, count(orders, "order", "orders"))
                    .count(1, count(usize::from(output.pages), "page", "pages"))
                    .count(2, printed),
            );
        }
    }
    rows
}

/// The units to take from stock per item over every batch of the day.
pub(crate) fn picks(facts: &DayFacts<'_>) -> Vec<(i64, String)> {
    batch_picks(facts.batches.iter().map(|batch| batch.record))
}
