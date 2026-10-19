//! New batch and Plan/Save commands (08 › *2. New batch*, *3. Plan*, *4. Messages*): the
//! `#[tauri::command]` functions the window calls and the read that opens a batch, over the
//! modules in the folder beside this file. The open draft lives in [`AppState`]; its label
//! documents stay open on the PDFium worker until the batch is saved or discarded (07 ›
//! *Commands between window and Rust*).

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::thread;

use packing_engine::day::{DayState, PackingListChoice};
use packing_engine::groupings::{self, Grouping};
use packing_engine::ids::BatchNumber;
use packing_engine::label::{self, CourierTable};
use packing_engine::orders::LogoAnswer;
use packing_engine::plan::Plan;
use packing_engine::rules::{Category, parse_condition};
use packing_pdf::batch;
use packing_pdf::worker::Progress;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::commands::day::remember_last_grouping;
use crate::{AmendState, AppState, COURIERS_FILE, OpenDraft};

mod amend;
mod logo;
mod plan_view;
mod save;
mod views;

use self::amend::*;
use self::logo::*;
use self::plan_view::*;
use self::save::*;
use self::views::*;

use self::logo::{read_pictures, read_result};
use self::views::ReadResult;

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
/// pages}`; returns the draft summary and the plan, or the stop error text. A read replaces any
/// open draft, so the Plan screen's *Back* and a new *Read labels* start again from the files and
/// no plan edit survives (08 › *3. Plan*). `grouping` is the grouping the new batch uses, chosen
/// on the selection page (04 › *Sets*). `redo` is the batch being redone when the read comes from
/// the amend's selection page (08 › *2. New batch* → *The amend's selection page*); `None` for a
/// new batch's file list. An amend reads with the grouping that batch recorded, whatever
/// `grouping` says (04 › *Sets*).
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

/// *Cancel* (08 › *Reading*): stop the running read; nothing is written.
#[tauri::command]
pub fn cancel_read(state: State<'_, AppState>) {
    state.request_cancel();
}

/// Answer the logo prompt about one unseen logo (08 › *The logo prompt*; D-LABEL-4): applies the
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
        Ok(plan_view(
            &open.plan,
            &open.draft.orders,
            first_number,
            &open.grouping,
        ))
    })?
}

/// *Save batch* (08 › *3. Plan* → *1. Day*): write the batch and forget the draft. Runs on its own
/// thread, emitting `save-progress {stage, page, pages}` (the check before saving reports
/// *Writing* and *Checking*); the window shows the stage and a progress bar while it runs. The
/// check before saving runs on every save; `save_anyway` (the dialog's *Save anyway*) lets a batch
/// with findings be written and stores the two lists with it (07 › *Check before saving*).
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
/// an amend's *Cancel amend* (08 › *2. New batch*, *3. Plan*): discard the open batch; nothing was
/// saved and nothing is written. The Plan screen's top *Back* does not come here — it keeps the
/// draft and shows the selection page again. For an amend this leaves the original batch
/// untouched (07 › *Amend*).
#[tauri::command]
pub fn discard_draft(state: State<'_, AppState>) {
    state.clear_draft();
}

/// *Revert* the last batch (08 › *1. Day*): delete its folder and forget it. A file open in a PDF
/// viewer stops with the 08 *file in use* message and changes nothing.
#[tauri::command]
pub fn revert_last(state: State<'_, AppState>, day: String) -> Result<(), String> {
    revert_last_impl(&state, &day)
}

/// *Amend* the last batch (08 › *1. Day* → *3. Plan*): read the batch's own `download/` files and
/// open the *Redo batch `<b>`* Plan screen with them, the draft open; the batch's file list comes
/// back for the *Back* selection page (08 › *2. New batch* → *The amend's selection page*). Runs
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

// The read a new batch and an amendment both go through.

