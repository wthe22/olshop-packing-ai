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
mod tests {
    use super::*;
    use packing_engine::day::{BatchEntry, PackingListChoice, PdfEntry, SavedItem, SavedOrder};
    use packing_engine::plan::Plan;
    use packing_engine::rules_edit::condition_to_tree;
    use packing_pdf::batch::Draft;

    use crate::OpenDraft;

    fn temp_dir(label: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("packing-app-search-{}-{label}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp dir");
        dir
    }

    fn batch_entry(batch: i64, pdf_file: &str) -> BatchEntry {
        BatchEntry {
            batch,
            save_time: "2026-10-07T07:40:12+07:00".to_string(),
            files: Vec::new(),
            pdfs: vec![PdfEntry {
                number: 1,
                code: "A".to_string(),
                name: "Sepatu".to_string(),
                file: pdf_file.to_string(),
                orders: 1,
                runs: 1,
            }],
            packing_list: PackingListChoice::default(),
            printed: Vec::new(),
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// One saved order with the searched fields written (as a save does, 07 › *`state.json`*).
    fn saved_order(order_id: &str, tracking_id: &str, name: &str, quantity: i64) -> SavedOrder {
        SavedOrder {
            order_id: order_id.to_string(),
            tracking_id: tracking_id.to_string(),
            batch: 1,
            pdf: 1,
            run: 1,
            items: vec![SavedItem {
                name: name.to_string(),
                variation: "42".to_string(),
                seller_sku: "SP-0001".to_string(),
                quantity,
            }],
            courier: "J&T Express".to_string(),
            total_quantity: quantity,
            distinct_items: 1,
            ship_by: None,
        }
    }

    /// One saved order as an older day holds it: only its IDs, no searched fields.
    fn legacy_order(order_id: &str, tracking_id: &str) -> SavedOrder {
        SavedOrder {
            order_id: order_id.to_string(),
            tracking_id: tracking_id.to_string(),
            batch: 1,
            pdf: 1,
            run: 1,
            items: Vec::new(),
            courier: String::new(),
            total_quantity: 0,
            distinct_items: 0,
            ship_by: None,
        }
    }

    fn write_day(folder: &Path, day: &str, saved: Vec<SavedOrder>) {
        let day_dir = folder.join(LABELS_DIR).join(day);
        let mut state = DayState::new(day);
        state.batches.push(batch_entry(1, "1 A Sepatu ×1.pdf"));
        state.saved = saved;
        state.save(&day_dir).expect("save state");
    }

    fn app_state(folder: &Path, day: &str) -> AppState {
        let state = AppState::new();
        state.set_data_folder(folder.to_path_buf());
        state.set_day(day);
        state
    }

    fn tree(text: &str) -> Tree {
        condition_to_tree(text).expect("boxes")
    }

    #[test]
    fn a_tracking_id_finds_its_one_order_with_the_saved_pdf_path() {
        let dir = temp_dir("tracking");
        write_day(
            &dir,
            "2026-10-07",
            vec![
                saved_order("580000000000000001", "JY0000001234", "Sepatu", 2),
                saved_order("580000000000000002", "JY0000005678", "Spion", 1),
            ],
        );
        let state = app_state(&dir, "2026-10-07");

        let view = search_orders_impl(
            &state,
            &tree("tracking_id equals \"JY0000001234\""),
            SearchScope::Day,
        )
        .expect("search");
        assert_eq!(view.orders, 1);
        assert_eq!(view.batches, 1);
        assert_eq!(view.rows.len(), 1);
        let row = &view.rows[0];
        assert_eq!(row.order_id, "580000000000000001");
        assert_eq!(row.courier, "J&T Express");
        assert_eq!(row.items.len(), 1);
        assert_eq!(row.items[0].name, "Sepatu");
        assert_eq!(row.items[0].quantity, 2);
        assert_eq!(row.file, "1 A Sepatu ×1.pdf");
        assert_eq!(row.batch, 1);
        assert_eq!(row.day, "2026-10-07");
        assert!(!row.items_not_stored);
        assert!(
            row.path
                .ends_with("labels\\2026-10-07\\batch 1\\1 A Sepatu ×1.pdf")
                || row
                    .path
                    .ends_with("labels/2026-10-07/batch 1/1 A Sepatu ×1.pdf"),
            "{}",
            row.path
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn this_day_does_not_see_another_days_order_but_all_days_does() {
        let dir = temp_dir("scope");
        write_day(
            &dir,
            "2026-10-07",
            vec![saved_order(
                "580000000000000001",
                "JY0000001234",
                "Sepatu",
                1,
            )],
        );
        write_day(
            &dir,
            "2026-10-06",
            vec![saved_order(
                "580000000000000009",
                "JY0000009999",
                "Spion",
                1,
            )],
        );
        let state = app_state(&dir, "2026-10-07");
        let cond = tree("tracking_id starts_with \"JY\"");

        let this_day = search_orders_impl(&state, &cond, SearchScope::Day).expect("day");
        assert_eq!(this_day.orders, 1);
        assert_eq!(this_day.rows[0].day, "2026-10-07");

        let all = search_orders_impl(&state, &cond, SearchScope::All).expect("all");
        assert_eq!(all.orders, 2);
        // Newest day first (the Day switch's order).
        assert_eq!(all.rows[0].day, "2026-10-07");
        assert_eq!(all.rows[1].day, "2026-10-06");
        assert_eq!(all.batches, 2, "two batches, one per day");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn an_order_of_an_open_draft_is_not_found() {
        let dir = temp_dir("draft");
        write_day(
            &dir,
            "2026-10-07",
            vec![saved_order(
                "580000000000000001",
                "JY0000001234",
                "Sepatu",
                1,
            )],
        );
        let state = app_state(&dir, "2026-10-07");
        // An open draft holding an order that was not saved: Search reads `saved[]` only.
        let order = packing_engine::orders::ReadOrder {
            order: packing_engine::orders::Order {
                order_id: "580000000000000777".to_string(),
                tracking_id: "JY0000000777".to_string(),
                courier: "JNE".to_string(),
                ship_by: None,
                lines: vec![packing_engine::orders::Line {
                    name: "Sepatu".to_string(),
                    variation: String::new(),
                    seller_sku: String::new(),
                    quantity: 1,
                    display_name: "Sepatu".to_string(),
                }],
                customer_message: String::new(),
            },
            pages: vec![packing_engine::orders::PageRef { file: 0, page: 0 }],
            position: 1,
            has_slip: true,
            qty_total: None,
        };
        state.set_draft(OpenDraft {
            draft: Draft {
                files: Vec::new(),
                orders: vec![order],
                duplicates: Vec::new(),
                warnings: Vec::new(),
                pages: 1,
                total_orders: 1,
                slip_pages: 1,
                unseen_logos: Vec::new(),
                read_warnings: Vec::new(),
            },
            plan: Plan::from_categories(&[]),
            first_number: 1,
            couriers: packing_engine::label::courier_data().clone(),
            amend: None,
            logo_pictures: Default::default(),
        });

        let view = search_orders_impl(
            &state,
            &tree("order_id contains \"580000000000000777\""),
            SearchScope::Day,
        )
        .expect("search");
        assert_eq!(view.orders, 0, "the draft's order is not saved");

        // The saved order is still found.
        let view = search_orders_impl(
            &state,
            &tree("order_id contains \"580000000000000001\""),
            SearchScope::Day,
        )
        .expect("search");
        assert_eq!(view.orders, 1);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn an_order_saved_before_search_is_found_by_its_ids_only() {
        let dir = temp_dir("legacy");
        write_day(
            &dir,
            "2026-10-07",
            vec![legacy_order("580000000000000001", "JY0000001234")],
        );
        let state = app_state(&dir, "2026-10-07");

        let by_tracking = search_orders_impl(
            &state,
            &tree("tracking_id equals \"JY0000001234\""),
            SearchScope::Day,
        )
        .expect("search");
        assert_eq!(by_tracking.orders, 1);
        assert!(by_tracking.rows[0].items_not_stored);
        assert!(by_tracking.rows[0].items.is_empty());

        let by_order = search_orders_impl(
            &state,
            &tree("order_id contains \"580000000000000001\""),
            SearchScope::Day,
        )
        .expect("search");
        assert_eq!(by_order.orders, 1, "Order ID also finds it");

        // A condition on an item or other stored field cannot see it.
        let by_name =
            search_orders_impl(&state, &tree("name contains \"sepatu\""), SearchScope::Day)
                .expect("search");
        assert_eq!(by_name.orders, 0);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_bad_condition_is_a_message() {
        let dir = temp_dir("bad");
        let state = app_state(&dir, "2026-10-07");
        // An empty root group is an error (08 › *5. Categories*): the panel shows it.
        let empty = Tree {
            root: packing_engine::rules_edit::Group {
                kind: packing_engine::rules_edit::GroupKind::All,
                not: false,
                children: Vec::new(),
            },
        };
        let error = search_orders_impl(&state, &empty, SearchScope::Day).expect_err("empty group");
        assert!(error.contains("empty"), "{error}");
        std::fs::remove_dir_all(&dir).ok();
    }
}
