//! Search: find a saved order by the condition boxes of the Categories editor (2.21,
//! [Search](../ui/search.md)). The engine takes an already-parsed condition and the day states to
//! search; the app reads the day folders (no PDF code, no paths here).
//!
//! Only `saved[]` is searched: a batch's orders are there once it is saved, so an order of an
//! open draft is never found (07 › *`state.json`*). An order saved before search existed keeps
//! only its IDs; its row says *items not stored*, and only a condition on `order_id` or
//! `tracking_id` can match it. Every other field is absent for such an order, so a condition on
//! one cannot match (2.21; Q-UI-3).

use crate::day::{DayState, SavedOrder};
use crate::ids::{BatchNumber, OrderId, PdfNumber, TrackingId};
use crate::orders::{Line, Order, display_name};
use crate::rules::{Condition, evaluate, fields_used};

/// One line of a result row, as the Search panel shows it: the product name and its quantity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchItem {
    pub name: String,
    pub quantity: i64,
}

/// One matching saved order: what the Search panel's row shows (2.21).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchRow {
    /// The day folder the order was saved in (`YYYY-MM-DD`).
    pub day: String,
    pub order_id: OrderId,
    pub tracking_id: Option<TrackingId>,
    /// The named courier; empty when it is unknown.
    pub courier: String,
    pub items: Vec<SearchItem>,
    pub batch: BatchNumber,
    pub pdf: PdfNumber,
    /// The saved PDF's file name (07 › *Day folder*); the app prepends the folder path.
    pub file: String,
    /// The order has no stored items: the row says *items not stored*.
    pub items_not_stored: bool,
}

/// The Search panel's result: one row per matching saved order and the counts shown above them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SearchResults {
    pub rows: Vec<SearchRow>,
    /// The matched orders.
    pub orders: usize,
    /// The distinct batches the matched orders sit in.
    pub batches: usize,
}

/// The fields every saved order keeps, whatever its age (07 › *`state.json`*): the only ones a
/// condition can match an order saved before search existed on.
const ID_FIELDS: &[&str] = &["order_id", "tracking_id"];

/// The order a [`SavedOrder`] describes, for the condition evaluator: its item fields read the
/// stored lines. An order saved before search existed has no lines, so only its IDs are real.
fn saved_order(saved: &SavedOrder) -> Order {
    Order {
        order_id: saved.order_id.clone(),
        tracking_id: saved.tracking_id.clone(),
        courier: saved.courier.clone(),
        ship_by: saved.ship_by.as_deref().and_then(|text| text.parse().ok()),
        lines: saved
            .items
            .iter()
            .map(|item| Line {
                name: item.name.clone(),
                variation: item.variation.clone(),
                seller_sku: item.seller_sku.clone(),
                quantity: item.quantity,
                display_name: display_name(&item.name, &item.variation),
            })
            .collect(),
        customer_message: String::new(),
    }
}

/// Whether a condition can match an order saved before search existed: every field it uses is one
/// every saved order keeps (07 › *`state.json`*; [Search](../ui/search.md): *items not stored*,
/// found by IDs only).
fn only_ids(condition: &Condition) -> bool {
    fields_used(condition)
        .iter()
        .all(|field| ID_FIELDS.contains(&field.as_str()))
}

/// Search the saved orders of `days` for `condition` (2.21). `days` are `(day folder name, state)`
/// in the order the rows are wanted; only `state.saved` is read, so an order of an open draft is
/// never found.
pub fn search(condition: &Condition, days: &[(&str, &DayState)]) -> SearchResults {
    let mut rows: Vec<SearchRow> = Vec::new();
    let mut batches: Vec<(String, BatchNumber)> = Vec::new();
    for (day, state) in days {
        for saved in &state.saved {
            let stored = !saved.items.is_empty();
            // An order without stored items keeps only its IDs: a condition on any other field
            // cannot match it.
            if !stored && !only_ids(condition) {
                continue;
            }
            if !evaluate(condition, &saved_order(saved)) {
                continue;
            }
            rows.push(SearchRow {
                day: (*day).to_string(),
                order_id: saved.order_id.clone(),
                tracking_id: saved.tracking_id.clone(),
                courier: saved.courier.clone(),
                items: saved
                    .items
                    .iter()
                    .map(|item| SearchItem {
                        name: item.name.clone(),
                        quantity: item.quantity,
                    })
                    .collect(),
                batch: saved.batch,
                pdf: saved.pdf,
                file: saved_pdf_file(state, saved),
                items_not_stored: !stored,
            });
            let key = ((*day).to_string(), saved.batch);
            if !batches.contains(&key) {
                batches.push(key);
            }
        }
    }
    SearchResults {
        orders: rows.len(),
        batches: batches.len(),
        rows,
    }
}

/// The saved PDF's file name (07 › *Day folder*), from the batch the order was saved in.
fn saved_pdf_file(state: &DayState, saved: &SavedOrder) -> String {
    state
        .batches
        .iter()
        .find(|batch| batch.batch == saved.batch)
        .and_then(|batch| {
            batch
                .pdfs
                .iter()
                .find(|pdf| pdf.number == saved.pdf)
                .map(|pdf| pdf.file.clone())
        })
        .unwrap_or_default()
}
