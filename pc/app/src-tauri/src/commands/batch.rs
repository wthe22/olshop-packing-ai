//! New batch and Plan/Save commands (08 › *2. New batch*, *3. Plan*, *4. Messages*): check the
//! chosen files, read the labels with progress and a cancel flag, edit the plan and save the
//! batch. The open draft lives in [`AppState`]; its label documents stay open on the PDFium
//! worker until the batch is saved or discarded (07 › *Commands between window and Rust*).
//!
//! The plan view and the plan edits are plain functions, so they are tested on orders built in
//! code (no PDFium needed); the read/save path is tested on `testdata/` and skipped when
//! `pc/vendor/pdfium.dll` is missing.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, mpsc};
use std::thread;

use packing_engine::day::{DayState, PackingListChoice};
use packing_engine::orders::{Order, ReadOrder, display_name};
use packing_engine::plan::{PickResult, Plan};
use packing_engine::rules::{Category, Condition, evaluate, load_rules, parse_condition};
use packing_pdf::batch::{self, FileCheck};
use packing_pdf::worker::Progress;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::{AppState, CATEGORIES_FILE, OpenDraft, commands::day::settings_load};

// ------------------------------------------------------------------------- new batch

/// One chosen label file as the New batch screen shows it (08 › *2. New batch*): page count,
/// size, download time and `used in batch n`.
#[derive(Serialize)]
pub struct FileCheckView {
    path: String,
    name: String,
    size: u64,
    /// `HH:MM` of the file's modified time (its download time).
    time: String,
    pages: usize,
    used_in_batch: Option<i64>,
}

/// The `read-progress` event payload (07 › *Commands between window and Rust*).
#[derive(Clone, Serialize)]
pub struct ReadProgress {
    page: usize,
    pages: usize,
}

/// What the New batch screen needs when reading ends (08 › *Reading*): the counts and warnings,
/// and the initial plan view for the Plan screen.
#[derive(Serialize)]
pub struct ReadResult {
    files: usize,
    pages: usize,
    orders: usize,
    /// Orders read before the duplicate guard.
    total_orders: usize,
    already_saved: usize,
    warnings: Vec<String>,
    plan: PlanView,
}

/// The Plan screen's data (08 › *3. Plan*).
#[derive(Serialize)]
pub struct PlanView {
    rows: Vec<PlanRow>,
    /// Two saved PDFs of the batch share a name: *Save batch* waits (08 › *3. Plan*).
    has_duplicate_names: bool,
    /// Where *+ Add a pick for this batch* inserts, above the rest row (08 › *3. Plan*).
    add_at: usize,
}

/// One row of the Plan screen (08 › *3. Plan*).
#[derive(Serialize)]
pub struct PlanRow {
    /// The plan entry's index, for the edits; `None` for the generated `? Uncategorised` row.
    index: Option<usize>,
    /// The saved-PDF number this row will get; `None` when it takes no orders.
    number: Option<i64>,
    code: String,
    name: String,
    /// The condition text as written (under the row); `None` when the row has no condition.
    condition: Option<String>,
    orders: usize,
    runs: Vec<RunView>,
    pages: usize,
    file_name: String,
    /// Skipped for this batch only (08 › *3. Plan*: *Use* unticked).
    skipped: bool,
    /// The generated rest row (not a plan entry): no *Use* tick and no moving.
    generated: bool,
    /// The rest row: the last entry when it has no condition, else `? Uncategorised`.
    is_rest: bool,
    /// The `#` of the earlier row with the same file name (*Same name as PDF 6*).
    duplicate_of: Option<i64>,
    can_use: bool,
    can_move_up: bool,
    can_move_down: bool,
    /// Takes 0 orders: shown grey with *0 orders — no PDF* and no number.
    empty: bool,
}

/// One run inside a saved PDF, as the expanded row shows it (08 › *3. Plan*):
/// `6-01  Sepatu Standar Samping Motor ×1   96 orders`.
#[derive(Serialize)]
pub struct RunView {
    label: String,
    contents: String,
    orders: usize,
}

