//! The New batch, Plan and save commands the window calls, over the modules beside this file.
//! The batch is for the open date (the Day screen's); the draft lives in [`AppState`] until it is
//! saved or discarded.

use std::path::PathBuf;

use jiff::civil::Date;
use packing_batch::check_files as check;
use packing_engine::ids::BatchId;
use packing_engine::orders::LogoAnswer;
use packing_engine::rules::parse_filter;
use packing_pdf::worker::Progress;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::commands::day::date_of;
use crate::{AppState, settings, today};

mod logo;
mod plan_view;
mod read;
mod repeats;
mod save;
mod views;

use self::read::{Reading, read_into_draft, read_result};
use self::repeats::RepeatChoices;
use self::views::{
    AmendStart, FileCheckView, PlanEdit, PlanView, ReadProgress, ReadResult, SaveProgressEvent,
    SaveResultView, file_view,
};

/// The date a new batch is for: the one the Day screen has open.
fn open_date(state: &AppState) -> Result<Date, String> {
    state.with_data(|data| {
        Ok(data
            .day
            .as_ref()
            .map_or_else(today, |day| day.folder.date()))
    })
}

/// *Add files…* / a drop: each file's page count, size, time and *used in batch n*.
#[tauri::command]
pub fn check_files(
    state: State<'_, AppState>,
    paths: Vec<String>,
) -> Result<Vec<FileCheckView>, String> {
    let paths: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
    checked(&state, &paths)
}

pub(crate) fn checked(state: &AppState, paths: &[PathBuf]) -> Result<Vec<FileCheckView>, String> {
    let date = open_date(state)?;
    state.with_data(|data| {
        let day = data.open_day(date)?;
        let files = state
            .with_worker(|worker| check(worker, paths, day.journal.fold()))?
            .map_err(|error| error.to_string())?;
        Ok(files.iter().map(file_view).collect())
    })
}

/// The sort rules of `packing-workflows.toml`, and the one to preselect: the rule of the workspace's
/// newest batch while it exists, else the first.
#[derive(Serialize)]
pub struct SortRulesView {
    rules: Vec<String>,
    preselected: String,
}

#[tauri::command]
pub fn sort_rules(state: State<'_, AppState>) -> Result<SortRulesView, String> {
    sort_rules_of(&state)
}

pub(crate) fn sort_rules_of(state: &AppState) -> Result<SortRulesView, String> {
    let (_, workflows) = settings::workflows(state.program())?;
    let rules: Vec<String> = workflows
        .sort_rules
        .iter()
        .map(|r| r.name.clone())
        .collect();
    let newest = state
        .with_data(|data| {
            Ok(data.day.as_ref().and_then(|day| {
                let fold = day.journal.fold();
                fold.live
                    .values()
                    .next_back()
                    .map(|live| live.saved.sort_rule.name.clone())
            }))
        })
        .ok()
        .flatten();
    let preselected = newest
        .filter(|name| rules.contains(name))
        .or_else(|| rules.first().cloned())
        .unwrap_or_default();
    Ok(SortRulesView { rules, preselected })
}

/// *Read labels*: read on a blocking thread, emitting `read-progress {page, pages}`. A read
/// replaces any open draft. `sort_rule` names the sort rule; `redo` is the batch an amend redoes,
/// which reads with that batch's rule.
#[tauri::command]
pub async fn read_labels(
    app: AppHandle,
    paths: Vec<String>,
    sort_rule: String,
    redo: Option<BatchId>,
) -> Result<ReadResult, String> {
    let paths: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let reading = Reading {
            paths,
            rule: sort_rule,
            redo,
            date: open_date(&state)?,
        };
        read_into_draft(&state, reading, progress_to(app.clone()))
    })
    .await
    .map_err(|error| format!("the reading task stopped: {error}"))?
}

fn progress_to(app: AppHandle) -> impl Fn(Progress) + Send + 'static {
    move |progress: Progress| {
        let _ = app.emit(
            "read-progress",
            ReadProgress {
                page: progress.page,
                pages: progress.pages,
            },
        );
    }
}

