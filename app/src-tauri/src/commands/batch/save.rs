//! Saving the open draft: the plan counted, the batch written and checked, and the save's
//! progress carried to the window.

use std::sync::mpsc;
use std::thread;

use packing_engine::day::{DayState, PackingListChoice};
use packing_pdf::batch;
use packing_pdf::worker::SaveProgress;

use crate::AppState;
use crate::commands::day::settings_load;

use super::views::SaveResultView;

/// Apply the plan and save the batch, then forget the draft. A failed save keeps the draft so *Save batch* can be pressed
/// again. The check before saving runs on every save; when it finds something and `save_anyway`
/// is not set, nothing is renamed, `state.json` is untouched, the draft stays and the findings
/// come back in the view (`saved: false`) — the Plan screen then shows the dialog with *Save
/// anyway* and *Cancel*.
///
/// `on_progress` is called for every step of the write and of the check; the window turns it into
/// a `save-progress` event.
pub(super) fn save_batch_impl(
    state: &AppState,
    packing: Option<PackingListChoice>,
    save_anyway: bool,
    on_progress: impl Fn(SaveProgress) + Send + 'static,
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
    // An amend saves into the state without its batch, so the numbers are reused and the batch is
    // replaced; a new batch reads the day's state from disk.
    let mut day_state = match &open.amend {
        Some(amend) => amend.state.clone(),
        None => DayState::load(&day_dir).map_err(|error| error.to_string())?,
    };
    let plan = open.plan.count(&open.draft.orders, open.first_number);

    // The check before saving reports `Writing`/`Checking` progress; a forwarder thread carries
    // it out of the worker call and to the window, as the reading step does with `read-progress`.
    let (progress_tx, progress_rx) = mpsc::channel::<SaveProgress>();
    let forwarder = thread::spawn(move || {
        while let Ok(progress) = progress_rx.recv() {
            on_progress(progress);
        }
    });

    // The batch records the grouping it was packed with, so an amend reads it again.
    let grouping = open.grouping.clone();
    let result = state.with_worker(|worker| {
        batch::save(
            &batch::SaveRequest {
                worker,
                draft: &open.draft,
                plan: &plan,
                packing: &packing,
                grouping: &grouping,
                day: &day,
                day_dir: &day_dir,
                save_anyway,
                progress: progress_tx,
            },
            &mut day_state,
        )
    });

    let _ = forwarder.join(); // the sender is dropped when `with_worker` returns

    match result {
        Ok(Ok(save)) => {
            let saved = save.saved;
            // The check stopped the save (the temporary folder is gone, `state.json` is
            // untouched): the draft stays so *Save anyway* can be pressed again.
            if !saved {
                state.restore_draft(open);
            }
            Ok(save_view(save, saved))
        }
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

/// The save's outcome as the window's view (`saved: false` when the check stopped it).
fn save_view(save: batch::SaveResult, saved: bool) -> SaveResultView {
    SaveResultView {
        batch: save.batch,
        pdfs: if saved { save.pdf_files.len() } else { 0 },
        packing_lists: if saved { save.packing_lists.len() } else { 0 },
        saved,
        errors: save.errors,
        warnings: save.warnings,
    }
}