/// One plan change from the window (08 › *3. Plan*). Applied to the open plan in order.
#[derive(Deserialize, Debug)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PlanEdit {
    /// Tick/untick *Use*: skip the pick for this batch only.
    SetSkipped {
        index: usize,
        skipped: bool,
    },
    MoveUp {
        index: usize,
    },
    MoveDown {
        index: usize,
    },
    /// *+ Add a pick for this batch*: a one-off pick at `at` (above the rest row).
    Add {
        at: usize,
        code: String,
        name: String,
        condition: String,
    },
    /// The whole saved-PDF name (07 › *Plan*: number included).
    Rename {
        index: usize,
        name: String,
    },
}

/// What *Save batch* wrote, for the green line on the Day screen (08 › *1. Day*).
#[derive(Serialize)]
pub struct SaveResultView {
    batch: i64,
    pdfs: usize,
    packing_lists: usize,
}

// ------------------------------------------------------------------------- the plan view

/// Map the plan and the draft's orders into the Plan screen's rows (08 › *3. Plan*). Every entry
/// becomes a row (skipped and 0-order rows included); the leftovers, if any, become the final
/// `? Uncategorised` row. The numbers, runs and names come from [`Plan::count`] so the screen and
/// the saved PDFs agree.
fn plan_view(plan: &Plan, orders: &[ReadOrder], first_number: i64) -> PlanView {
    let result = plan.count(orders, first_number);
    let entries = plan.entries();
    let index = order_index(orders);

    // The last entry when it has no condition is the rest row and stays last (08 › *3. Plan*).
    let rest_entry = match entries.last() {
        Some(last) if last.category.condition.is_none() => Some(entries.len() - 1),
        _ => None,
    };
    let add_at = rest_entry.unwrap_or(entries.len());

    let mut remaining: Vec<&ReadOrder> = orders.iter().collect();
    let mut picks = result.picks.iter();
    let mut rows: Vec<PlanRow> = Vec::with_capacity(entries.len() + 1);

    for (position, entry) in entries.iter().enumerate() {
        let is_rest = rest_entry == Some(position);
        let base =
            |number, orders: usize, runs, pages, file_name: String, empty, skipped| PlanRow {
                index: Some(position),
                number,
                code: entry.category.code.clone(),
                name: entry.category.name.clone(),
                condition: entry.category.when.clone(),
                orders,
                runs,
                pages,
                file_name,
                skipped,
                generated: false,
                is_rest,
                duplicate_of: None,
                can_use: true,
                can_move_up: position > 0 && !is_rest,
                can_move_down: position + 1 < entries.len()
                    && rest_entry != Some(position + 1)
                    && !is_rest,
                empty,
            };
        if entry.skipped {
            rows.push(base(None, 0, Vec::new(), 0, String::new(), false, true));
            continue;
        }
        let (taken, left) = split(&remaining, entry.category.condition.as_ref());
        remaining = left;
        if taken.is_empty() {
            rows.push(base(None, 0, Vec::new(), 0, String::new(), true, false));
        } else {
            let pick = picks.next().expect("a pick for every non-empty entry");
            rows.push(base(
                Some(pick.number),
                pick.orders.len(),
                run_views(pick, &index),
                pick.pages,
                pick.file_name.clone(),
                false,
                false,
            ));
        }
    }

    // The leftovers when no entry took them become `? Uncategorised` (08 › *3. Plan*).
    if !remaining.is_empty() {
        let pick = picks.next().expect("the rest pick");
        rows.push(PlanRow {
            index: None,
            number: Some(pick.number),
            code: pick.code.clone(),
            name: pick.name.clone(),
            condition: None,
            orders: pick.orders.len(),
            runs: run_views(pick, &index),
            pages: pick.pages,
            file_name: pick.file_name.clone(),
            skipped: false,
            generated: true,
            is_rest: true,
            duplicate_of: None,
            can_use: false,
            can_move_up: false,
            can_move_down: false,
            empty: false,
        });
    }

    // Duplicate saved-PDF names, case-insensitive as Windows is (08 › *3. Plan*).
    let mut seen: Vec<(String, i64)> = Vec::new();
    let mut has_duplicate_names = false;
    for row in &mut rows {
        let Some(number) = row.number else {
            continue;
        };
        let key = row.file_name.to_lowercase();
        if let Some((_, first)) = seen.iter().find(|(name, _)| *name == key) {
            row.duplicate_of = Some(*first);
            has_duplicate_names = true;
        } else {
            seen.push((key, number));
        }
    }

    PlanView {
        rows,
        has_duplicate_names,
        add_at,
    }
}

