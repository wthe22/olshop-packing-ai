//! Steps 11–15 (04-save.md): the inputs stored by content, the outputs written into
//! `output/.partial/` from `input/`, verified, the record committed, the outputs moved in and the
//! views written. Nothing of the date changes before the commit; after it the batch is saved,
//! whatever a later move or view write says.

use std::path::PathBuf;

use packing_engine::batch_record::{BatchFacts, batch_saved};
use packing_engine::ids::BatchId;
use packing_engine::record::{BatchSaved, Finding, FindingKind, InputPdf, RecordBody};
use packing_engine::sorting::{SortedBatch, check_file_names};
use packing_pdf::summary_pdf::{SummaryFacts, write_summary};
use packing_pdf::worker::PdfWorker;
use packing_store::{
    DateFolder, Fold, Journal, StoreError, discard_partial, input_path, move_aside, move_forward,
    partial_dir, settle_partial, store_input, write_views,
};
use packing_verify::{BatchCheck, Findings, verify_batch};

use crate::written::{copy_pages, described, partial};
use crate::{BatchError, Draft};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveStage {
    CopyingInputs,
    Writing,
    Verifying,
    Saving,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SaveProgress {
    pub stage: SaveStage,
    pub done: usize,
    pub total: usize,
}

pub struct SaveRequest<'a> {
    pub worker: &'a PdfWorker,
    pub folder: &'a DateFolder,
    pub draft: &'a Draft,
    /// *Save anyway*: the verifier's findings go into the record, and the batch is saved.
    pub save_anyway: bool,
    /// The app version the records carry.
    pub app: &'a str,
    pub progress: &'a dyn Fn(SaveProgress),
}

#[derive(Debug)]
pub enum SaveOutcome {
    Saved(Saved),
    /// The verifier found something and *Save anyway* was not chosen: nothing was saved.
    Findings(Findings),
}

#[derive(Debug)]
pub struct Saved {
    pub record: BatchSaved,
    /// What failed after the commit (a move Windows refused, a view): the batch is saved, and the
    /// next open settles it.
    pub after_commit: Vec<StoreError>,
}

pub fn save(request: &SaveRequest<'_>, journal: &mut Journal) -> Result<SaveOutcome, BatchError> {
    let dir = request.folder.dir();
    let fold = journal.fold().clone();
    settle_partial(dir, &fold)?;
    let batch = number(request.draft, &fold)?;
    let progress = |stage, done, total| (request.progress)(SaveProgress { stage, done, total });

    progress(SaveStage::CopyingInputs, 0, request.draft.files.len());
    let inputs = store_inputs(request.draft, dir, &fold)?;
    let sorted = request.draft.sorted(batch);
    let mut files: Vec<String> = sorted.groups.iter().map(|g| g.file.clone()).collect();
    files.push(sorted.summary_file());
    check_file_names(&files).map_err(BatchError::FileName)?;

    let written = write_outputs(request, &inputs, &sorted, &fold, &progress);
    let mut record = match written {
        Ok(record) => record,
        Err(error) => {
            discard_partial(dir)?;
            return Err(error);
        }
    };

    progress(SaveStage::Verifying, 0, 1);
    let mut pending = journal.prepare(vec![RecordBody::BatchSaved(record.clone())], request.app)?;
    let findings = verify_batch(&BatchCheck {
        journal: &journal.bytes()?,
        append: &pending.bytes,
        input_dir: &dir.join("input"),
        output_dir: &partial_dir(dir),
    });
    if !findings.is_clean() {
        if !request.save_anyway {
            discard_partial(dir)?;
            return Ok(SaveOutcome::Findings(findings));
        }
        record
            .errors
            .extend(findings.errors.iter().map(|text| verifier(text)));
        record
            .warnings
            .extend(findings.warnings.iter().map(|text| verifier(text)));
        pending = journal.prepare(vec![RecordBody::BatchSaved(record.clone())], request.app)?;
    }

    progress(SaveStage::Saving, 0, 1);
    if record.amends {
        let old: Vec<String> = fold.live[&batch]
            .saved
            .outputs
            .iter()
            .map(|output| output.file.clone())
            .collect();
        if let Err(error) = move_aside(dir, &old) {
            discard_partial(dir)?;
            return Err(error.into());
        }
    }
    if let Err(error) = journal.append_pending(pending) {
        settle_partial(dir, journal.fold())?;
        return Err(error.into());
    }

    let mut after_commit = Vec::new();
    if let Err(error) = move_forward(dir) {
        after_commit.push(error);
    }
    if let Err(error) = write_views(dir, journal.fold()) {
        after_commit.push(error);
    }
    Ok(SaveOutcome::Saved(Saved {
        record,
        after_commit,
    }))
}

