//! What the summary PDF prints, as text, from the batch's record: the header's counts, the facts
//! pages' sections, the groups with their numbered order contents and the pick lines. Plain
//! functions, so they are tested without PDFium.

use std::collections::BTreeMap;

use packing_engine::ids::CourierCode;
use packing_engine::orders::display_name;
use packing_engine::record::{BatchSaved, LeftOutReason, SCHEMA, SavedOrder};
use packing_engine::sorting::{REST_CODE, REST_NAME};
use packing_engine::workflows::{Courier, courier_name};

use crate::PDFIUM_VERSION;
use crate::facts::{Row, Section};

/// The header's counts: orders, units and group PDFs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Counts {
    pub orders: usize,
    pub units: u32,
    pub groups: usize,
}

pub(crate) fn counts(record: &BatchSaved) -> Counts {
    let units = record
        .orders
        .iter()
        .flat_map(|order| &record.contents[order.content as usize].items)
        .map(|item| u32::from(item.quantity))
        .sum();
    Counts {
        orders: record.orders.len(),
        units,
        groups: record.outputs.iter().filter(|o| o.group.is_some()).count(),
    }
}

/// The units to take from stock per item (name and variation) over the batches, as (units,
/// display name), sorted by display name, then name and variation.
pub(crate) fn picks<'r>(records: impl IntoIterator<Item = &'r BatchSaved>) -> Vec<(i64, String)> {
    let mut totals: Vec<((String, String, String), i64)> = Vec::new();
    for record in records {
        for order in &record.orders {
            for item in &record.contents[order.content as usize].items {
                let key = (
                    display_name(&item.name, &item.variant),
                    item.name.clone(),
                    item.variant.clone(),
                );
                match totals.iter_mut().find(|(known, _)| *known == key) {
                    Some((_, units)) => *units += i64::from(item.quantity),
                    None => totals.push((key, i64::from(item.quantity))),
                }
            }
        }
    }
    totals.sort();
    totals
        .into_iter()
        .map(|((display, _, _), units)| (units, display))
        .collect()
}

/// One group PDF of the batch: its code and name, its counts, and its order contents in the order
/// the group PDF prints them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GroupSection {
    pub code: String,
    pub name: String,
    pub orders: usize,
    pub units: u32,
    pub contents: Vec<ContentRow>,
}

/// One order content of a group: its number in the group (`A-1`), its items as
/// `display name ×quantity`, and its orders.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ContentRow {
    pub number: String,
    pub items: Vec<String>,
    pub orders: usize,
}

/// Each group PDF in group order, with its order contents.
pub(crate) fn group_sections(record: &BatchSaved) -> Vec<GroupSection> {
    record
        .outputs
        .iter()
        .filter_map(|output| {
            let code = output.group.as_ref()?;
            let mut contents: Vec<(u16, usize)> = Vec::new();
            for order in record.orders.iter().filter(|order| &order.group == code) {
                match contents
                    .iter_mut()
                    .find(|(content, _)| *content == order.content)
                {
                    Some((_, orders)) => *orders += 1,
                    None => contents.push((order.content, 1)),
                }
            }
            let items = |content: u16| &record.contents[content as usize].items;
            let units = contents
                .iter()
                .map(|&(content, orders)| {
                    let each: u32 = items(content).iter().map(|i| u32::from(i.quantity)).sum();
                    each * orders as u32
                })
                .sum();
            Some(GroupSection {
                code: code.clone(),
                name: group_name(record, code).to_string(),
                orders: contents.iter().map(|(_, orders)| orders).sum(),
                units,
                contents: contents
                    .iter()
                    .enumerate()
                    .map(|(index, &(content, orders))| ContentRow {
                        number: format!("{code}-{}", index + 1),
                        items: items(content)
                            .iter()
                            .map(|item| {
                                let name = display_name(&item.name, &item.variant);
                                format!("{name} ×{}", item.quantity)
                            })
                            .collect(),
                        orders,
                    })
                    .collect(),
            })
        })
        .collect()
}

