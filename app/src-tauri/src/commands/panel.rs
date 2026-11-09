//! The side panel (panel.md): the workspaces of the data folder, the open one's dates with their
//! orders from the search index, and opening another workspace.

use std::path::Path;

use jiff::civil::Date;
use packing_store::{dates, has_unprinted_groups, workspaces};
use serde::Serialize;
use tauri::State;

use crate::{AppState, today};

#[derive(Serialize)]
pub struct PanelView {
    workspaces: Vec<String>,
    /// The workspaces with a group PDF of today not marked Printed yet.
    unprinted: Vec<String>,
    open: Option<String>,
    /// Newest first; today is always listed.
    dates: Vec<DateRow>,
    /// The search index is still being built: no counts yet.
    loading: bool,
}

#[derive(Serialize)]
pub struct DateRow {
    date: String,
    today: bool,
    /// Its saved orders, from the search index.
    orders: Option<usize>,
    /// Its journal is damaged: it opens for viewing only.
    damaged: bool,
    /// The date's folder, for *Open folder*; none before its first commit.
    folder: Option<String>,
}

pub(crate) fn panel(state: &AppState) -> Result<PanelView, String> {
    let (list, open, mut folders, workspace_dir, data_dir) = state.with_data(|data| {
        let list = workspaces(&data.dir).map_err(|error| error.to_string())?;
        let folders = match &data.workspace {
            Some(workspace) => dates(&data.dir, workspace).map_err(|error| error.to_string())?,
            None => Vec::new(),
        };
        let workspace_dir = data.workspace.as_ref().map(|name| data.dir.join(name));
        Ok((
            list,
            data.workspace.clone(),
            folders,
            workspace_dir,
            data.dir.clone(),
        ))
    })?;
    let today = today();
    let unprinted = unprinted_workspaces(&data_dir, &list, today)?;
    let today_has_folder = folders.contains(&today);
    if !today_has_folder {
        folders.push(today);
    }
    folders.sort_unstable_by(|a, b| b.cmp(a));
    let counts = state.index().with(|index| {
        (
            index.counts(),
            index.damaged.keys().copied().collect::<Vec<Date>>(),
        )
    })?;
    let loading = counts.is_none();
    let dates = folders
        .into_iter()
        .map(|date| DateRow {
            date: date.to_string(),
            today: date == today,
            orders: counts
                .as_ref()
                .map(|(counts, _)| counts.get(&date).copied().unwrap_or(0)),
            damaged: counts
                .as_ref()
                .is_some_and(|(_, damaged)| damaged.contains(&date)),
            folder: workspace_dir
                .as_ref()
                .filter(|_| date != today || today_has_folder)
                .map(|dir| dir.join(date.to_string()).display().to_string()),
        })
        .collect();
    Ok(PanelView {
        workspaces: list,
        unprinted,
        open,
        dates,
        loading,
    })
}

/// The workspaces whose today has a group PDF not marked Printed, read from each journal as it is.
fn unprinted_workspaces(
    data_dir: &Path,
    list: &[String],
    today: Date,
) -> Result<Vec<String>, String> {
    let mut names = Vec::new();
    for name in list {
        if has_unprinted_groups(data_dir, name, today).map_err(|error| error.to_string())? {
            names.push(name.clone());
        }
    }
    Ok(names)
}

/// Open another workspace of the data folder on today's date; an open draft is dropped.
pub(crate) fn open_workspace(state: &AppState, name: &str) -> Result<(), String> {
    state.with_data(|data| {
        let list = workspaces(&data.dir).map_err(|error| error.to_string())?;
        if !list.iter().any(|known| known == name) {
            return Err(format!("{name} is not a workspace of this data folder."));
        }
        data.workspace = Some(name.to_string());
        data.day = None;
        state.index().rebuild(data.dir.clone(), name.to_string());
        Ok(())
    })?;
    state.clear_draft();
    Ok(())
}

#[tauri::command]
pub fn workspaces_list(state: State<'_, AppState>) -> Result<PanelView, String> {
    panel(&state)
}

#[tauri::command]
pub fn workspace_open(state: State<'_, AppState>, name: String) -> Result<(), String> {
    open_workspace(&state, &name)
}
