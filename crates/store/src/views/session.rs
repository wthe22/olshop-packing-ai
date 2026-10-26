//! `session.toml`: the date's inputs, sort rules, live batches with their group PDFs and order
//! contents, and the reprints.

use std::collections::{BTreeMap, BTreeSet};

use jiff::civil::Date;
use packing_engine::ids::{BatchId, CourierCode, OrderId, PdfId};
use packing_engine::orders::display_name;
use packing_engine::record::{
    Datetime, Finding, InputPdf, ItemPick, LabelSortRule, LeftOutCopy, SCHEMA, SavedOrder, Sha256,
};
use serde::Serialize;

use super::{ContentIds, Numbered};
use crate::fold::{Fold, LiveBatch};

#[derive(Serialize)]
struct Root<'a> {
    schema: &'static str,
    namespace: &'a str,
    date: Date,
    journal_seq: u32,
    input_pdfs: Numbered<LabelPdf<'a>>,
    label_sort_rules: Numbered<&'a LabelSortRule>,
    label_batches: Vec<LabelBatch<'a>>,
    reprints: Vec<Reprint<'a>>,
}

#[derive(Serialize)]
struct LabelPdf<'a> {
    filename: &'a str,
    size: u64,
    hash_sha256: &'a Sha256,
    create_time: &'a Datetime,
    modify_time: &'a Datetime,
    order_count: u16,
    page_count: u16,
    earliest_ship_by_date: Option<Date>,
    latest_ship_by_date: Option<Date>,
    couriers: BTreeSet<&'a CourierCode>,
    warnings: Vec<&'a str>,
    errors: Vec<&'a str>,
}

#[derive(Serialize)]
struct LabelBatch<'a> {
    id: BatchId,
    create_time: &'a Datetime,
    modify_time: &'a Datetime,
    input_pdfs: Vec<PdfId>,
    sort_rule_id: u16,
    result_groups: Vec<LabelGroup<'a>>,
    picking_list: Vec<ItemPick>,
    earliest_ship_by_date: Option<Date>,
    latest_ship_by_date: Option<Date>,
    couriers: BTreeSet<&'a CourierCode>,
    warnings: Vec<&'a str>,
    errors: Vec<&'a str>,
    left_out: &'a [LeftOutCopy],
    order_count: u16,
    summary_pdf_path: Option<String>,
}

#[derive(Serialize)]
struct LabelGroup<'a> {
    code: &'a str,
    output_path: String,
    is_marked_as_printed: bool,
    order_contents: Vec<OrderContentSummary>,
}

#[derive(Serialize)]
struct OrderContentSummary {
    content_id: u16,
    order_count: u16,
}

#[derive(Serialize)]
struct Reprint<'a> {
    number: u8,
    output_path: String,
    order_ids: Vec<&'a OrderId>,
}

pub(super) fn text(fold: &Fold, ids: &ContentIds) -> String {
    let mut rules: Vec<&LabelSortRule> = Vec::new();
    let mut batches = Vec::new();
    for (batch, live) in &fold.live {
        let rule = &live.saved.sort_rule;
        let rule_id = match rules.iter().position(|known| *known == rule) {
            Some(at) => at + 1,
            None => {
                rules.push(rule);
                rules.len()
            }
        };
        batches.push(label_batch(*batch, live, rule_id as u16, ids));
    }
    let root = Root {
        schema: SCHEMA,
        namespace: &fold.namespace,
        date: fold.date,
        journal_seq: fold.last_seq,
        input_pdfs: input_pdfs(fold),
        label_sort_rules: Numbered(
            rules
                .into_iter()
                .enumerate()
                .map(|(index, rule)| (index as u16 + 1, rule))
                .collect(),
        ),
        label_batches: batches,
        reprints: fold
            .reprints
            .iter()
            .map(|reprint| Reprint {
                number: reprint.made.number,
                output_path: output_path(&reprint.made.file),
                order_ids: reprint
                    .made
                    .orders
                    .iter()
                    .map(|order| &order.order_id)
                    .collect(),
            })
            .collect(),
    };
    toml::to_string(&root).expect("a view is plain TOML")
}

fn output_path(file: &str) -> String {
    format!("output/{file}")
}