/// The group's name in the batch's sort rule; the orders no group took are *Uncategorised*.
fn group_name<'r>(record: &'r BatchSaved, code: &str) -> &'r str {
    if code == REST_CODE {
        return REST_NAME;
    }
    record
        .sort_rule
        .label_groups
        .iter()
        .find(|group| group.code == code)
        .map_or("", |group| group.name.as_str())
}

/// The facts pages: the couriers, the warnings, the inputs with the ledger, the sort rule and the
/// versions. `couriers` names the courier codes.
pub(crate) fn facts(record: &BatchSaved, app: &str, couriers: &[Courier]) -> Vec<Section> {
    let section = |heading: &str, rows: Vec<Row>| Section {
        heading: heading.to_string(),
        right: None,
        rows,
    };
    vec![
        section("Couriers", courier_rows(&record.orders, couriers)),
        section(
            "Warnings",
            if record.warnings.is_empty() {
                vec![Row::new("none")]
            } else {
                record
                    .warnings
                    .iter()
                    .map(|w| Row::new(w.text.clone()))
                    .collect()
            },
        ),
        section(
            "Inputs",
            record
                .inputs
                .iter()
                .map(|input| {
                    let sha = input.sha256.as_str();
                    Row::new(input.filename.clone())
                        .count(0, usize::from(input.page_count), "page", "pages")
                        .tail(&sha[..12.min(sha.len())])
                })
                .chain([Row::new(ledger(record)).bold()])
                .collect(),
        ),
        section(
            &format!("Sort rule · {}", record.sort_rule.name),
            record
                .sort_rule
                .label_groups
                .iter()
                .map(|group| {
                    let filter = group.filter.as_deref().unwrap_or("every order left");
                    Row::new(format!("{} · {filter}", group.name)).key(group.code.clone())
                })
                .collect(),
        ),
        section(
            "Versions",
            vec![Row::new(format!(
                "schema {SCHEMA} · Packing v{app} · PDFium {PDFIUM_VERSION}"
            ))],
        ),
    ]
}

/// The orders per courier, by the courier's name in `couriers` (its code when the file no longer
/// names it), most first; the orders no clue named are *Unknown*, last.
pub(crate) fn courier_rows<'r>(
    orders: impl IntoIterator<Item = &'r SavedOrder>,
    couriers: &[Courier],
) -> Vec<Row> {
    let mut known: BTreeMap<&CourierCode, usize> = BTreeMap::new();
    let mut unknown = 0;
    for order in orders {
        match &order.courier {
            Some(code) => *known.entry(code).or_default() += 1,
            None => unknown += 1,
        }
    }
    let mut known: Vec<(String, usize)> = known
        .into_iter()
        .map(|(code, n)| (courier_name(couriers, code.as_str()).to_string(), n))
        .collect();
    known.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    let unknown = (unknown > 0).then(|| ("Unknown".to_string(), unknown));
    known
        .into_iter()
        .chain(unknown)
        .map(|(name, n)| Row::new(name).count(0, n, "order", "orders"))
        .collect()
}

/// `<read> pages read = <written> written + <left> left out (<r> repeats, <s> saved earlier)`, in
/// pages.
fn ledger(record: &BatchSaved) -> String {
    let pages = |start: u16, end: u16| u32::from(end - start) + 1;
    let read: u32 = record.inputs.iter().map(|i| u32::from(i.page_count)).sum();
    let written: u32 = record
        .orders
        .iter()
        .map(|o| pages(o.page_start, o.page_end))
        .sum();
    let (mut repeats, mut earlier) = (0, 0);
    for left in &record.left_out {
        match left.reason {
            LeftOutReason::Repeat { .. } => repeats += pages(left.page_start, left.page_end),
            LeftOutReason::SavedEarlier { .. } => earlier += pages(left.page_start, left.page_end),
        }
    }
    format!(
        "{read} pages read = {written} written + {} left out ({repeats} repeats, {earlier} saved earlier)",
        repeats + earlier
    )
}