/// Read the chosen files into a draft and store it (08 › *2. New batch* → *3. Plan*). Runs on a
/// blocking thread; `on_progress` is called for every page (the window turns it into a
/// `read-progress` event). A stop (no packing slip, every order already saved, a rules error,
/// cancelled) returns its message and writes nothing. `grouping` is the grouping of categories the
/// batch uses (04 › *Sets*), chosen on the New batch screen; it becomes the grouping used last
/// (07 › *Settings*), which the next New batch preselects.
pub(super) fn read_draft(
    state: &AppState,
    paths: Vec<PathBuf>,
    grouping: &Grouping,
    on_progress: impl Fn(Progress) + Send + 'static,
) -> Result<ReadResult, String> {
    let day_dir = state.day_dir()?;
    let day_state = DayState::load(&day_dir).map_err(|error| error.to_string())?;
    let read = read_into_draft(
        state,
        ReadRequest {
            paths,
            base_state: day_state,
            amend: None,
            grouping: grouping.clone(),
            sort_by_name: true,
        },
        on_progress,
    )?;
    remember_last_grouping(&state.data_folder()?, grouping);
    Ok(read)
}

/// What the shared read needs beyond `state`: the chosen files, the state it is checked against,
/// the batch being redone when this is an amend, the grouping to read with, and whether the files
/// are read in file-name order.
pub(super) struct ReadRequest {
    paths: Vec<PathBuf>,
    /// Supplies the duplicate guard and the day's next number.
    base_state: DayState,
    /// Marks the draft as redoing a batch (07 › *Amend*).
    amend: Option<AmendState>,
    /// The grouping to read with (04 › *Sets*).
    grouping: Grouping,
    /// Read a new batch in file-name order (= download order, 08 › *2. New batch*) and an amend in
    /// the given order (its `download/` copy in the order stored in `state.json`).
    sort_by_name: bool,
}

pub(super) fn read_into_draft(
    state: &AppState,
    request: ReadRequest,
    on_progress: impl Fn(Progress) + Send + 'static,
) -> Result<ReadResult, String> {
    if request.paths.is_empty() {
        return Err("Choose at least one label file first.".to_string());
    }
    let folder = state.data_folder()?;
    let categories = load_categories(&folder, &request.grouping)?;
    let couriers = load_couriers(&folder)?;

    let mut files = request.paths;
    if request.sort_by_name {
        files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
    }

    let cancel = Arc::new(AtomicBool::new(false));
    state.set_cancel(cancel.clone());

    let (progress_tx, progress_rx) = mpsc::channel::<Progress>();
    let forwarder = thread::spawn(move || {
        while let Ok(progress) = progress_rx.recv() {
            on_progress(progress);
        }
    });

    let inner = state.with_worker(|worker| {
        batch::check_files(worker, &files, &request.base_state).and_then(|checks| {
            batch::read(&batch::ReadRequest {
                worker,
                files: &checks,
                state: &request.base_state,
                cancel: cancel.clone(),
                progress: progress_tx,
                couriers: &couriers,
            })
        })
    });

    state.clear_cancel();
    let _ = forwarder.join(); // the sender is dropped when `with_worker` returns

    let draft = inner
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())?;
    // The prompt's pictures are read back while the label documents are still open on the
    // worker (08 › *The logo prompt*); an empty list costs nothing.
    let logo_pictures = state
        .with_worker(|worker| read_pictures(worker, &draft))
        .unwrap_or_default();
    let plan = Plan::from_categories(&categories);
    let first_number = request.base_state.next_pdf_number();
    let open = OpenDraft {
        draft,
        plan,
        first_number,
        couriers,
        grouping: request.grouping,
        amend: request.amend,
        logo_pictures,
    };
    let read = read_result(&open);
    state.set_draft(open);
    Ok(read)
}

/// Load the chosen grouping's categories — `<data>/categories.toml` for the grouping `default`,
/// else `<data>/categories/<grouping name>.toml` — or the one catch-all entry when it has no file
/// (04 › *Sets*, 07 › *Inputs*, 08 › *5. Categories*): every order then goes into one PDF set. A
/// file that does not load stops the batch with its line/col (08 › *Messages*).
pub(super) fn load_categories(folder: &Path, grouping: &Grouping) -> Result<Vec<Category>, String> {
    groupings::load(folder, grouping)
        .map_err(|error| format!("The categories have an error: {error}. Fix it in Categories."))
}

/// Load `<data>/couriers.toml`, or the built-in list when the data folder has none
/// (07 › *Inputs*, D-LABEL-1). A broken file stops the read with its line (08 › *Messages*).
pub(super) fn load_couriers(folder: &Path) -> Result<CourierTable, String> {
    label::load_couriers_file(&folder.join(COURIERS_FILE)).map_err(|error| {
        format!("The couriers have an error: {error}. Fix {COURIERS_FILE} in the data folder.")
    })
}

#[cfg(test)]
mod tests;
