//! Settings › Workspaces (workspaces.md): every workspace of the data folder with its dates, and
//! renaming or deleting one. The open one follows its new name; deleting it opens the workspace
//! the start would open, or none.

use packing_store::{dates, delete_workspace, rename_workspace, workspaces};
use serde::Serialize;
use tauri::State;

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
    let was_open = state.with_data(|data| {
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
    let (was_open, now_open) = state.with_data(|data| {
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

#[tauri::command]
pub fn workspace_rename(
    state: State<'_, AppState>,
    name: String,
    new_name: String,
) -> Result<(), String> {
    rename(&state, &name, &new_name)
}

#[tauri::command]
pub fn workspace_delete(
    state: State<'_, AppState>,
    name: String,
) -> Result<Option<String>, String> {
    delete(&state, &name)
}

#[cfg(test)]
mod tests;
