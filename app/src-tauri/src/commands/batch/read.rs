//! Reading a new batch or an amendment into an open draft (08 › *2. New batch* → *3. Plan*): the
//! files read in order, the categories and couriers loaded, and the draft stored in
//! [`AppState`].

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::thread;

use packing_engine::day::DayState;
use packing_engine::groupings::{self, Grouping};
use packing_engine::label::{self, CourierTable};
use packing_engine::plan::Plan;
use packing_engine::rules::Category;
use packing_pdf::batch;
use packing_pdf::worker::Progress;

use crate::commands::day::remember_last_grouping;
use crate::{AmendState, AppState, COURIERS_FILE, OpenDraft};

use super::logo::{read_pictures, read_result};
use super::views::ReadResult;

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
    pub(super) paths: Vec<PathBuf>,
    /// Supplies the duplicate guard and the day's next number.
    pub(super) base_state: DayState,
    /// Marks the draft as redoing a batch (07 › *Amend*).
    pub(super) amend: Option<AmendState>,
    /// The grouping to read with (04 › *Sets*).
    pub(super) grouping: Grouping,
    /// Read a new batch in file-name order (= download order, 08 › *2. New batch*) and an amend in
    /// the given order (its `download/` copy in the order stored in `state.json`).
    pub(super) sort_by_name: bool,
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
