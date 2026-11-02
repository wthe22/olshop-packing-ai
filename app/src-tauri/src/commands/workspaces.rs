//! Settings › Workspaces (workspaces.md): every workspace of the data folder with its dates, and
//! renaming or deleting one. The open one follows its new name; deleting it opens the workspace
//! the start would open, or none.

use packing_store::{dates, delete_workspace, rename_workspace, workspaces};
use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use crate::AppState;
use crate::commands::start::workspace_to_open;

#[derive(Serialize)]
pub struct WorkspaceView {
    name: String,
    /// Its dates that have a folder.
    dates: usize,
    open: bool,
}

pub(crate) fn overview(state: &AppState) -> Result<Vec<WorkspaceView>, String> {
    state.with_data(|data| {
        workspaces(&data.dir)
            .and_then(|list| {
                list.into_iter()
                    .map(|name| {
                        Ok(WorkspaceView {
                            dates: dates(&data.dir, &name)?.len(),
                            open: data.workspace.as_ref() == Some(&name),
                            name,
                        })
                    })
                    .collect()
            })
            .map_err(|error| error.to_string())
    })
}

/// Rename `name`; the open workspace stays open under its new name, an open draft is dropped.
pub(crate) fn rename(state: &AppState, name: &str, new_name: &str) -> Result<(), String> {
    let was_open = state.write_data(|data, _| {
        let open = data.workspace.as_deref() == Some(name);
        if open {
            data.day = None;
        }
        rename_workspace(&data.dir, name, new_name).map_err(|error| error.to_string())?;
        if open {
            data.workspace = Some(new_name.to_string());
            state
                .index()
                .rebuild(data.dir.clone(), new_name.to_string());
        }
        Ok(open)
    })?;
    if was_open {
        state.clear_draft();
    }
    Ok(())
}

/// Move `name` to the Recycle Bin. The workspace open afterwards: the same one, or, when `name`
/// was open, the one the start would open; none when no workspace is left.
pub(crate) fn delete(state: &AppState, name: &str) -> Result<Option<String>, String> {
    let (was_open, now_open) = state.write_data(|data, _| {
        let open = data.workspace.as_deref() == Some(name);
        if open {
            data.day = None;
        }
        delete_workspace(&data.dir, name).map_err(|error| error.to_string())?;
        if open {
            data.workspace = workspace_to_open(&data.dir)?;
            if let Some(workspace) = &data.workspace {
                state.index().rebuild(data.dir.clone(), workspace.clone());
            }
        }
        Ok((open, data.workspace.clone()))
    })?;
    if was_open {
        state.clear_draft();
    }
    Ok(now_open)
}

#[tauri::command]
pub fn workspaces_overview(state: State<'_, AppState>) -> Result<Vec<WorkspaceView>, String> {
    overview(&state)
}

/// On its own thread, as each write: it may wait for another copy of the app to finish writing.
#[tauri::command]
pub async fn workspace_rename(
    app: AppHandle,
    name: String,
    new_name: String,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || rename(&app.state::<AppState>(), &name, &new_name))
        .await
        .map_err(|error| format!("the rename task stopped: {error}"))?
}

#[tauri::command]
pub async fn workspace_delete(app: AppHandle, name: String) -> Result<Option<String>, String> {
    tauri::async_runtime::spawn_blocking(move || delete(&app.state::<AppState>(), &name))
        .await
        .map_err(|error| format!("the delete task stopped: {error}"))?
}

#[cfg(test)]
mod tests;