/// The draft's number on the date as it is now: an amend's batch must still be the newest.
fn number(draft: &Draft, fold: &Fold) -> Result<BatchId, BatchError> {
    match draft.redo {
        Some(batch) if fold.newest_live() != Some(batch) => Err(BatchError::NotNewest),
        _ => Ok(draft.batch(fold)),
    }
}

/// Step 11: each input in `input/` by its content, as it was read.
fn store_inputs(
    draft: &Draft,
    dir: &std::path::Path,
    fold: &Fold,
) -> Result<Vec<InputPdf>, BatchError> {
    let inputs = draft.inputs(fold);
    for (file, input) in draft.files.iter().zip(&inputs) {
        let stored = store_input(dir, &file.path)?;
        if stored.sha256 != input.sha256 || stored.size != input.size {
            return Err(BatchError::FileChanged {
                file: file.filename.clone(),
            });
        }
    }
    Ok(inputs)
}

/// Step 12: the group PDFs, in plan order, each order's pages copied from `input/`; the record;
/// then the summary PDF from it.
fn write_outputs(
    request: &SaveRequest<'_>,
    inputs: &[InputPdf],
    sorted: &SortedBatch,
    fold: &Fold,
    progress: &dyn Fn(SaveStage, usize, usize),
) -> Result<BatchSaved, BatchError> {
    let dir = request.folder.dir();
    let partial = partial(dir)?;
    let draft = request.draft;
    let sources: Vec<PathBuf> = inputs.iter().map(|i| input_path(dir, &i.sha256)).collect();
    let mut outputs = Vec::new();
    let total = sorted.groups.len() + 1;
    for (done, group) in sorted.groups.iter().enumerate() {
        progress(SaveStage::Writing, done, total);
        let pages: Vec<(usize, usize)> = group
            .contents
            .iter()
            .flat_map(|block| &block.orders)
            .flat_map(|&at| {
                let copy = &draft.copies[at];
                (copy.page_start..=copy.page_end).map(move |p| (copy.input, p as usize - 1))
            })
            .collect();
        let out = partial.join(&group.file);
        let count = pages.len();
        copy_pages(request.worker, sources.clone(), pages, out.clone())?;
        outputs.push(described(&out, Some(&group.code), count)?);
    }
    let mut record = batch_saved(BatchFacts {
        batch: sorted.batch,
        amends: draft.redo.is_some(),
        inputs,
        rule_name: &draft.rule_name,
        plan: &draft.plan,
        copies: &draft.copies,
        orders: &draft.orders,
        sorted,
        outputs,
    });

    progress(SaveStage::Writing, total - 1, total);
    let out = partial.join(sorted.summary_file());
    let target = out.clone();
    let (workspace, date, app) = (fold.workspace.clone(), fold.date, request.app.to_string());
    let printed = jiff::Zoned::now().strftime("%H:%M").to_string();
    let snapshot = record.clone();
    let couriers = draft.couriers.clone();
    let pages = request.worker.run(move |pdfium| {
        write_summary(
            pdfium,
            &target,
            &SummaryFacts {
                workspace: &workspace,
                date,
                record: &snapshot,
                printed: &printed,
                app: &app,
                couriers: &couriers,
            },
        )
    })??;
    record.outputs.push(described(&out, None, pages)?);
    Ok(record)
}

/// A finding of the verifier as the record keeps it on a batch saved anyway.
fn verifier(text: &str) -> Finding {
    Finding {
        kind: FindingKind::new("verifier"),
        pdf: None,
        page: None,
        order_id: None,
        text: text.to_string(),
    }
}
