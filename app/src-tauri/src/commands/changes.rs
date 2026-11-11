//! What other copies of the app wrote to the data folder since the window last looked
//! (program.md, *Several copies*): the window looks each time it gets the focus and reads again what changed.
//! Every changed date is taken into the search index.

use std::path::PathBuf;

use packing_store::dates;
use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::AppState;

#[derive(Debug, Default, PartialEq, Eq, Serialize)]
pub struct DataChanges {
    /// The open date changed, or was opened while another copy wrote: the Day screen reads it again.
    day: bool,
    /// A date came or went, or a journal changed: the panel reads its list again.
    panel: bool,
}

/// What one look found in the data folder's state.
struct Looked {
    dir: PathBuf,
    listed: bool,
    day: bool,
}

/// Look for other copies' writes. While another command of this window uses the data folder there
/// is nothing to look at: that command reads the folder as it is.
pub(crate) fn changes(state: &AppState) -> Result<DataChanges, String> {
    let Some(looked) = state.try_with_data(|data| {
        let list = dates(&data.dir).map_err(|error| error.to_string())?;
        let listed = list != data.dates;
        data.dates = list;
        Ok(Looked {
            dir: data.dir.clone(),
            listed,
            day: data
                .day
                .as_ref()
                .is_some_and(|day| !day.settled || day.journal.changed_on_disk()),
        })
    })?
    else {
        return Ok(DataChanges::default());
    };
    let journals = state
        .index()
        .take_in_changes(&looked.dir)
        .map_err(|error| error.to_string())?;
    Ok(DataChanges {
        day: looked.day,
        panel: looked.listed || journals,
    })
}

/// The window's look, on its own thread: a slow network folder never holds up the window.
#[tauri::command]
pub async fn data_changes(app: AppHandle) -> Result<DataChanges, String> {
    tauri::async_runtime::spawn_blocking(move || changes(&app.state::<AppState>()))
        .await
        .map_err(|error| format!("the look for changes stopped: {error}"))?
}

#[cfg(test)]
mod tests;
