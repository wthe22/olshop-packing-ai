//! The Day screen (05-after-save.md): the open workspace's date from its fold, and what changes it —
//! the Printed tick, revert and restore — each one append through `packing-batch`; the day
//! summary, a view written from the fold; and *Open* on an output, which writes a missing one again.

mod view;

use std::path::PathBuf;

use jiff::civil::Date;
use packing_batch::{RestoreOutcome, RestoreRequest, restore};
use packing_engine::ids::BatchId;
use packing_engine::record::PrintedMarked;
use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use crate::commands::start::open_with_explorer;
use crate::{APP_VERSION, AppState, settings};

pub use self::view::{BatchView, DayOverview, GroupPdfView, Totals};

pub(crate) fn date_of(day: &str) -> Result<Date, String> {
    day.parse()
        .map_err(|_| format!("{day} is not a date (YYYY-MM-DD)."))
}

/// Open `day` of the workspace (folded and checked unless it is open) and describe it.
pub(crate) fn overview(state: &AppState, day: &str) -> Result<DayOverview, String> {
    let date = date_of(day)?;
    state.with_data(|data| {
        let old_data = view::holds_v01_data(&data.dir);
        let open = data.open_day(date)?;
        Ok(view::overview(open, old_data))
    })
}

pub(crate) fn mark_printed(state: &AppState, day: &str, mark: PrintedMarked) -> Result<(), String> {
    let date = date_of(day)?;
    state.with_data(|data| {
        let open = data.open_day(date)?;
        packing_batch::set_printed(&open.folder, &mut open.journal, mark, APP_VERSION)
            .map_err(|error| error.to_string())
    })
}

/// Write the date's day summary from its journal and return its path.
pub(crate) fn write_day_summary(state: &AppState, day: &str) -> Result<String, String> {
    let date = date_of(day)?;
    let (_, workflows) = settings::workflows(state.program())?;
    state.with_data(|data| {
        let open = data.open_day(date)?;
        let path = state
            .with_worker(|worker| {
                packing_batch::day_summary(
                    worker,
                    &open.folder,
                    open.journal.fold(),
                    &workflows.couriers,
                )
            })?
            .map_err(|error| error.to_string())?;
        Ok(path.display().to_string())
    })
}

pub(crate) fn revert(state: &AppState, day: &str) -> Result<BatchId, String> {
    let date = date_of(day)?;
    state.with_data(|data| {
        let open = data.open_day(date)?;
        let batch = packing_batch::revert(&open.folder, &mut open.journal, APP_VERSION)
            .map_err(|error| error.to_string())?;
        open.outputs.retain(|problem| problem.batch != batch);
        state.index().refresh(open.journal.fold());
        Ok(batch)
    })
}

/// The verifier's findings of a restore; none when the file was written again.
#[derive(Serialize)]
pub struct RestoreView {
    restored: bool,
    errors: Vec<String>,
}

pub(crate) fn restore_file(state: &AppState, day: &str, file: &str) -> Result<RestoreView, String> {
    let date = date_of(day)?;
    let (_, workflows) = settings::workflows(state.program())?;
    state.with_data(|data| {
        let open = data.open_day(date)?;
        let outcome = state
            .with_worker(|worker| {
                restore(
                    &RestoreRequest {
                        worker,
                        folder: &open.folder,
                        file,
                        app: APP_VERSION,
                        couriers: &workflows.couriers,
                    },
                    &mut open.journal,
                )
            })?
            .map_err(|error| error.to_string())?;
        Ok(match outcome {
            RestoreOutcome::Restored(_) => {
                open.outputs.retain(|problem| problem.file != file);
                RestoreView {
                    restored: true,
                    errors: Vec::new(),
                }
            }
            RestoreOutcome::Findings(findings) => RestoreView {
                restored: false,
                errors: findings
                    .errors
                    .into_iter()
                    .chain(findings.warnings)
                    .collect(),
            },
        })
    })
}

/// What *Open* on a group PDF or summary PDF did.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OpenView {
    Opened,
    /// It was not there: written again first, then opened.
    WrittenAgain,
    /// It was not there, and the file written again did not pass the check: nothing was kept.
    NotKept {
        errors: Vec<String>,
    },
}

/// A group PDF or summary PDF of `day` made ready to open, and its path: one that is not there
/// any more is written again and checked, as **Restore** does, so an old batch's summary PDF comes
/// back in today's layout.
pub(crate) fn ready_to_open(
    state: &AppState,
    day: &str,
    file: &str,
) -> Result<(PathBuf, OpenView), String> {
    if file.contains(['/', '\\']) {
        return Err(format!("{file} is not a file name."));
    }
    let date = date_of(day)?;
    let path =
        state.with_data(|data| Ok(data.open_day(date)?.folder.dir().join("output").join(file)))?;
    if path.is_file() {
        return Ok((path, OpenView::Opened));
    }
    let restored = restore_file(state, day, file)?;
    let view = if restored.restored {
        OpenView::WrittenAgain
    } else {
        OpenView::NotKept {
            errors: restored.errors,
        }
    };
    Ok((path, view))
}

#[tauri::command]
pub fn day_overview(state: State<'_, AppState>, day: String) -> Result<DayOverview, String> {
    overview(&state, &day)
}

/// The **Printed** tick of a group PDF: a note only.
#[tauri::command]
pub fn set_printed(
    state: State<'_, AppState>,
    day: String,
    batch: BatchId,
    group: String,
    printed: bool,
) -> Result<(), String> {
    let mark = PrintedMarked {
        batch,
        group,
        printed,
    };
    mark_printed(&state, &day, mark)
}

/// *Revert* the newest batch. A file open in a PDF viewer stops it with the file named, and the
/// date is as before.
#[tauri::command]
pub fn revert_last(state: State<'_, AppState>, day: String) -> Result<(), String> {
    revert(&state, &day).map(|_| ())
}

/// **Day summary**: the date's summary PDF, written on its own thread; its path comes back.
#[tauri::command]
pub async fn day_summary(app: AppHandle, day: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        write_day_summary(&state, &day)
    })
    .await
    .map_err(|error| format!("the day summary task stopped: {error}"))?
}

/// *Open* a group PDF or summary PDF in the PC's default PDF viewer, on its own thread: one that is
/// not there any more is written again first.
#[tauri::command]
pub async fn open_output(app: AppHandle, day: String, file: String) -> Result<OpenView, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let (path, view) = ready_to_open(&state, &day, &file)?;
        if !matches!(view, OpenView::NotKept { .. }) {
            open_with_explorer(&path.display().to_string())?;
        }
        Ok(view)
    })
    .await
    .map_err(|error| format!("the open task stopped: {error}"))?
}

/// **Restore** a group PDF or summary PDF found missing or changed, on its own thread.
#[tauri::command]
pub async fn restore_output(
    app: AppHandle,
    day: String,
    file: String,
) -> Result<RestoreView, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        restore_file(&state, &day, &file)
    })
    .await
    .map_err(|error| format!("the restore task stopped: {error}"))?
}