fn order_index(orders: &[ReadOrder]) -> HashMap<&str, &ReadOrder> {
    orders
        .iter()
        .map(|order| (order.order.order_id.as_str(), order))
        .collect()
}

/// Split the orders into the ones a condition takes and the ones left; `None` takes every order
/// (the rest entry). Mirrors the engine's `count` so the row counts match the saved PDFs.
fn split<'a>(
    orders: &[&'a ReadOrder],
    condition: Option<&Condition>,
) -> (Vec<&'a ReadOrder>, Vec<&'a ReadOrder>) {
    let mut taken = Vec::new();
    let mut left = Vec::new();
    for order in orders {
        let matches = match condition {
            None => true,
            Some(cond) => evaluate(cond, &order.order),
        };
        if matches {
            taken.push(*order);
        } else {
            left.push(*order);
        }
    }
    (taken, left)
}

/// A pick's runs as the expanded row shows them; the label is `<pick number>-<run number>`.
fn run_views(pick: &PickResult, index: &HashMap<&str, &ReadOrder>) -> Vec<RunView> {
    pick.runs
        .iter()
        .map(|run| RunView {
            label: format!("{}-{:02}", pick.number, run.number),
            contents: run
                .orders
                .first()
                .and_then(|id| index.get(id.as_str()))
                .map(|order| run_contents(&order.order))
                .unwrap_or_default(),
            orders: run.orders.len(),
        })
        .collect()
}

