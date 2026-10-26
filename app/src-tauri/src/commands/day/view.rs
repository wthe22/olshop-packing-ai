//! The Day screen's data from a date's fold: the live batches, newest first, each with its group
//! PDFs, Printed ticks and summary PDF, the copies left out, its warnings and errors; the outputs
//! the open-time checks found missing, the inputs that cannot be used, and a damaged journal.

use std::collections::BTreeSet;
use std::path::Path;

use packing_engine::ids::{BatchId, PdfId};
use packing_engine::record::{BatchSaved, LeftOutReason};
use packing_engine::sorting::{REST_CODE, REST_NAME};
use packing_store::{InputState, LiveBatch, OutputState};
use serde::Serialize;

use crate::OpenDay;

#[derive(Serialize)]
pub struct GroupPdfView {
    /// The label group's code: the Printed tick names it.
    group: String,
    name: String,
    file: String,
    orders: usize,
    contents: usize,
    printed: bool,
    problem: Option<FileProblem>,
}

/// What the open-time check found of an output: its card offers **Restore**.
#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FileProblem {
    Missing,
    /// Its size is not the one its record names.
    Changed,
}

/// One copy left out of a batch: where it is, and why.
#[derive(Serialize)]
pub struct LeftOutView {
    order_id: String,
    input: String,
    page_start: u16,
    page_end: u16,
    #[serde(flatten)]
    because: LeftOutBecause,
}

#[derive(Serialize)]
#[serde(tag = "reason", rename_all = "snake_case")]
pub enum LeftOutBecause {
    /// A repeat in this batch: the copy written instead.
    Repeat {
        kept_input: String,
        kept_page: u16,
    },
    SavedEarlier {
        batch: BatchId,
    },
}

#[derive(Serialize)]
pub struct BatchView {
    batch: BatchId,
    /// `HH:MM` of its newest save.
    time: String,
    /// Saved again by an amend.
    amended: bool,
    sort_rule: String,
    files: usize,
    orders: usize,
    left_out: Vec<LeftOutView>,
    pdfs: Vec<GroupPdfView>,
    summary: String,
    summary_problem: Option<FileProblem>,
    /// The read's warnings, then the verifier's on a batch saved anyway.
    warnings: Vec<String>,
    /// The verifier's errors: only on a batch saved anyway.
    errors: Vec<String>,
    /// `output/`, for *Open* and *Open folder*.
    folder: String,
}

#[derive(Serialize)]
pub struct Totals {
    batches: usize,
    orders: usize,
    pdfs: usize,
}

#[derive(Serialize)]
pub struct DayOverview {
    day: String,
    workspace: String,
    batches: Vec<BatchView>,
    totals: Totals,
    /// What keeps a part of the date from being used: a damaged journal, an input missing or
    /// changed.
    problems: Vec<String>,
    /// The data folder also holds a v0.1 data folder's files, which are not read.
    old_data: bool,
    /// The reprints made on this date.
    reprints: Vec<ReprintRow>,
}

#[derive(Serialize)]
pub struct ReprintRow {
    number: u8,
    /// `HH:MM` when it was made.
    time: String,
    file: String,
    orders: usize,
    pages: u16,
    path: String,
}

pub(super) fn overview(day: &OpenDay, old_data: bool) -> DayOverview {
    let fold = day.journal.fold();
    let output = day.folder.dir().join("output");
    let batches: Vec<BatchView> = fold
        .live
        .iter()
        .rev()
        .map(|(batch, live)| batch_view(day, *batch, live, &output))
        .collect();
    let mut problems = Vec::new();
    if let Some(damage) = day.journal.damage() {
        problems.push(format!(
            "The journal of this date is damaged at {damage}. The date can be viewed; nothing is \
             written to it."
        ));
    }
    for input in &day.inputs {
        let state = match input.state {
            InputState::Missing => "is missing",
            InputState::Changed => "has changed",
        };
        problems.push(format!(
            "{} of batch {} {state} in the date's input folder. An amend cannot use it.",
            input.filename, input.batch
        ));
    }
    DayOverview {
        day: fold.date.to_string(),
        workspace: fold.workspace.clone(),
        totals: Totals {
            batches: batches.len(),
            orders: batches.iter().map(|b| b.orders).sum(),
            pdfs: batches.iter().map(|b| b.pdfs.len()).sum(),
        },
        batches,
        problems,
        old_data,
        reprints: fold
            .reprints
            .iter()
            .map(|reprint| ReprintRow {
                number: reprint.made.number,
                time: reprint.create_time.zoned().strftime("%H:%M").to_string(),
                file: reprint.made.file.clone(),
                orders: reprint.made.orders.len(),
                pages: reprint.made.pages,
                path: output.join(&reprint.made.file).display().to_string(),
            })
            .collect(),
    }
}

