//! The Day screen (05-after-save.md): the open date from its fold, and what changes it —
//! the Sent mark, the Printed tick, revert and restore — each one append through `packing-batch`; the day
//! summary, a view written from the fold; and *Open* on an output, which writes a missing one again.

mod view;

use std::path::PathBuf;

use jiff::civil::Date;
use packing_batch::{RestoreOutcome, RestoreRequest, restore};
use packing_engine::ids::BatchId;
use packing_engine::record::{PrintSent, PrintedMarked};
use packing_store::{Fold, older_data};
use serde::Serialize;
use tauri::ipc::Response;
use tauri::{AppHandle, Manager, State};

use crate::commands::start::open_with_explorer;
use crate::{APP_VERSION, AppState, settings};

pub use self::view::{BatchView, DayOverview, GroupPdfView, Totals};

pub(crate) fn date_of(day: &str) -> Result<Date, String> {
    day.parse()
        .map_err(|_| format!("{day} is not a date (YYYY-MM-DD)."))
}

/// Open `day` (folded and checked unless it is open) and describe it.
pub(crate) fn overview(state: &AppState, day: &str) -> Result<DayOverview, String> {
    let date = date_of(day)?;
    state.with_data(|data| {
        let older = older_data(&data.dir).map_err(|error| error.to_string())?;
        let older = view::older_views(&data.dir, &older);
        let open = data.open_day(date)?;
        Ok(view::overview(open, older))
    })
}

/// `batch` must still be the date's newest: another copy of the app may have saved one since the
/// Day screen showed it.
pub(crate) fn check_newest(fold: &Fold, batch: BatchId) -> Result<(), String> {
    if fold.newest_live() == Some(batch) {
        return Ok(());
    }
    Err(format!(
        "Batch {batch} is no longer the newest batch of this date: another copy of the app changed the date. It now shows as it is."
    ))
}

pub(crate) fn mark_printed(state: &AppState, day: &str, mark: PrintedMarked) -> Result<(), String> {
    let date = date_of(day)?;
    state.write_day(date, |open| {
        packing_batch::set_printed(&open.folder, &mut open.journal, mark, APP_VERSION)
            .map_err(|error| error.to_string())
    })
}

pub(crate) fn mark_sent(state: &AppState, day: &str, mark: PrintSent) -> Result<(), String> {
    let date = date_of(day)?;
    state.write_day(date, |open| {
        packing_batch::set_sent(&open.folder, &mut open.journal, mark, APP_VERSION)
            .map_err(|error| error.to_string())
    })
}

/// Write the date's day summary from its journal and return its path.
pub(crate) fn write_day_summary(state: &AppState, day: &str) -> Result<String, String> {
    let date = date_of(day)?;
    let (_, workflows) = settings::workflows(state.program())?;
    state.write_day(date, |open| {
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

pub(crate) fn revert(state: &AppState, day: &str, batch: BatchId) -> Result<(), String> {
    let date = date_of(day)?;
    state.write_day(date, |open| {
        check_newest(open.journal.fold(), batch)?;
        packing_batch::revert(&open.folder, &mut open.journal, APP_VERSION)
            .map_err(|error| error.to_string())?;
        open.outputs.retain(|problem| problem.batch != batch);
        state.index().refresh(open.journal.fold());
        Ok(())
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
    state.write_day(date, |open| {
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

/// Where output `file` of `day` is; `file` must be a name only.
fn output_path(state: &AppState, day: &str, file: &str) -> Result<PathBuf, String> {
    if file.contains(['/', '\\']) {
        return Err(format!("{file} is not a file name."));
    }
    let date = date_of(day)?;
    state.with_data(|data| Ok(data.open_day(date)?.folder.dir().join("output").join(file)))
}

pub(crate) fn output_bytes(state: &AppState, day: &str, file: &str) -> Result<Vec<u8>, String> {
    let path = output_path(state, day, file)?;
    std::fs::read(&path).map_err(|error| format!("{} could not be read: {error}", path.display()))
}

/// A group PDF or summary PDF of `day` made ready to open, and its path: one that is not there
/// any more is written again and checked, as **Restore** does, so an old batch's summary PDF comes
/// back in today's layout.
pub(crate) fn ready_to_open(
    state: &AppState,
    day: &str,
    file: &str,
) -> Result<(PathBuf, OpenView), String> {
    let path = output_path(state, day, file)?;
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

/// The **Printed** tick of a group PDF: a note only. On its own thread, as each write: it may wait
/// for another copy of the app to finish writing.
#[tauri::command]
pub async fn set_printed(
    app: AppHandle,
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
    tauri::async_runtime::spawn_blocking(move || mark_printed(&app.state::<AppState>(), &day, mark))
        .await
        .map_err(|error| format!("the Printed task stopped: {error}"))?
}

/// *Revert* `batch`, which must still be the newest. A file open in a PDF viewer stops it with the
/// file named, and the date is as before.
#[tauri::command]
pub async fn revert_last(app: AppHandle, day: String, batch: BatchId) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || revert(&app.state::<AppState>(), &day, batch))
        .await
        .map_err(|error| format!("the revert task stopped: {error}"))?
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

/// **Print** a group PDF, summary PDF or reprint, on its own thread: one that is not there any more
/// is written again first. The print preview is the front end's, over its PDF bytes.
#[tauri::command]
pub async fn print_output(app: AppHandle, day: String, file: String) -> Result<OpenView, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        ready_to_open(&state, &day, &file).map(|(_, view)| view)
    })
    .await
    .map_err(|error| format!("the print task stopped: {error}"))?
}

/// The bytes of an output of `day` that is there, for the print preview.
#[tauri::command]
pub async fn output_pdf(app: AppHandle, day: String, file: String) -> Result<Response, String> {
    tauri::async_runtime::spawn_blocking(move || {
        output_bytes(&app.state::<AppState>(), &day, &file).map(Response::new)
    })
    .await
    .map_err(|error| format!("the print task stopped: {error}"))?
}

/// The **Sent** mark of a group PDF: set as its print preview opens, taken back by its ✕.
#[tauri::command]
pub async fn set_sent(
    app: AppHandle,
    day: String,
    batch: BatchId,
    group: String,
    sent: bool,
) -> Result<(), String> {
    let mark = PrintSent { batch, group, sent };
    tauri::async_runtime::spawn_blocking(move || mark_sent(&app.state::<AppState>(), &day, mark))
        .await
        .map_err(|error| format!("the Sent task stopped: {error}"))?
}
