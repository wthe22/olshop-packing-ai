//! The side panel (panel.md): the dates of the data folder with their orders of every account, from
//! the search index, each date's size and whether it is archived.

use jiff::civil::Date;
use packing_store::{DateFolder, dates, folder_size};
use serde::Serialize;
use tauri::State;

use crate::{AppState, today};

#[derive(Serialize)]
pub struct PanelView {
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
    /// Every file of its folder together.
    size_bytes: u64,
    /// Its PDFs were removed (archive.md).
    archived: bool,
}

pub(crate) fn panel(state: &AppState) -> Result<PanelView, String> {
    let today = today();
    let (mut folders, data_dir) = state.with_data(|data| {
        let folders = dates(&data.dir).map_err(|error| error.to_string())?;
        Ok((folders, data.dir.clone()))
    })?;
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
    let archived = state
        .index()
        .with(|index| index.archived_dates(&data_dir))?
        .unwrap_or_default();
    let loading = counts.is_none();
    let mut dates = Vec::with_capacity(folders.len());
    for date in folders {
        let size_bytes = folder_size(DateFolder::new(&data_dir, date).dir())
            .map_err(|error| error.to_string())?;
        dates.push(DateRow {
            date: date.to_string(),
            today: date == today,
            orders: counts
                .as_ref()
                .map(|(counts, _)| counts.get(&date).copied().unwrap_or(0)),
            damaged: counts
                .as_ref()
                .is_some_and(|(_, damaged)| damaged.contains(&date)),
            folder: (date != today || today_has_folder)
                .then(|| data_dir.join(date.to_string()).display().to_string()),
            size_bytes,
            archived: archived.contains(&date),
        });
    }
    Ok(PanelView { dates, loading })
}

#[tauri::command]
pub fn panel_dates(state: State<'_, AppState>) -> Result<PanelView, String> {
    panel(&state)
}
