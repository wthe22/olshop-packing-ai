//! Search commands (08 › *Search*, 2.21): find a saved order of the day, or of every day, by the
//! condition boxes of the Categories editor. The matching is the engine's (`packing_engine::
//! search`); this module reads the day folders and turns a row's saved PDF into a path the window
//! can open.
//!
//! Search only reads `state.json`: nothing is changed or saved, and an order of an open draft
//! (not yet saved) is never found (07 › *`state.json`*).

use std::path::Path;

use packing_engine::day::DayState;
use packing_engine::names;
use packing_engine::rules::parse_condition;
use packing_engine::rules_edit::{self, Tree};
use packing_engine::search::{self, SearchResults};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::commands::day::days_in;
use crate::{AppState, LABELS_DIR};

/// Which day folders to search ([Q-UI-3]): the day in the title bar (*this day*, the default), or
/// every day folder of the data folder (*all days*).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchScope {
    Day,
    All,
}

/// One line of a result row: the product name and its quantity.
#[derive(Debug, Serialize)]
pub struct SearchItemView {
    name: String,
    quantity: i64,
}

/// One matching saved order as the Search panel's row shows it (08 › *Search*): its IDs, courier,
/// items, the batch it was saved in and its saved PDF (name, and the full path *Open* uses).
#[derive(Debug, Serialize)]
pub struct SearchRowView {
    /// The day folder the order was saved in (`YYYY-MM-DD`).
    day: String,
    order_id: String,
    tracking_id: String,
    /// The named courier; empty when it is unknown.
    courier: String,
    items: Vec<SearchItemView>,
    batch: i64,
    pdf: i64,
    /// The saved PDF's file name.
    file: String,
    /// The saved PDF's full path, for *Open* (`open_path`).
    path: String,
    /// The order has no stored items: the row says *items not stored*, and only an ID condition
    /// found it (2.21).
    items_not_stored: bool,
}

/// The Search panel's result (08 › *Search*): the counts shown above the rows, then one row per
/// matching saved order.
#[derive(Debug, Serialize)]
pub struct SearchResultsView {
    rows: Vec<SearchRowView>,
    orders: usize,
    batches: usize,
}

/// The days to search: this day's folder, or every day folder (newest first, as the Day switch
/// lists them).
fn days_to_search(state: &AppState, folder: &Path, scope: SearchScope) -> Vec<String> {
    match scope {
        SearchScope::Day => vec![state.day()],
        SearchScope::All => days_in(folder),
    }
}

fn search_orders_impl(
    state: &AppState,
    conditions: &Tree,
    scope: SearchScope,
) -> Result<SearchResultsView, String> {
    // The boxes are made by the engine's own parser/printer (08 › *5. Categories*), so the
    // condition language has one home.
    let text = rules_edit::tree_to_condition(conditions).map_err(|error| error.to_string())?;
    let condition = parse_condition(&text).map_err(|error| error.to_string())?;

    let folder = state.data_folder()?;
    let labels = folder.join(LABELS_DIR);
    let mut days: Vec<(String, DayState)> = Vec::new();
    for day in days_to_search(state, &folder, scope) {
        let day_dir = labels.join(&day);
        let day_state = DayState::load(&day_dir).map_err(|error| error.to_string())?;
        days.push((day, day_state));
    }
    let refs: Vec<(&str, &DayState)> = days
        .iter()
        .map(|(day, state)| (day.as_str(), state))
        .collect();
    let results = search::search(&condition, &refs);
    Ok(search_results_view(&results, &labels))
}

fn search_results_view(results: &SearchResults, labels: &Path) -> SearchResultsView {
    SearchResultsView {
        orders: results.orders,
        batches: results.batches,
        rows: results
            .rows
            .iter()
            .map(|row| SearchRowView {
                day: row.day.clone(),
                order_id: row.order_id.clone(),
                tracking_id: row.tracking_id.clone(),
                courier: row.courier.clone(),
                items: row
                    .items
                    .iter()
                    .map(|item| SearchItemView {
                        name: item.name.clone(),
                        quantity: item.quantity,
                    })
                    .collect(),
                batch: row.batch,
                pdf: row.pdf,
                file: row.file.clone(),
                path: labels
                    .join(&row.day)
                    .join(names::batch_folder(row.batch))
                    .join(&row.file)
                    .display()
                    .to_string(),
                items_not_stored: row.items_not_stored,
            })
            .collect(),
    }
}

/// Search the saved orders of the day (or of every day) for the given condition boxes (08 ›
/// *Search*). Reads `state.json` only; *Open* on a row uses `open_path` with the row's `path`.
#[tauri::command]
pub fn search_orders(
    state: State<'_, AppState>,
    conditions: Tree,
    scope: SearchScope,
) -> Result<SearchResultsView, String> {
    search_orders_impl(&state, &conditions, scope)
}

#[cfg(test)]
mod tests;
