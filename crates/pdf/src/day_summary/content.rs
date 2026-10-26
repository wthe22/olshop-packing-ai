//! What the day summary prints, as text, from the date's live batches and reprints: the header's
//! counts, the couriers, the batches with their group PDFs, the reprints and the day's picks. Plain
//! functions, so they are tested without PDFium.

use std::collections::BTreeMap;

use packing_engine::sorting::{REST_CODE, REST_NAME};

use super::{DayBatch, DayFacts};
use crate::summary_pdf::content::{counts as batch_counts, picks as batch_picks};

/// `1 order`, `3 orders`.
pub(crate) fn count(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// The header's counts line: `729 orders · 801 units · 2 batches · 5 group PDFs · 1 reprint`.
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
        count(
            each.iter().map(|c| c.groups).sum(),
            "group PDF",
            "group PDFs",
        ),
        count(facts.reprints.len(), "reprint", "reprints"),
    ]
    .join(" · ")
}

/// How a line is drawn: a section's own line, the head of a batch or reprint, or one of its parts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LineKind {
    Plain,
    Head,
    Part,
}

/// One line of a section: its text, and a count at the right edge.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DayLine {
    pub kind: LineKind,
    pub text: String,
    pub right: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DaySection {
    pub heading: &'static str,
    pub lines: Vec<DayLine>,
}

fn line(kind: LineKind, text: String, right: Option<String>) -> DayLine {
    DayLine { kind, text, right }
}

/// The couriers, the batches and the reprints.
pub(crate) fn sections(facts: &DayFacts<'_>) -> Vec<DaySection> {
    vec![
        DaySection {
            heading: "Couriers",
            lines: couriers(facts),
        },
        DaySection {
            heading: "Batches",
            lines: batches(facts),
        },
        DaySection {
            heading: "Reprints",
            lines: reprints(facts),
        },
    ]
}

/// The day's orders per courier code, most first; the orders no clue named are *unknown*, last.
fn couriers(facts: &DayFacts<'_>) -> Vec<DayLine> {
    let mut known: BTreeMap<&str, usize> = BTreeMap::new();
    let mut unknown = 0;
    for order in facts.batches.iter().flat_map(|batch| &batch.record.orders) {
        match &order.courier {
            Some(code) => *known.entry(code.as_str()).or_default() += 1,
            None => unknown += 1,
        }
    }
    let mut known: Vec<(&str, usize)> = known.into_iter().collect();
    known.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    let mut lines: Vec<DayLine> = known
        .into_iter()
        .map(|(code, n)| {
            let orders = count(n, "order", "orders");
            line(LineKind::Plain, code.to_string(), Some(orders))
        })
        .collect();
    if unknown > 0 {
        let orders = count(unknown, "order", "orders");
        lines.push(line(LineKind::Plain, "unknown".to_string(), Some(orders)));
    }
    lines
}

/// Each batch, newest first, then its group PDFs with their counts and Printed tick.
fn batches(facts: &DayFacts<'_>) -> Vec<DayLine> {
    let mut lines = Vec::new();
    for batch in &facts.batches {
        let record = batch.record;
        let counts = batch_counts(record);
        let amended = if record.amends { " · amended" } else { "" };
        lines.push(line(
            LineKind::Head,
            format!(
                "Batch {} · saved {} · sort rule {}{amended}",
                record.batch, batch.time, record.sort_rule.name
            ),
            Some(format!(
                "{} · {}",
                count(counts.orders, "order", "orders"),
                count(counts.units as usize, "unit", "units")
            )),
        ));
        for code in record.outputs.iter().filter_map(|o| o.group.as_ref()) {
            let mut contents: Vec<u16> = record
                .orders
                .iter()
                .filter(|order| &order.group == code)
                .map(|order| order.content)
                .collect();
            let orders = contents.len();
            contents.sort_unstable();
            contents.dedup();
            let printed = if batch.printed.contains(code) {
                "printed"
            } else {
                "not printed"
            };
            lines.push(line(
                LineKind::Part,
                format!("{}-{code} {}", record.batch, group_name(batch, code)),
                Some(format!(
                    "{} | {} · {printed}",
                    count(orders, "order", "orders"),
                    count(contents.len(), "content", "contents")
                )),
            ));
        }
    }
    lines
}

fn group_name<'r>(batch: &DayBatch<'r>, code: &str) -> &'r str {
    if code == REST_CODE {
        return REST_NAME;
    }
    batch
        .record
        .sort_rule
        .label_groups
        .iter()
        .find(|group| group.code == code)
        .map_or("", |group| group.name.as_str())
}

/// Each reprint of the date, then its orders with the date and batch they were saved in.
fn reprints(facts: &DayFacts<'_>) -> Vec<DayLine> {
    if facts.reprints.is_empty() {
        return vec![line(LineKind::Plain, "none".to_string(), None)];
    }
    let mut lines = Vec::new();
    for reprint in &facts.reprints {
        let made = reprint.made;
        lines.push(line(
            LineKind::Head,
            format!("{} · made {}", made.file, reprint.time),
            Some(format!(
                "{} · {}",
                count(made.orders.len(), "order", "orders"),
                count(usize::from(made.pages), "page", "pages")
            )),
        ));
        for order in &made.orders {
            lines.push(line(
                LineKind::Part,
                format!(
                    "{} · saved {} in batch {}",
                    order.order_id, order.date, order.batch
                ),
                None,
            ));
        }
    }
    lines
}

/// The units to take from stock per item over every batch of the day.
pub(crate) fn picks(facts: &DayFacts<'_>) -> Vec<(i64, String)> {
    batch_picks(facts.batches.iter().map(|batch| batch.record))
}
