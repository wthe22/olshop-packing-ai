//! Reading a batch into the open draft through `packing-batch`: the chosen files checked and read
//! on the worker with the sort rule, or an amend's own inputs with the batch's rule and its kept
//! copies; the read's progress carried to the window; and the read result the screens show.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::thread;

use jiff::civil::Date;
use packing_batch::{Draft, ReadRequest, amend, check_files, read_draft};
use packing_engine::ids::{BatchId, OrderId};
use packing_engine::repeats::LeftOutBecause;
use packing_engine::workflows::Workflows;
use packing_pdf::worker::Progress;
use packing_store::Fold;

use crate::settings;
use crate::{AppState, OpenDraft};

use super::logo::{logo_views, read_pictures};
use super::plan_view::{entries_of, plan_view};
use super::repeats::repeats_view;
use super::views::ReadResult;

/// What to read: the files in reading order, the sort rule's name, the batch an amend redoes,
/// and the date the batch is for.
pub(super) struct Reading {
    pub paths: Vec<PathBuf>,
    pub rule: String,
    pub redo: Option<BatchId>,
    pub date: Date,
}

/// Read into a new open draft, replacing any earlier one. A stop returns its message and keeps
/// nothing.
pub(super) fn read_into_draft(
    state: &AppState,
    reading: Reading,
    on_progress: impl Fn(Progress) + Send + 'static,
) -> Result<ReadResult, String> {
    if reading.paths.is_empty() {
        return Err("Choose at least one label file first.".to_string());
    }
    let (_, workflows) = settings::workflows(state.program())?;
    let cancel = Arc::new(AtomicBool::new(false));
    state.set_cancel(cancel.clone());
    let (progress, received) = mpsc::channel::<Progress>();
    let forwarder = thread::spawn(move || {
        while let Ok(progress) = received.recv() {
            on_progress(progress);
        }
    });
    let read = state.with_data(|data| {
        let day = data.open_day(reading.date)?;
        let fold = day.journal.fold();
        let (rule, record) = match reading.redo {
            Some(batch) => {
                let redo = amend(&day.folder, fold, &workflows).map_err(|e| e.to_string())?;
                if redo.batch != batch {
                    return Err("The batch to redo is no longer the newest.".to_string());
                }
                (redo.rule, Some(fold.live[&batch].saved.clone()))
            }
            None => (
                workflows
                    .sort_rule(&reading.rule)
                    .cloned()
                    .ok_or_else(|| format!("There is no sort rule \"{}\".", reading.rule))?,
                None,
            ),
        };
        state
            .with_worker(|worker| {
                let files = check_files(worker, &reading.paths, &day.folder, fold, reading.redo)?;
                let mut draft = read_draft(ReadRequest {
                    worker,
                    files,
                    workflows: &workflows,
                    rule: &rule,
                    fold,
                    redo: reading.redo,
                    cancel,
                    progress,
                })?;
                if let Some(record) = &record {
                    draft.keep_as_recorded(record)?;
                }
                let pictures = read_pictures(worker, &draft);
                Ok((draft, pictures))
            })?
            .map_err(|error: packing_batch::BatchError| error.to_string())
    });
    state.clear_cancel();
    let _ = forwarder.join();
    let (draft, logo_pictures) = read?;
    let mut open = OpenDraft {
        date: reading.date,
        entries: entries_of(&draft.plan),
        draft,
        logo_pictures,
        skipped_logos: Vec::new(),
    };
    open.apply_plan();
    let result = state.with_data(|data| {
        let day = data.open_day(open.date)?;
        Ok(read_result(&open, day.journal.fold(), &workflows))
    })?;
    state.set_draft(open);
    Ok(result)
}

/// The read as the screens show it: the counts, the warnings, the logos still to ask about and
/// the plan.
pub(super) fn read_result(open: &OpenDraft, fold: &Fold, workflows: &Workflows) -> ReadResult {
    let draft = &open.draft;
    let order_ids: BTreeSet<&OrderId> = draft.copies.iter().map(|c| &c.order_id).collect();
    let saved_earlier: BTreeSet<&OrderId> = draft
        .orders
        .left_out
        .iter()
        .filter(|left| matches!(left.because, LeftOutBecause::SavedEarlier { .. }))
        .map(|left| &draft.copies[left.copy].order_id)
        .collect();
    ReadResult {
        files: draft.files.len(),
        pages: draft.files.iter().map(|f| usize::from(f.page_count)).sum(),
        orders: draft.orders.kept.len(),
        total_orders: order_ids.len(),
        already_saved: saved_earlier.len(),
        warnings: warnings(draft, fold, saved_earlier.len()),
        amend: draft.redo,
        couriers: workflows.couriers.iter().map(|c| c.name.clone()).collect(),
        logos: logo_views(open),
        repeats: repeats_view(draft),
        plan: plan_view(open, fold),
    }
}

/// The orders already saved, the repeats, then each order's own warnings.
fn warnings(draft: &Draft, fold: &Fold, saved_earlier: usize) -> Vec<String> {
    let mut lines = Vec::new();
    if saved_earlier > 0 {
        lines.push(format!(
            "{saved_earlier} orders were already saved today and are left out."
        ));
    }
    let (repeats, extra, pages) = draft.orders.repeat_counts(&draft.copies);
    if repeats > 0 {
        lines.push(format!(
            "{repeats} orders are in the files more than once: {extra} extra copies \
             ({pages} pages) are left out, one copy of each order is kept."
        ));
    }
    lines.extend(draft.findings(fold).into_iter().map(|finding| finding.text));
    lines
}
