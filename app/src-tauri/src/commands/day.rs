//! The Day screen (05-after-save.md): the open namespace's date from its fold, and what changes it —
//! the Printed tick and revert — each one append through `packing-batch`.

mod view;

use jiff::civil::Date;
use packing_engine::ids::BatchId;
use packing_engine::record::PrintedMarked;
use packing_store::dates;
use tauri::State;

use crate::{APP_VERSION, AppState};

pub use self::view::{BatchView, DayOverview, GroupPdfView, Totals};

pub(crate) fn date_of(day: &str) -> Result<Date, String> {
    day.parse()
        .map_err(|_| format!("{day} is not a date (YYYY-MM-DD)."))
}

/// Open `day` of the namespace (folded and checked unless it is open) and describe it.
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

pub(crate) fn revert(state: &AppState, day: &str) -> Result<BatchId, String> {
    let date = date_of(day)?;
    state.with_data(|data| {
        let open = data.open_day(date)?;
        let batch = packing_batch::revert(&open.folder, &mut open.journal, APP_VERSION)
            .map_err(|error| error.to_string())?;
        open.outputs.retain(|problem| problem.batch != batch);
        Ok(batch)
    })
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

/// The open namespace's dates with a folder, newest first.
#[tauri::command]
pub fn days(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    state.with_data(|data| {
        let Some(namespace) = &data.namespace else {
            return Ok(Vec::new());
        };
        let mut dates = dates(&data.dir, namespace).map_err(|error| error.to_string())?;
        dates.sort_unstable_by(|a, b| b.cmp(a));
        Ok(dates.iter().map(ToString::to_string).collect())
    })
}

/// *Revert* the newest batch. A file open in a PDF viewer stops it with the file named, and the
/// date is as before.
#[tauri::command]
pub fn revert_last(state: State<'_, AppState>, day: String) -> Result<(), String> {
    revert(&state, &day).map(|_| ())
}
