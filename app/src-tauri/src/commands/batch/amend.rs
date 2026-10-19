//! Amending and reverting the day's last batch: its own `download/` files read again as an
//! amendment, and its folder deleted before the day state changes.

use std::path::{Path, PathBuf};

use packing_engine::day::{DayState, delete_batch_folder};
use packing_engine::names;
use packing_pdf::batch;
use packing_pdf::worker::Progress;

use crate::{AmendState, AppState};

use super::read_into_draft;
use super::views::{AmendStart, ReadResult, file_check_view};

/// Revert the last batch (07 › *Revert*): delete its folder **first**, so a file open in a PDF
/// viewer stops with the 08 *file in use* message and changes nothing, then remove the batch and
/// its orders from `state.json`; the numbers are reused.
pub(super) fn revert_last_impl(state: &AppState, day: &str) -> Result<(), String> {
    state.set_day(day);
    let day_dir = state.day_dir()?;
    let mut day_state = DayState::load(&day_dir).map_err(|error| error.to_string())?;
    let Some(last) = day_state.batches.iter().map(|batch| batch.batch).max() else {
        return Err("No batch to undo on this day.".to_string());
    };
    let folder = day_dir.join(names::batch_folder(last));
    // Name the open file (07 › *Revert*: "the app says which file to close") and delete nothing
    // when one is found, so the day is left exactly as it was.
    if let Some(file) = locked_file(&folder) {
        return Err(file_in_use(&file));
    }
    delete_batch_folder(&day_dir, last).map_err(|error| error.to_string())?;
    day_state.revert_last();
    day_state.save(&day_dir).map_err(|error| error.to_string())
}

/// The 08 *Messages* file-in-use wording, naming the file to close.
fn file_in_use(path: &Path) -> String {
    format!(
        "{} is open in another program. Close it (e.g. the PDF viewer) and try again.",
        path.display()
    )
}

/// The first file inside `folder` (and its subfolders) that another program holds open. A viewer
/// keeps a PDF with read-only sharing, so opening it for writing fails with a sharing violation
/// (`ERROR_SHARING_VIOLATION`, 32); that is the file the owner must close before Revert.
fn locked_file(folder: &Path) -> Option<PathBuf> {
    let mut stack = vec![folder.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if let Err(error) = std::fs::OpenOptions::new().write(true).open(&path)
                && error.raw_os_error() == Some(32)
            {
                return Some(path);
            }
        }
    }
    None
}

/// Amend the last batch (08 › *1. Day* → *3. Plan*): read the batch's own `download/` files again
/// — the order stored in `state.json`, the batch's orders ignored by the duplicate guard and its
/// numbers reused ([`read_amend_draft`]) — and open the *Redo batch `<b>`* Plan screen with them,
/// the draft open. The batch's file list comes back too, so the Plan screen's *Back* shows the
/// selection page listing the current selection
/// (08 › *2. New batch* → *The amend's selection page*). Nothing is written.
pub(super) fn start_amend_impl(
    state: &AppState,
    day: &str,
    on_progress: impl Fn(Progress) + Send + 'static,
) -> Result<AmendStart, String> {
    state.set_day(day);
    let day_dir = state.day_dir()?;
    let day_state = DayState::load(&day_dir).map_err(|error| error.to_string())?;
    let amend = day_state
        .amend(&day_dir)
        .ok_or_else(|| "No batch to redo on this day.".to_string())?;
    // The selection page's starting list: the batch's own `download/` files, checked against the
    // day state without the batch, so none is marked *used in batch `<b>`* (Q-APP-2).
    let checks = state
        .with_worker(|worker| batch::check_files(worker, &amend.files, &amend.state))?
        .map_err(|error| error.to_string())?;
    let files = checks.into_iter().map(file_check_view).collect();
    let batch = amend.batch;
    // The read is the amendment's (the selection page's **Read labels** path): the batch's orders
    // are not seen as already saved and its numbers are reused.
    let read = read_amend_draft(state, amend.files, batch, on_progress)?;
    Ok(AmendStart { batch, files, read })
}

/// The amend's **Read labels** (07 › *Amend*): read the selection with the batch removed from
/// the day state, so the duplicate guard ignores the batch's own orders and the numbers are
/// reused, and in the selection's order (the batch's `download/` copy in `state.json` order, an
/// added file where the owner put it). `redoing` is the batch the selection page was opened for;
/// a state that no longer holds it stops the read.
pub(super) fn read_amend_draft(
    state: &AppState,
    paths: Vec<PathBuf>,
    redoing: i64,
    on_progress: impl Fn(Progress) + Send + 'static,
) -> Result<ReadResult, String> {
    let day_dir = state.day_dir()?;
    let day_state = DayState::load(&day_dir).map_err(|error| error.to_string())?;
    let amend = day_state
        .amend(&day_dir)
        .filter(|amend| amend.batch == redoing)
        .ok_or_else(|| "No batch to redo on this day.".to_string())?;
    let base_state = amend.state.clone();
    // The amend reads with the set the batch recorded, whatever was used last (04 › *Sets*;
    // 07 › *Amend*).
    let set = amend.set.clone();
    let amend_state = AmendState {
        batch: amend.batch,
        state: amend.state,
    };
    read_into_draft(
        state,
        paths,
        base_state,
        Some(amend_state),
        &set,
        false,
        on_progress,
    )
}
