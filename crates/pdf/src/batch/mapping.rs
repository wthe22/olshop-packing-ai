//! Mapping a plan into the packing jobs and the state entries of a batch: the saved PDF's pages
//! in run order, the packing-list sections and the orders as Search finds them. Plain functions,
//! so they are tested without a PDF library.

use std::collections::HashMap;

use packing_engine::day::{Layout, PackingListChoice, SavedItem, SavedOrder, Scope};
use packing_engine::ids::{BatchNumber, OrderId, PdfNumber};
use packing_engine::names;
use packing_engine::orders::{Line, ReadOrder};
use packing_engine::plan::{CategoryResult, PlanResult};

use crate::packing_list::{PackingLine, PackingList, RunOrder, RunRows, SavedPdfSection};

use super::error::{BatchError, missing};
use super::read::Draft;

/// One packing list to write: its file name, layout and built list.
pub struct PackingJob {
    pub(super) name: String,
    pub(super) layout: Layout,
    pub(super) list: PackingList,
}

/// The lists one batch asks for: the scope and layouts chosen, and the header texts every written
/// list carries.
pub(super) struct ListRequest<'a> {
    pub(super) packing: &'a PackingListChoice,
    pub(super) day: &'a str,
    pub(super) batch: Option<BatchNumber>,
    pub(super) printed: &'a str,
}

pub(super) fn packing_jobs(
    plan: &PlanResult,
    sections: &[SavedPdfSection],
    list: ListRequest<'_>,
) -> Vec<PackingJob> {
    let several = list.packing.layouts.len() > 1;
    let mut jobs = Vec::new();
    let mut make = |number: Option<PdfNumber>, section: Vec<SavedPdfSection>| {
        for layout in &list.packing.layouts {
            let suffix = several.then(|| layout.as_str());
            let name = names::packing_list_file_name(number, suffix);
            jobs.push(PackingJob {
                name,
                layout: *layout,
                list: PackingList {
                    day: list.day.to_string(),
                    batch: list.batch,
                    printed: list.printed.to_string(),
                    sections: section.clone(),
                },
            });
        }
    };
    match list.packing.scope {
        Scope::None => {}
        Scope::Whole => make(None, sections.to_vec()),
        Scope::PerPdf => {
            for (category, section) in plan.categories.iter().zip(sections) {
                make(Some(category.number), vec![section.clone()]);
            }
        }
    }
    jobs
}

/// A saved PDF's pages in run order.
pub(super) type OrderIndex<'a> = HashMap<&'a OrderId, &'a ReadOrder>;

pub(super) fn order_index(draft: &Draft) -> OrderIndex<'_> {
    draft
        .orders
        .iter()
        .map(|order| (&order.order.order_id, order))
        .collect()
}

pub(super) fn category_pages(
    category: &CategoryResult,
    index: &OrderIndex<'_>,
) -> Result<Vec<(usize, usize)>, BatchError> {
    let mut pages = Vec::new();
    for run in &category.runs {
        for id in &run.orders {
            let order = index.get(id).ok_or_else(|| missing(id))?;
            for page in &order.pages {
                pages.push((page.file, page.page));
            }
        }
    }
    Ok(pages)
}

fn packing_line(line: &Line) -> PackingLine {
    PackingLine {
        name: line.name.clone(),
        variation: line.variation.clone(),
        quantity: line.quantity,
        display_name: line.display_name.clone(),
    }
}

/// Map a pick and its orders into a packing-list section; the run structure is the pick's, so
/// the printed runs and the saved Pages match.
pub(super) fn category_section(
    category: &CategoryResult,
    index: &OrderIndex<'_>,
) -> Result<SavedPdfSection, BatchError> {
    let mut runs = Vec::with_capacity(category.runs.len());
    for run in &category.runs {
        let mut orders = Vec::with_capacity(run.orders.len());
        for id in &run.orders {
            let order = index.get(id).ok_or_else(|| missing(id))?;
            orders.push(RunOrder {
                tracking_id: order.order.tracking_id.clone(),
                lines: order.order.lines.iter().map(packing_line).collect(),
            });
        }
        runs.push(RunRows {
            number: run.number as u32,
            orders,
        });
    }
    Ok(SavedPdfSection {
        number: category.number,
        code: category.code.clone(),
        name: category.name.clone(),
        runs,
    })
}

pub(super) fn category_saved(
    category: &CategoryResult,
    index: &OrderIndex<'_>,
    batch: BatchNumber,
) -> Result<Vec<SavedOrder>, BatchError> {
    let mut entries = Vec::with_capacity(category.orders.len());
    for run in &category.runs {
        for id in &run.orders {
            let order = index.get(id).ok_or_else(|| missing(id))?;
            entries.push(SavedOrder {
                order_id: id.clone(),
                tracking_id: order.order.tracking_id.clone(),
                batch,
                pdf: category.number,
                run: run.number,
                // The fields the Search panel finds the order by, written at save: its lines
                // and the order-level fields.
                items: order
                    .order
                    .lines
                    .iter()
                    .map(|line| SavedItem {
                        name: line.name.clone(),
                        variation: line.variation.clone(),
                        seller_sku: line.seller_sku.clone(),
                        quantity: line.quantity,
                    })
                    .collect(),
                courier: order.order.courier.clone(),
                total_quantity: order.order.total_quantity(),
                distinct_items: order.order.distinct_items(),
                ship_by: order.order.ship_by.map(|time| time.to_string()),
            });
        }
    }
    Ok(entries)
}

/// `HH:MM` of an ISO save time (`2026-10-07T07:40:12+07:00` → `07:40`).
pub(super) fn printed_time(save_time: &str) -> String {
    save_time.chars().skip(11).take(5).collect()
}
