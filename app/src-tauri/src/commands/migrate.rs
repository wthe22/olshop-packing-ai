//! The question at start when the migration would change the data folder (start.md, *Update the
//! data folder?*): what it would change, **Update** with its progress, and **Open read-only**.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Sender};
use std::thread;

use packing_batch::{MigrateProgress, MigrateRequest, migrate_date};
use packing_engine::ids::BatchId;
use packing_engine::workflows::Workflows;
use packing_store::{DataLock, DateFolder, DateMigration, Journal, migrations};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::commands::start::open_data;
use crate::{APP_VERSION, AppState, Data, settings};

/// A date the migration would change, as the question lists it.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct MigrationView {
    date: String,
    /// Its files are still in v0.7's input and output folders and are moved into the date folder.
    v0_7: bool,
    /// Its views are of an older version and are written again.
    older_views: bool,
    /// The batches whose receivers' areas are read, and how many orders they hold.
    batches: usize,
    orders: usize,
}

/// What the migration would change in the data folder `dir`, oldest date first.
pub(crate) fn pending(dir: &Path) -> Result<Vec<MigrationView>, String> {
    let found = migrations(dir).map_err(|error| error.to_string())?;
    Ok(found
        .into_iter()
        .map(|date| MigrationView {
            date: date.date.to_string(),
            v0_7: date.v0_7,
            older_views: date.older_views.is_some(),
            batches: date.batches.len(),
            orders: date.orders,
        })
        .collect())
}

/// A date the Update has finished.
#[derive(Clone, Debug, Serialize)]
pub struct DateUpdated {
    date: String,
    /// Its files of v0.7 were moved into the date folder.
    moved: bool,
    /// Its batches whose areas were read.
    batches: usize,
    skipped: Vec<SkippedView>,
}

/// A batch whose areas were not read, and why.
#[derive(Clone, Debug, Serialize)]
pub struct SkippedView {
    batch: BatchId,
    reason: String,
}

/// What the Update shows as it runs: the file of v0.7 being moved, the label page being read, and
/// each date finished.
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MigrationStep {
    Moving {
        date: String,
        file: usize,
        files: usize,
    },
    Reading {
        date: String,
        batch: BatchId,
        page: usize,
        pages: usize,
    },
    Updated(DateUpdated),
}

fn data_dir(state: &AppState) -> Result<PathBuf, String> {
    settings::data_dir(state.program())?.ok_or_else(|| "No data folder is set yet.".to_string())
}

/// **Update**: every date the migration would change, oldest first, under the data folder's lock;
/// then the folder opens as at start.
pub(crate) fn update(
    state: &AppState,
    steps: Sender<MigrationStep>,
) -> Result<Vec<DateUpdated>, String> {
    let dir = data_dir(state)?;
    let lock = DataLock::acquire(&dir).map_err(|error| error.to_string())?;
    settings::ensure_files(state.program())?;
    let (_, workflows) = settings::workflows(state.program())?;
    let mut updated = Vec::new();
    for found in migrations(&dir).map_err(|error| error.to_string())? {
        let date = update_date(state, &dir, &found, &workflows, &steps)?;
        let _ = steps.send(MigrationStep::Updated(date.clone()));
        updated.push(date);
    }
    drop(lock);
    open_data(state, dir)?;
    Ok(updated)
}

fn update_date(
    state: &AppState,
    dir: &Path,
    found: &DateMigration,
    workflows: &Workflows,
    steps: &Sender<MigrationStep>,
) -> Result<DateUpdated, String> {
    let folder = DateFolder::new(dir, found.date);
    let mut journal = Journal::open(&folder).map_err(|error| error.to_string())?;
    let (progress, read) = mpsc::channel::<MigrateProgress>();
    let (date, out) = (found.date.to_string(), steps.clone());
    let forwarder = thread::spawn(move || {
        for step in read {
            let date = date.clone();
            let _ = out.send(match step {
                MigrateProgress::Moving { file, files } => {
                    MigrationStep::Moving { date, file, files }
                }
                MigrateProgress::Reading { batch, page, pages } => MigrationStep::Reading {
                    date,
                    batch,
                    page,
                    pages,
                },
            });
        }
    });
    let migrated = state.with_worker(|worker| {
        let request = MigrateRequest {
            worker,
            workflows,
            app: APP_VERSION,
            progress,
        };
        migrate_date(&folder, &mut journal, &request)
    });
    let _ = forwarder.join();
    let migrated = migrated?.map_err(|error| error.to_string())?;
    Ok(DateUpdated {
        date: found.date.to_string(),
        moved: found.v0_7,
        batches: migrated.read,
        skipped: migrated
            .skipped
            .into_iter()
            .map(|batch| SkippedView {
                batch: batch.batch,
                reason: batch.error.to_string(),
            })
            .collect(),
    })
}

/// **Open read-only**: the data folder opens without its lock, and nothing in it is written.
pub(crate) fn open_read_only(state: &AppState) -> Result<(), String> {
    let dir = data_dir(state)?;
    settings::workflows(state.program())?;
    state.index().rebuild(dir.clone());
    state.set_data(Data::read_only(dir));
    Ok(())
}

#[tauri::command]
pub async fn update_data(app: AppHandle) -> Result<Vec<DateUpdated>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let (steps, sent) = mpsc::channel();
        let window = app.clone();
        let forwarder = thread::spawn(move || {
            for step in sent {
                let _ = window.emit("migration-step", step);
            }
        });
        let updated = update(&app.state::<AppState>(), steps);
        let _ = forwarder.join();
        updated
    })
    .await
    .map_err(|error| format!("the update stopped: {error}"))?
}

#[tauri::command]
pub fn read_only(state: State<'_, AppState>) -> Result<(), String> {
    open_read_only(&state)
}

#[cfg(test)]
mod tests;