fn batch_view(day: &OpenDay, batch: BatchId, live: &LiveBatch, output: &Path) -> BatchView {
    let saved = &live.saved;
    let problem = |file: &str| {
        day.outputs
            .iter()
            .find(|problem| problem.batch == batch && problem.file == file)
            .map(|problem| match problem.state {
                OutputState::Missing => FileProblem::Missing,
                OutputState::Changed => FileProblem::Changed,
            })
    };
    let pdfs = saved
        .outputs
        .iter()
        .filter_map(|file| {
            let group = file.group.as_ref()?;
            let orders: Vec<u16> = saved
                .orders
                .iter()
                .filter(|order| &order.group == group)
                .map(|order| order.content)
                .collect();
            let contents: BTreeSet<u16> = orders.iter().copied().collect();
            let name = if group == REST_CODE {
                REST_NAME.to_string()
            } else {
                saved
                    .sort_rule
                    .label_groups
                    .iter()
                    .find(|rule| &rule.code == group)
                    .map_or_else(String::new, |rule| rule.name.clone())
            };
            Some(GroupPdfView {
                group: group.clone(),
                name,
                file: file.file.clone(),
                orders: orders.len(),
                contents: contents.len(),
                printed: live.printed.contains(group),
                problem: problem(&file.file),
            })
        })
        .collect();
    let summary = saved
        .outputs
        .iter()
        .find(|file| file.group.is_none())
        .map_or_else(String::new, |file| file.file.clone());
    BatchView {
        batch,
        time: live.modify_time.zoned().strftime("%H:%M").to_string(),
        amended: saved.amends,
        sort_rule: saved.sort_rule.name.clone(),
        files: saved.inputs.len(),
        orders: saved.orders.len(),
        left_out: left_out(saved),
        pdfs,
        summary_problem: problem(&summary),
        summary,
        warnings: saved.warnings.iter().map(|f| f.text.clone()).collect(),
        errors: saved.errors.iter().map(|f| f.text.clone()).collect(),
        folder: output.display().to_string(),
    }
}

fn left_out(saved: &BatchSaved) -> Vec<LeftOutView> {
    let input = |pdf: &PdfId| {
        saved
            .inputs
            .iter()
            .find(|input| &input.pdf == pdf)
            .map_or_else(String::new, |input| input.filename.clone())
    };
    saved
        .left_out
        .iter()
        .map(|copy| LeftOutView {
            order_id: copy.order_id.to_string(),
            input: input(&copy.pdf),
            page_start: copy.page_start,
            page_end: copy.page_end,
            because: match &copy.reason {
                LeftOutReason::Repeat { pdf, page_start } => LeftOutBecause::Repeat {
                    kept_input: input(pdf),
                    kept_page: *page_start,
                },
                LeftOutReason::SavedEarlier { batch } => {
                    LeftOutBecause::SavedEarlier { batch: *batch }
                }
            },
        })
        .collect()
}

/// Whether the data folder holds what a v0.1 data folder holds (workspaces.md).
pub(super) fn holds_v01_data(data_dir: &Path) -> bool {
    [
        "labels",
        "state.json",
        "couriers.toml",
        "categories.toml",
        "categories",
    ]
    .iter()
    .any(|name| data_dir.join(name).exists())
}
