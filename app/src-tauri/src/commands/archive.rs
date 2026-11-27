//! Archive (archive.md): a date's PDFs deleted and its data kept, for one date (the panel's row
//! menu), a range of dates (Settings › Storage) or the dates older than `archive_after_days` (at
//! start); what that would delete, asked first; and the data folder's size Settings shows.

use std::path::Path;

use jiff::ToSpan;
use jiff::civil::Date;
use packing_store::{
    ArchivePlan, DateFolder, StoreError, archive, dates, folder_size, plan_archive,
};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::commands::day::date_of;
use crate::{AppState, settings, today};

/// The dates an archive is asked for.
#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ArchiveScope {
    /// The dates from `from` to `to`, both included; one date is a range of one.
    Range { from: String, to: String },
    /// The dates more than `days` days before today.
    OlderThan { days: u16 },
}

/// The dates more than `days` days before `today`.
pub(crate) fn older_than(dates: &[Date], today: Date, days: u16) -> Vec<Date> {
    let limit = today.saturating_sub(i64::from(days).days());
    dates.iter().copied().filter(|date| *date < limit).collect()
}

impl ArchiveScope {
    /// The dates of `all` (oldest first) it covers.
    fn dates(&self, all: &[Date], today: Date) -> Result<Vec<Date>, String> {
        Ok(match self {
            ArchiveScope::Range { from, to } => {
                let (from, to) = (date_of(from)?, date_of(to)?);
                all.iter()
                    .copied()
                    .filter(|date| (from..=to).contains(date))
                    .collect()
            }
            ArchiveScope::OlderThan { days } => older_than(all, today, *days),
        })
    }
}

/// What archiving the asked dates would delete.
#[derive(Debug, Serialize)]
pub struct ArchivePreview {
    /// The dates that would be archived, oldest first; today, a date with no batch and an archived
    /// date are passed over.
    dates: Vec<String>,
    pdfs: usize,
    bytes: u64,
}

/// The date being archived, as *Archiving: 2026-10-07, 3 of 18* names it.
#[derive(Clone, Debug, Serialize)]
pub struct ArchiveStep {
    date: String,
    number: usize,
    of: usize,
}

/// A PDF Windows refused to delete: it stays, and its date is not archived when it is an input.
#[derive(Debug, Serialize)]
pub struct LeftPdf {
    date: String,
    file: String,
}

#[derive(Debug, Serialize)]
pub struct ArchiveOutcome {
    /// The dates archived now.
    archived: Vec<String>,
    left: Vec<LeftPdf>,
}

/// The plans of the dates of data folder `dir` that `scope` covers and can be archived, oldest
/// first.
fn plans(dir: &Path, scope: &ArchiveScope) -> Result<Vec<ArchivePlan>, String> {
    let today = today();
    let all = dates(dir).map_err(|error| error.to_string())?;
    let mut plans = Vec::new();
    for date in scope.dates(&all, today)? {
        match plan_archive(&DateFolder::new(dir, date), today) {
            Ok(plan) => plans.push(plan),
            Err(StoreError::NotArchivable(_)) => {}
            Err(error) => return Err(error.to_string()),
        }
    }
    Ok(plans)
}

pub(crate) fn preview(state: &AppState, scope: &ArchiveScope) -> Result<ArchivePreview, String> {
    let dir = state.with_data(|data| Ok(data.dir.clone()))?;
    let plans = plans(&dir, scope)?;
    Ok(ArchivePreview {
        dates: plans
            .iter()
            .map(|plan| plan.folder.date().to_string())
            .collect(),
        pdfs: plans.iter().map(|plan| plan.pdfs.len()).sum(),
        bytes: plans.iter().map(ArchivePlan::bytes).sum(),
    })
}

/// Archive the dates `scope` covers, oldest first, under the data folder's lock; each date is
/// sent to `step` as it starts. Read-only, nothing is archived.
pub(crate) fn archive_dates(
    state: &AppState,
    scope: &ArchiveScope,
    step: &dyn Fn(ArchiveStep),
) -> Result<ArchiveOutcome, String> {
    state.write_data(|data, _lock| {
        let plans = plans(&data.dir, scope)?;
        let of = plans.len();
        let mut outcome = ArchiveOutcome {
            archived: Vec::new(),
            left: Vec::new(),
        };
        for (at, plan) in plans.into_iter().enumerate() {
            let date = plan.folder.date();
            step(ArchiveStep {
                date: date.to_string(),
                number: at + 1,
                of,
            });
            let done = archive(plan);
            if data
                .day
                .as_ref()
                .is_some_and(|day| day.folder.date() == date)
            {
                data.day = None;
            }
            if done.archived {
                outcome.archived.push(date.to_string());
            }
            outcome
                .left
                .extend(done.left.into_iter().map(|file| LeftPdf {
                    date: date.to_string(),
                    file,
                }));
        }
        Ok(outcome)
    })
}

/// Settings › Storage › *Used*: the data folder's size, its dates and how many are archived.
#[derive(Debug, Serialize)]
pub struct StorageUsed {
    bytes: u64,
    dates: usize,
    /// None while the search index is being built.
    archived: Option<usize>,
}

pub(crate) fn storage_used(state: &AppState) -> Result<StorageUsed, String> {
    let dir = state.with_data(|data| Ok(data.dir.clone()))?;
    let archived = state
        .index()
        .with(|index| index.archived_dates(&dir).len())?;
    Ok(StorageUsed {
        bytes: folder_size(&dir).map_err(|error| error.to_string())?,
        dates: dates(&dir).map_err(|error| error.to_string())?.len(),
        archived,
    })
}

#[tauri::command]
pub fn archive_preview(
    state: State<'_, AppState>,
    scope: ArchiveScope,
) -> Result<ArchivePreview, String> {
    preview(&state, &scope)
}

/// **Archive**, on its own thread; each date is sent as an `archive-step` event as it starts.
#[tauri::command]
pub async fn archive_scope(app: AppHandle, scope: ArchiveScope) -> Result<ArchiveOutcome, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let window = app.clone();
        let step = move |step: ArchiveStep| {
            let _ = window.emit("archive-step", step);
        };
        archive_dates(&app.state::<AppState>(), &scope, &step)
    })
    .await
    .map_err(|error| format!("the archive task stopped: {error}"))?
}

#[tauri::command]
pub fn storage(state: State<'_, AppState>) -> Result<StorageUsed, String> {
    storage_used(&state)
}

/// Settings › Storage › *Archive automatically*: its number of days; none: off.
#[tauri::command]
pub fn archive_after_days(state: State<'_, AppState>) -> Result<Option<u16>, String> {
    settings::archive_after_days(state.program())
}

#[tauri::command]
pub fn set_archive_after_days(state: State<'_, AppState>, days: Option<u16>) -> Result<(), String> {
    if days == Some(0) {
        return Err("Archive automatically takes at least 1 day.".to_string());
    }
    settings::set_archive_after_days(state.program(), days)
}

#[cfg(test)]
mod tests;
