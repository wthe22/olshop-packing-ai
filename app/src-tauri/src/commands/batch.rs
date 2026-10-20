//! New batch and Plan/Save commands: the
//! `#[tauri::command]` functions the window calls and the read that opens a batch, over the
//! modules in the folder beside this file. The open draft lives in [`AppState`]; its label
//! documents stay open on the PDFium worker until the batch is saved or discarded.

use std::path::PathBuf;

use packing_engine::day::{DayState, PackingListChoice};
use packing_engine::groupings::Grouping;
use packing_engine::ids::BatchNumber;
use packing_engine::orders::LogoAnswer;
use packing_engine::rules::parse_condition;
use packing_pdf::batch;
use packing_pdf::worker::Progress;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::AppState;

mod amend;
mod logo;
mod plan_view;
mod read;
mod save;
mod views;

use self::amend::*;
use self::logo::*;
use self::plan_view::*;
use self::read::read_draft;
use self::save::*;
use self::views::*;

use self::views::ReadResult;

/// *Add files…* / a drop: the page count, size, time and `used in batch n` of each chosen file.
/// Needs only the page count, so it is quick.
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

/// *Read labels*: read on a blocking thread, emitting `read-progress {page,
/// pages}`; returns the draft summary and the plan, or the stop error text. A read replaces any
/// open draft, so the Plan screen's *Back* and a new *Read labels* start again from the files and
/// no plan edit survives. `grouping` is the grouping the new batch uses, chosen
/// on the selection page. `redo` is the batch being redone when the read comes from
/// the amend's selection page; `None` for a
/// new batch's file list. An amend reads with the grouping that batch recorded, whatever
/// `grouping` says.
#[tauri::command]
pub async fn read_labels(
    app: AppHandle,
    paths: Vec<String>,
    grouping: Grouping,
    redo: Option<BatchNumber>,
) -> Result<ReadResult, String> {
    let paths: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
    tauri::async_runtime::spawn_blocking(move || {
        let progress_app = app.clone();
        let state = app.state::<AppState>();
        let on_progress = move |progress: Progress| {
            let _ = progress_app.emit(
                "read-progress",
                ReadProgress {
                    page: progress.page,
                    pages: progress.pages,
                },
            );
        };
        match redo {
            Some(batch) => read_amend_draft(&state, paths, batch, on_progress),
            None => read_draft(&state, paths, &grouping, on_progress),
        }
    })
    .await
    .map_err(|error| format!("the reading task stopped: {error}"))?
}

/// *Cancel*: stop the running read; nothing is written.
#[tauri::command]
pub fn cancel_read(state: State<'_, AppState>) {
    state.request_cancel();
}

/// Answer the logo prompt about one unseen logo: applies the
/// answer to the open draft and returns the refreshed read result, so the screen shows the new
/// counts, warnings and remaining logos. *Skip all* is the screen's alone and calls nothing.
#[tauri::command]
pub fn answer_logo(
    state: State<'_, AppState>,
    hash: String,
    answer: LogoAnswer,
) -> Result<ReadResult, String> {
    answer_logo_impl(&state, &hash, &answer)
}

/// The live check of the *Add a pick* condition text:
/// the engine's parser message with line/col, or nothing when it is valid.
#[tauri::command]
pub fn check_condition(condition: String) -> Result<(), String> {
    parse_condition(&condition)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

/// Apply the plan edits and recount.
#[tauri::command]
pub fn plan(state: State<'_, AppState>, edits: Vec<PlanEdit>) -> Result<PlanView, String> {
    state.with_draft(|open| {
        apply_edits(&mut open.plan, &edits)?;
        let first_number = open.first_number;
        Ok(plan_view(
            &open.plan,
            &open.draft.orders,
            first_number,
            &open.grouping,
        ))
    })?
}

/// *Save batch*: write the batch and forget the draft. Runs on its own
/// thread, emitting `save-progress {stage, page, pages}` (the check before saving reports
/// *Writing* and *Checking*); the window shows the stage and a progress bar while it runs. The
/// check before saving runs on every save; `save_anyway` (the dialog's *Save anyway*) lets a batch
/// with findings be written and stores the two lists with it.
#[tauri::command]
pub async fn save_batch(
    app: AppHandle,
    packing: Option<PackingListChoice>,
    save_anyway: Option<bool>,
) -> Result<SaveResultView, String> {
    let save_anyway = save_anyway.unwrap_or(false);
    tauri::async_runtime::spawn_blocking(move || {
        let progress_app = app.clone();
        let state = app.state::<AppState>();
        save_batch_impl(&state, packing, save_anyway, move |progress| {
            let _ = progress_app.emit("save-progress", SaveProgressEvent::of(progress));
        })
    })
    .await
    .map_err(|error| format!("the save task stopped: {error}"))?
}

/// *Cancel* on the PDF selection page (a new batch's file list and the amend's selection page) and
/// an amend's *Cancel amend*: discard the open batch; nothing was
/// saved and nothing is written. The Plan screen's top *Back* does not come here — it keeps the
/// draft and shows the selection page again. For an amend this leaves the original batch
/// untouched.
#[tauri::command]
pub fn discard_draft(state: State<'_, AppState>) {
    state.clear_draft();
}

/// *Revert* the last batch: delete its folder and forget it. A file open in a PDF
/// viewer stops with the 08 *file in use* message and changes nothing.
#[tauri::command]
pub fn revert_last(state: State<'_, AppState>, day: String) -> Result<(), String> {
    revert_last_impl(&state, &day)
}

/// *Amend* the last batch: read the batch's own `download/` files and
/// open the *Redo batch `<b>`* Plan screen with them, the draft open; the batch's file list comes
/// back for the *Back* selection page. Runs
/// on a blocking thread, emitting `read-progress {page, pages}` as a new batch's read does.
#[tauri::command]
pub async fn start_amend(app: AppHandle, day: String) -> Result<AmendStart, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let progress_app = app.clone();
        let state = app.state::<AppState>();
        start_amend_impl(&state, &day, move |progress: Progress| {
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
    .map_err(|error| format!("the amend task stopped: {error}"))?
}

#[cfg(test)]
mod tests;
