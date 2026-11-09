//! What other copies of the app wrote to the data folder since the window last looked
//! (program.md, *Several copies*): the window looks each time it gets the focus and reads again what changed.
//! Every changed date is taken into the search index, and the open workspace is followed when
//! another copy renamed or deleted it.

use std::path::PathBuf;

use packing_store::workspaces;
use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::AppState;
use crate::commands::start::workspace_to_open;

#[derive(Debug, Default, PartialEq, Eq, Serialize)]
pub struct DataChanges {
    /// The open date changed, or was opened while another copy wrote: the Day screen reads it again.
    day: bool,
    /// A workspace or a date came or went, or a journal changed: the panel reads its list again.
    panel: bool,
    /// The open workspace is no longer in the data folder.
    gone: Option<WorkspaceGone>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct WorkspaceGone {
    workspace: String,
    /// The workspace opened instead, as at start; none when no workspace is left.
    now_open: Option<String>,
}

/// What one look found in the data folder's state.
struct Looked {
    dir: PathBuf,
    listed: bool,
    day: bool,
    gone: Option<WorkspaceGone>,
}

/// Look for other copies' writes. While another command of this window uses the data folder there
/// is nothing to look at: that command reads the folder as it is.
pub(crate) fn changes(state: &AppState) -> Result<DataChanges, String> {
    let Some(looked) = state.try_with_data(|data| {
        let list = workspaces(&data.dir).map_err(|error| error.to_string())?;
        let listed = list != data.workspaces;
        data.workspaces = list;
        let mut looked = Looked {
            dir: data.dir.clone(),
            listed,
            day: false,
            gone: None,
        };
        let Some(workspace) = data.workspace.clone() else {
            return Ok(looked);
        };
        if data.workspaces.contains(&workspace) {
            looked.day = data
                .day
                .as_ref()
                .is_some_and(|day| !day.settled || day.journal.changed_on_disk());
        } else {
            data.day = None;
            data.workspace = workspace_to_open(&data.dir)?;
            if let Some(open) = &data.workspace {
                state.index().rebuild(data.dir.clone(), open.clone());
            }
            looked.gone = Some(WorkspaceGone {
                workspace,
                now_open: data.workspace.clone(),
            });
        }
        Ok(looked)
    })?
    else {
        return Ok(DataChanges::default());
    };
    if looked.gone.is_some() {
        state.clear_draft();
        return Ok(DataChanges {
            day: true,
            panel: true,
            gone: looked.gone,
        });
    }
    let dates = state
        .index()
        .take_in_changes(&looked.dir)
        .map_err(|error| error.to_string())?;
    Ok(DataChanges {
        day: looked.day,
        panel: looked.listed || dates,
        gone: None,
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