/// The identical contents of a run, as text: each line `display name ×quantity`.
fn run_contents(order: &Order) -> String {
    order
        .lines
        .iter()
        .map(|line| {
            format!(
                "{} ×{}",
                display_name(&line.name, &line.variation),
                line.quantity
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

// ------------------------------------------------------------------------- the plan edits

/// Apply the window's edits to the open plan in order (08 › *3. Plan*). A bad condition text or a
/// bad file name stops with the engine's message (line/col for a condition).
fn apply_edits(plan: &mut Plan, edits: &[PlanEdit]) -> Result<(), String> {
    for edit in edits {
        match edit {
            PlanEdit::SetSkipped { index, skipped } => plan.set_skipped(*index, *skipped),
            PlanEdit::MoveUp { index } => plan.move_up(*index),
            PlanEdit::MoveDown { index } => plan.move_down(*index),
            PlanEdit::Add {
                at,
                code,
                name,
                condition,
            } => {
                let code = code.trim();
                if code.is_empty() {
                    return Err("the pick code must not be empty".to_string());
                }
                if name.trim().is_empty() {
                    return Err("the pick name must not be empty".to_string());
                }
                let condition = condition.trim();
                let node = parse_condition(condition).map_err(|error| error.to_string())?;
                let category = Category {
                    code: code.to_string(),
                    name: name.trim().to_string(),
                    condition: Some(node),
                    when: Some(condition.to_string()),
                };
                plan.add(*at, category)
            }
            PlanEdit::Rename { index, name } => plan.rename(*index, name),
        }
        .map_err(|error| error.to_string())?;
    }
    Ok(())
}

// ------------------------------------------------------------------------------ reading

/// Read the chosen files into a draft and store it (08 › *2. New batch* → *3. Plan*). Runs on a
/// blocking thread; `on_progress` is called for every page (the window turns it into a
/// `read-progress` event). A stop (no packing slip, every order already saved, a rules error,
/// cancelled) returns its message and writes nothing.
fn read_draft(
    state: &AppState,
    paths: Vec<PathBuf>,
    on_progress: impl Fn(Progress) + Send + 'static,
) -> Result<ReadResult, String> {
    if paths.is_empty() {
        return Err("Choose at least one label file first.".to_string());
    }
    let folder = state.data_folder()?;
    let day_dir = state.day_dir()?;
    let categories = load_categories(&folder)?;
    let day_state = DayState::load(&day_dir).map_err(|error| error.to_string())?;

    // Files are read in file-name order (= download time and part number; 08 › *2. New batch*).
    let mut files = paths;
    files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    let cancel = Arc::new(AtomicBool::new(false));
    state.set_cancel(cancel.clone());

    let (progress_tx, progress_rx) = mpsc::channel::<Progress>();
    let forwarder = thread::spawn(move || {
        while let Ok(progress) = progress_rx.recv() {
            on_progress(progress);
        }
    });

    let inner = state.with_worker(|worker| {
        batch::check_files(worker, &files, &day_state).and_then(|checks| {
            batch::read(worker, &checks, &day_state, cancel.clone(), progress_tx)
        })
    });

    state.clear_cancel();
    let _ = forwarder.join(); // the sender is dropped when `with_worker` returns

    let draft = inner
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())?;
    let plan = Plan::from_categories(&categories);
    let first_number = day_state.next_pdf_number();
    let read = ReadResult {
        files: draft.files.len(),
        pages: draft.pages,
        orders: draft.orders.len(),
        total_orders: draft.total_orders,
        already_saved: draft.duplicates.len(),
        warnings: draft.warnings.clone(),
        plan: plan_view(&plan, &draft.orders, first_number),
    };
    state.set_draft(OpenDraft {
        draft,
        plan,
        first_number,
    });
    Ok(read)
}

/// Load `<data>/categories.toml` (08 › *Messages*: a rules error stops the batch with its
/// line/col).
fn load_categories(folder: &Path) -> Result<Vec<Category>, String> {
    let path = folder.join(CATEGORIES_FILE);
    let text = std::fs::read_to_string(&path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    load_rules(&text).map_err(|error| error.to_string())
}

// ------------------------------------------------------------------------------- saving

/// Apply the plan and save the batch (07 › *What happens in one batch* step 6), then forget the
/// draft. A failed save keeps the draft so *Save batch* can be pressed again.
fn save_batch_impl(
    state: &AppState,
    packing: Option<PackingListChoice>,
) -> Result<SaveResultView, String> {
    let folder = state.data_folder()?;
    let day = state.day();
    let day_dir = state.day_dir()?;
    let packing = match packing {
        Some(choice) => choice,
        None => settings_load(&folder)?.packing_choice(),
    };
    let open = state
        .take_draft()
        .ok_or_else(|| "No batch is open.".to_string())?;
    let mut day_state = DayState::load(&day_dir).map_err(|error| error.to_string())?;
    let plan = open.plan.count(&open.draft.orders, open.first_number);

    let result = state.with_worker(|worker| {
        batch::save(
            worker,
            &open.draft,
            &plan,
            &packing,
            &day,
            &day_dir,
            &mut day_state,
        )
    });

    match result {
        Ok(Ok(save)) => Ok(SaveResultView {
            batch: save.batch,
            pdfs: save.pdf_files.len(),
            packing_lists: save.packing_lists.len(),
        }),
        Ok(Err(error)) => {
            state.restore_draft(open);
            Err(error.to_string())
        }
        Err(message) => {
            state.restore_draft(open);
            Err(message)
        }
    }
}

// ----------------------------------------------------------------------------- helpers

fn file_check_view(check: FileCheck) -> FileCheckView {
    FileCheckView {
        path: check.path.display().to_string(),
        name: check.name,
        size: check.size,
        time: modified_time(&check.path),
        pages: check.pages,
        used_in_batch: check.used_in_batch,
    }
}

/// `HH:MM` of a file's modified time, or empty when it cannot be read.
fn modified_time(path: &Path) -> String {
    let Ok(metadata) = std::fs::metadata(path) else {
        return String::new();
    };
    let Ok(modified) = metadata.modified() else {
        return String::new();
    };
    let Ok(timestamp) = jiff::Timestamp::try_from(modified) else {
        return String::new();
    };
    timestamp
        .to_zoned(jiff::tz::TimeZone::system())
        .strftime("%H:%M")
        .to_string()
}

// ---------------------------------------------------------------------------- commands

/// *Add files…* / a drop: the page count, size, time and `used in batch n` of each chosen file
/// (08 › *2. New batch*). Needs only the page count, so it is quick.
#[tauri::command]
pub fn check_files(
    state: State<'_, AppState>,
    paths: Vec<String>,
) -> Result<Vec<FileCheckView>, String> {
    let paths: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
    let day_dir = state.day_dir()?;
    let day_state = DayState::load(&day_dir).map_err(|error| error.to_string())?;
    let inner = state.with_worker(|worker| batch::check_files(worker, &paths, &day_state))?;
    let checks = inner.map_err(|error| error.to_string())?;
    Ok(checks.into_iter().map(file_check_view).collect())
}

/// *Read labels* (08 › *Reading*): read on a blocking thread, emitting `read-progress {page,
/// pages}`; returns the draft summary and the plan, or the stop error text.
#[tauri::command]
pub async fn read_labels(app: AppHandle, paths: Vec<String>) -> Result<ReadResult, String> {
    let paths: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
    tauri::async_runtime::spawn_blocking(move || {
        let progress_app = app.clone();
        let state = app.state::<AppState>();
        read_draft(&state, paths, move |progress| {
            let _ = progress_app.emit(
                "read-progress",
                ReadProgress {
                    page: progress.page,
                    pages: progress.pages,
                },
            );
        })
    })
    .await
    .map_err(|error| format!("the reading task stopped: {error}"))?
}

/// *Cancel* (08 › *Reading*): stop the running read; nothing is written.
#[tauri::command]
pub fn cancel_read(state: State<'_, AppState>) {
    state.request_cancel();
}

/// The live check of the *Add a pick* condition text (08 › *3. Plan*; the boxes editor is 2.12):
/// the engine's parser message with line/col, or nothing when it is valid.
#[tauri::command]
pub fn check_condition(condition: String) -> Result<(), String> {
    parse_condition(&condition)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

/// Apply the plan edits and recount (08 › *3. Plan*: every change recounts all rows at once).
#[tauri::command]
pub fn plan(state: State<'_, AppState>, edits: Vec<PlanEdit>) -> Result<PlanView, String> {
    state.with_draft(|open| {
        apply_edits(&mut open.plan, &edits)?;
        let first_number = open.first_number;
        Ok(plan_view(&open.plan, &open.draft.orders, first_number))
    })?
}

/// *Save batch* (08 › *3. Plan* → *1. Day*): write the batch and forget the draft.
#[tauri::command]
pub fn save_batch(
    state: State<'_, AppState>,
    packing: Option<PackingListChoice>,
) -> Result<SaveResultView, String> {
    save_batch_impl(&state, packing)
}

/// *Cancel* on the Plan screen (08 › *3. Plan*): discard the open batch; nothing was saved.
#[tauri::command]
pub fn discard_draft(state: State<'_, AppState>) {
    state.clear_draft();
}

#[cfg(test)]
mod tests {
    use super::*;
    use packing_engine::orders::Line;
    use packing_engine::plan::REST_CODE;

    fn line(name: &str, variation: &str, quantity: i64) -> Line {
        Line {
            name: name.to_string(),
            variation: variation.to_string(),
            seller_sku: String::new(),
            quantity,
            display_name: display_name(name, variation),
        }
    }

    fn read_order(id: &str, tracking: &str, lines: Vec<Line>) -> ReadOrder {
        ReadOrder {
            order: Order {
                order_id: id.to_string(),
                tracking_id: tracking.to_string(),
                courier: "J&T Express".to_string(),
                ship_by: None,
                lines,
                customer_message: String::new(),
            },
            pages: vec![packing_engine::orders::PageRef { file: 0, page: 0 }],
            position: 1,
            has_slip: true,
            qty_total: None,
        }
    }

    fn category(code: &str, name: &str, when: Option<&str>) -> Category {
        Category {
            code: code.to_string(),
            name: name.to_string(),
            condition: when.map(|text| parse_condition(text).expect("condition")),
            when: when.map(str::to_string),
        }
    }

    /// Three sepatu, two spion, one leftover; two categories with conditions so the leftovers
    /// become the generated rest row.
    fn orders() -> Vec<ReadOrder> {
        vec![
            read_order("o1", "t1", vec![line("Sepatu A", "", 1)]),
            read_order("o2", "t2", vec![line("Sepatu B", "", 1)]),
            read_order("o3", "t3", vec![line("Sepatu C", "", 1)]),
            read_order("o4", "t4", vec![line("Spion X", "", 1)]),
            read_order("o5", "t5", vec![line("Spion Y", "", 1)]),
            read_order("o6", "t6", vec![line("Lain", "", 1)]),
        ]
    }

    fn sepatu_plan() -> Plan {
        Plan::from_categories(&[
            category("A", "Sepatu", Some("name contains \"sepatu\"")),
            category("B", "Spion", Some("name contains \"spion\"")),
        ])
    }

    #[test]
    fn plan_view_numbers_rows_and_adds_the_rest_row() {
        let view = plan_view(&sepatu_plan(), &orders(), 4);
        assert_eq!(view.rows.len(), 3);
        assert_eq!(view.add_at, 2); // after the two entries (no rest entry)

        let a = &view.rows[0];
        assert_eq!(a.number, Some(4));
        assert_eq!(a.code, "A");
        assert_eq!(a.orders, 3);
        assert_eq!(a.runs.len(), 3); // three different contents, one order each
        assert_eq!(a.runs[0].label, "4-01");
        assert_eq!(a.runs[0].orders, 1);
        assert_eq!(a.runs[0].contents, "Sepatu A ×1");
        assert_eq!(a.runs[2].label, "4-03");
        assert_eq!(a.file_name, "4 A Sepatu ×3.pdf");
        assert!(!a.empty && !a.is_rest && a.can_use);
        assert!(!a.can_move_up && a.can_move_down); // first row

        let b = &view.rows[1];
        assert_eq!(b.number, Some(5));
        assert_eq!(b.orders, 2);
        assert_eq!(b.condition.as_deref(), Some("name contains \"spion\""));

        let rest = &view.rows[2];
        assert!(rest.generated && rest.is_rest && !rest.can_use);
        assert_eq!(rest.index, None);
        assert_eq!(rest.number, Some(6));
        assert_eq!(rest.code, REST_CODE);
        assert_eq!(rest.name, "Uncategorised");
        assert_eq!(rest.orders, 1);
        assert!(!rest.can_move_up && !rest.can_move_down);

        assert!(!view.has_duplicate_names);
    }

    #[test]
    fn plan_view_shows_zero_order_and_skipped_rows() {
        let mut plan = sepatu_plan();
        // Untick the sepatu row: its three orders fall to the rest row.
        apply_edits(
            &mut plan,
            &[PlanEdit::SetSkipped {
                index: 0,
                skipped: true,
            }],
        )
        .expect("skip");
        let view = plan_view(&plan, &orders(), 1);

        assert!(view.rows[0].skipped);
        assert_eq!(view.rows[0].orders, 0);
        assert_eq!(view.rows[0].number, None);
        // Spion takes 2; the rest row takes the 3 sepatu + 1 lain.
        assert_eq!(view.rows[1].orders, 2);
        assert_eq!(view.rows[2].orders, 4);
        assert_eq!(view.rows[2].number, Some(2)); // the skipped row used no number

        // A condition that matches nothing shows a 0-order row with no number.
        let plan = Plan::from_categories(&[
            category("A", "Nope", Some("name contains \"zzz\"")),
            category("Z", "Lainnya", None),
        ]);
        let view = plan_view(&plan, &orders(), 1);
        assert!(view.rows[0].empty);
        assert_eq!(view.rows[0].number, None);
        assert_eq!(view.rows[1].number, Some(1)); // the rest entry catches everything
        assert!(view.rows[1].is_rest);
        assert_eq!(view.add_at, 1); // before the rest entry
    }

    #[test]
    fn plan_view_flags_duplicate_file_names() {
        let mut plan = sepatu_plan();
        apply_edits(
            &mut plan,
            &[
                PlanEdit::Rename {
                    index: 0,
                    name: "same.pdf".to_string(),
                },
                PlanEdit::Rename {
                    index: 1,
                    name: "Same.pdf".to_string(),
                },
            ],
        )
        .expect("rename");
        let view = plan_view(&plan, &orders(), 1);
        assert!(view.has_duplicate_names);
        assert_eq!(view.rows[0].duplicate_of, None);
        assert_eq!(view.rows[1].duplicate_of, Some(1)); // *Same name as PDF 1*
    }

    #[test]
    fn apply_edits_moves_adds_and_reports_condition_errors() {
        let mut plan = sepatu_plan();
        apply_edits(&mut plan, &[PlanEdit::MoveDown { index: 0 }]).expect("move");
        assert_eq!(plan.entries()[0].category.code, "B");
        apply_edits(&mut plan, &[PlanEdit::MoveUp { index: 1 }]).expect("move back");
        assert_eq!(plan.entries()[0].category.code, "A");

        apply_edits(
            &mut plan,
            &[PlanEdit::Add {
                at: 2,
                code: "X".to_string(),
                name: "One off".to_string(),
                condition: "total_quantity >= 2".to_string(),
            }],
        )
        .expect("add");
        assert_eq!(plan.len(), 3);
        assert_eq!(plan.entries()[2].category.code, "X");
        assert_eq!(
            plan.entries()[2].category.when.as_deref(),
            Some("total_quantity >= 2")
        );

        let error = apply_edits(
            &mut plan,
            &[PlanEdit::Add {
                at: 3,
                code: "Y".to_string(),
                name: "Bad".to_string(),
                condition: "name contains".to_string(),
            }],
        )
        .expect_err("bad condition");
        assert!(error.contains("line 1, col"), "{error}");

        let error = apply_edits(
            &mut plan,
            &[PlanEdit::Rename {
                index: 0,
                name: "   ".to_string(),
            }],
        )
        .expect_err("empty name");
        assert!(error.contains("empty"), "{error}");
    }

    #[test]
    fn check_condition_reports_the_engine_message() {
        assert!(check_condition("name contains \"x\"".to_string()).is_ok());
        let error = check_condition("name contains".to_string()).expect_err("bad");
        assert!(error.starts_with("line 1, col "), "{error}");
    }

    // ------------------------------------------------------------------ PDF-backed (testdata)

    /// The pinned `pdfium.dll` directory, or `None` when it has not been downloaded.
    fn vendor_dir() -> Option<PathBuf> {
        let vendor = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor");
        vendor.join("pdfium.dll").exists().then_some(vendor)
    }

    /// `testdata/` at the repository root.
    fn testdata(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../testdata")
            .join(name)
    }

    fn temp_dir(label: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("packing-app-batch-{}-{label}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp dir");
        dir
    }

    fn app_state(folder: &Path) -> AppState {
        let state = AppState::new();
        state.set_data_folder(folder.to_path_buf());
        state.set_day("2026-10-07");
        assert!(state.ensure_worker(), "run pc/tools/get-pdfium.sh");
        state
    }

    #[test]
    fn read_and_save_testdata_labels_slip() {
        let Some(_vendor) = vendor_dir() else {
            eprintln!("skipped: run pc/tools/get-pdfium.sh");
            return;
        };
        let dir = temp_dir("slip");
        std::fs::copy(testdata("categories.toml"), dir.join(CATEGORIES_FILE)).expect("copy rules");
        let state = app_state(&dir);

        let read = read_draft(&state, vec![testdata("labels-slip.pdf")], |_| {}).expect("read");
        assert_eq!(read.orders, 23);
        assert_eq!(read.total_orders, 23);
        assert_eq!(read.already_saved, 0);
        assert_eq!(read.pages, 24);
        let numbered: Vec<(i64, usize)> = read
            .plan
            .rows
            .iter()
            .filter_map(|row| row.number.map(|number| (number, row.orders)))
            .collect();
        assert_eq!(numbered, vec![(1, 7), (2, 8), (3, 4), (4, 2), (5, 2)]);

        let save = save_batch_impl(&state, None).expect("save");
        assert_eq!(save.batch, 1);
        assert_eq!(save.pdfs, 5);
        assert_eq!(save.packing_lists, 1);
        assert!(dir.join("labels/2026-10-07/state.json").is_file());

        // A second read of the same file: every order was already saved today.
        let again = read_draft(&state, vec![testdata("labels-slip.pdf")], |_| {})
            .err()
            .expect("already saved");
        assert!(again.contains("already saved today"), "{again}");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn read_plain_labels_stops_with_no_slip() {
        let Some(_vendor) = vendor_dir() else {
            eprintln!("skipped: run pc/tools/get-pdfium.sh");
            return;
        };
        let dir = temp_dir("plain");
        std::fs::copy(testdata("categories.toml"), dir.join(CATEGORIES_FILE)).expect("copy rules");
        let state = app_state(&dir);

        let error = read_draft(&state, vec![testdata("labels-plain.pdf")], |_| {})
            .err()
            .expect("no slip");
        assert!(error.contains("no packing slip"), "{error}");

        std::fs::remove_dir_all(&dir).ok();
    }
}