/// Each input of a live batch, once, with what its written orders and findings say.
fn input_pdfs(fold: &Fold) -> Numbered<LabelPdf<'_>> {
    let mut inputs: BTreeMap<PdfId, &InputPdf> = BTreeMap::new();
    for live in fold.live.values() {
        for input in &live.saved.inputs {
            inputs.entry(input.pdf).or_insert(input);
        }
    }
    let lives: Vec<&LiveBatch> = fold.live.values().collect();
    let entries = inputs
        .into_iter()
        .map(|(pdf, input)| {
            let orders: Vec<&SavedOrder> = lives
                .iter()
                .flat_map(|live| &live.saved.orders)
                .filter(|order| order.pdf == pdf)
                .collect();
            let names = |findings: fn(&LiveBatch) -> &Vec<Finding>| -> Vec<&str> {
                lives
                    .iter()
                    .flat_map(|live| findings(live))
                    .filter(|finding| finding.pdf == Some(pdf))
                    .map(|finding| finding.text.as_str())
                    .collect()
            };
            let (earliest, latest) = ship_by_range(&orders);
            let entry = LabelPdf {
                filename: &input.filename,
                size: input.size,
                hash_sha256: &input.sha256,
                create_time: &input.create_time,
                modify_time: &input.modify_time,
                order_count: orders.len() as u16,
                page_count: input.page_count,
                earliest_ship_by_date: earliest,
                latest_ship_by_date: latest,
                couriers: orders.iter().filter_map(|o| o.courier.as_ref()).collect(),
                warnings: names(|live| &live.saved.warnings),
                errors: names(|live| &live.saved.errors),
            };
            (pdf.get() as u16, entry)
        })
        .collect();
    Numbered(entries)
}

fn label_batch<'a>(
    batch: BatchId,
    live: &'a LiveBatch,
    sort_rule_id: u16,
    ids: &ContentIds,
) -> LabelBatch<'a> {
    let saved = &live.saved;
    let orders: Vec<&SavedOrder> = saved.orders.iter().collect();
    let (earliest, latest) = ship_by_range(&orders);
    let result_groups = saved
        .outputs
        .iter()
        .filter_map(|output| {
            let code = output.group.as_deref()?;
            Some(LabelGroup {
                code,
                output_path: output_path(&output.file),
                is_marked_as_printed: live.printed.contains(code),
                order_contents: order_contents(batch, code, &orders, ids),
            })
        })
        .collect();
    LabelBatch {
        id: batch,
        create_time: &live.create_time,
        modify_time: &live.modify_time,
        input_pdfs: saved.inputs.iter().map(|input| input.pdf).collect(),
        sort_rule_id,
        result_groups,
        picking_list: picking_list(live),
        earliest_ship_by_date: earliest,
        latest_ship_by_date: latest,
        couriers: orders.iter().filter_map(|o| o.courier.as_ref()).collect(),
        warnings: saved.warnings.iter().map(|f| f.text.as_str()).collect(),
        errors: saved.errors.iter().map(|f| f.text.as_str()).collect(),
        left_out: &saved.left_out,
        order_count: orders.len() as u16,
        summary_pdf_path: saved
            .outputs
            .iter()
            .find(|output| output.group.is_none())
            .map(|output| output_path(&output.file)),
    }
}

/// A group's order contents in printing order, each with its number of orders.
fn order_contents(
    batch: BatchId,
    group: &str,
    orders: &[&SavedOrder],
    ids: &ContentIds,
) -> Vec<OrderContentSummary> {
    let mut summaries: Vec<OrderContentSummary> = Vec::new();
    for order in orders.iter().filter(|order| order.group == group) {
        let id = ids.id(batch, order.content);
        match summaries
            .iter_mut()
            .find(|summary| summary.content_id == id)
        {
            Some(summary) => summary.order_count += 1,
            None => summaries.push(OrderContentSummary {
                content_id: id,
                order_count: 1,
            }),
        }
    }
    summaries
}

/// The units of each item over the batch, sorted by display name as the summary PDF prints them.
fn picking_list(live: &LiveBatch) -> Vec<ItemPick> {
    let mut units: BTreeMap<(String, String), u16> = BTreeMap::new();
    for order in &live.saved.orders {
        let content = &live.saved.contents[order.content as usize];
        for item in &content.items {
            let total = units
                .entry((item.name.clone(), item.variant.clone()))
                .or_default();
            *total = total.saturating_add(item.quantity);
        }
    }
    let mut list: Vec<ItemPick> = units
        .into_iter()
        .map(|((name, variant), quantity)| ItemPick {
            name,
            variant,
            quantity,
        })
        .collect();
    list.sort_by_cached_key(|item| {
        (
            display_name(&item.name, &item.variant),
            item.name.clone(),
            item.variant.clone(),
        )
    });
    list
}

fn ship_by_range(orders: &[&SavedOrder]) -> (Option<Date>, Option<Date>) {
    let dates = orders.iter().filter_map(|order| order.ship_by_date);
    (dates.clone().min(), dates.max())
}
