//! The side panel (panel.md): the namespaces of the data folder, the open one's dates with their
//! orders from the search index, and opening another namespace.

use jiff::civil::Date;
use packing_store::{dates, namespaces};
use serde::Serialize;
use tauri::State;

use crate::{AppState, today};

#[derive(Serialize)]
pub struct PanelView {
    namespaces: Vec<String>,
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
    let (list, open, mut folders, namespace_dir) = state.with_data(|data| {
        let list = namespaces(&data.dir).map_err(|error| error.to_string())?;
        let folders = match &data.namespace {
            Some(namespace) => dates(&data.dir, namespace).map_err(|error| error.to_string())?,
            None => Vec::new(),
        };
        let namespace_dir = data.namespace.as_ref().map(|name| data.dir.join(name));
        Ok((list, data.namespace.clone(), folders, namespace_dir))
    })?;
    let today = today();
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
            folder: namespace_dir
                .as_ref()
                .filter(|_| date != today || today_has_folder)
                .map(|dir| dir.join(date.to_string()).display().to_string()),
        })
        .collect();
    Ok(PanelView {
        namespaces: list,
        open,
        dates,
        loading,
    })
}

/// Open another namespace of the data folder on today's date; an open draft is dropped.
pub(crate) fn open_namespace(state: &AppState, name: &str) -> Result<(), String> {
    state.with_data(|data| {
        let list = namespaces(&data.dir).map_err(|error| error.to_string())?;
        if !list.iter().any(|known| known == name) {
            return Err(format!("{name} is not a namespace of this data folder."));
        }
        data.namespace = Some(name.to_string());
        data.day = None;
        state.index().rebuild(data.dir.clone(), name.to_string());
        Ok(())
    })?;
    state.clear_draft();
    Ok(())
}

#[tauri::command]
pub fn namespaces_list(state: State<'_, AppState>) -> Result<PanelView, String> {
    panel(&state)
}

#[tauri::command]
pub fn namespace_open(state: State<'_, AppState>, name: String) -> Result<(), String> {
    open_namespace(&state, &name)
}