/// *Cancel* while reading: nothing is written.
#[tauri::command]
pub fn cancel_read(state: State<'_, AppState>) {
    state.request_cancel();
}

/// The logo prompt's answer about one unseen logo; the read as it now is.
#[tauri::command]
pub fn answer_logo(
    state: State<'_, AppState>,
    hash: String,
    answer: LogoAnswer,
) -> Result<ReadResult, String> {
    logo::answer_logo(&state, &hash, &answer)
}

/// The live check of a one-off group's filter.
#[tauri::command]
pub fn check_condition(condition: String) -> Result<(), String> {
    parse_filter(&condition)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

/// Repeated orders: keep the copies the choices name; the read as it now is. A choice that cannot
/// apply changes nothing.
#[tauri::command]
pub fn resolve_repeats(
    state: State<'_, AppState>,
    choices: RepeatChoices,
) -> Result<ReadResult, String> {
    resolve(&state, &choices)
}

pub(crate) fn resolve(state: &AppState, choices: &RepeatChoices) -> Result<ReadResult, String> {
    let (_, workflows) = settings::workflows(state.program())?;
    state.with_draft(|open| {
        open.draft
            .choose(choices.choices())
            .map_err(|error| error.to_string())?;
        state.with_data(|data| {
            let day = data.open_day(open.date)?;
            Ok(read_result(open, day.journal.fold(), &workflows))
        })
    })
}

/// Apply the plan edits and sort again.
#[tauri::command]
pub fn plan(state: State<'_, AppState>, edits: Vec<PlanEdit>) -> Result<PlanView, String> {
    state.with_draft(|open| {
        plan_view::apply_edits(open, &edits)?;
        state.with_data(|data| {
            let day = data.open_day(open.date)?;
            Ok(plan_view::plan_view(open, day.journal.fold()))
        })
    })
}

/// *Save batch* on its own thread, emitting `save-progress {stage, page, pages}`. `save_anyway`
/// is the findings dialog's *Save anyway*.
#[tauri::command]
pub async fn save_batch(
    app: AppHandle,
    save_anyway: Option<bool>,
) -> Result<SaveResultView, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let progress = |progress| {
            let _ = app.emit("save-progress", SaveProgressEvent::of(progress));
        };
        save::save_draft(&state, save_anyway.unwrap_or(false), &progress)
    })
    .await
    .map_err(|error| format!("the save task stopped: {error}"))?
}

/// *Cancel* on the file list, or *Cancel amend*: the draft is forgotten; nothing was saved.
#[tauri::command]
pub fn discard_draft(state: State<'_, AppState>) {
    state.clear_draft();
}

/// *Amend* the newest batch: its inputs from `input/` read again with its rule and its kept
/// copies, the draft open; the file list comes back for the amend's *Back*.
#[tauri::command]
pub async fn start_amend(app: AppHandle, day: String) -> Result<AmendStart, String> {
    let date = date_of(&day)?;
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        amend_start(&state, date, progress_to(app.clone()))
    })
    .await
    .map_err(|error| format!("the amend task stopped: {error}"))?
}

pub(crate) fn amend_start(
    state: &AppState,
    date: Date,
    on_progress: impl Fn(Progress) + Send + 'static,
) -> Result<AmendStart, String> {
    let (batch, files) = state.with_data(|data| {
        let open = data.open_day(date)?;
        let (_, workflows) = settings::workflows(state.program())?;
        let redo = packing_batch::amend(&open.folder, open.journal.fold(), &workflows)
            .map_err(|error| error.to_string())?;
        Ok((redo.batch, redo.files))
    })?;
    let reading = Reading {
        paths: files.iter().map(|file| file.path.clone()).collect(),
        rule: String::new(),
        redo: Some(batch),
        date,
    };
    let read = read_into_draft(state, reading, on_progress)?;
    Ok(AmendStart {
        batch,
        files: files.iter().map(file_view).collect(),
        read,
    })
}

#[cfg(test)]
mod tests;
