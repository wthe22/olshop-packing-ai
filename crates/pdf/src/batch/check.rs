//! Checking a batch's files: the chosen label files before a read, and the written files
//! against the plan after a save.

use std::path::{Path, PathBuf};
use std::sync::mpsc;

use packing_engine::day::{self, DayState, Fingerprint, PackingListChoice};
use packing_engine::ids::BatchNumber;
use packing_engine::plan::PlanResult;
use packing_engine::verify::{self as engine_verify, Check, Expected, ExpectedList};

use crate::worker::PdfWorker;

use super::Draft;
use super::error::BatchError;
use super::{PackingJob, order_index, packing_jobs, pick_section};

/// One chosen label file: what 07 › *New batch* shows before reading (page count, fingerprint,
/// `used in batch n`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileCheck {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub pages: usize,
    pub fingerprint: Fingerprint,
    /// The batch that read this same file earlier today, if any.
    pub used_in_batch: Option<BatchNumber>,
}

/// Read each chosen file's page count and fingerprint and check it against the day's state
/// (07 › *New batch*; 08 › *Messages*: a file that cannot be opened stops the batch).
pub fn check_files(
    worker: &PdfWorker,
    paths: &[PathBuf],
    state: &DayState,
) -> Result<Vec<FileCheck>, BatchError> {
    let counts = worker.open(paths)?;
    let mut checks = Vec::with_capacity(paths.len());
    for (path, pages) in paths.iter().zip(counts) {
        let fingerprint = day::fingerprint(path)?;
        let used_in_batch = state.used_in_batch(&fingerprint);
        checks.push(FileCheck {
            name: file_name(path),
            path: path.clone(),
            size: fingerprint.size,
            pages,
            fingerprint,
            used_in_batch,
        });
    }
    Ok(checks)
}

/// The check's plan side (07 › *Check before saving*): every saved PDF's page sequence and every
/// packing list's counts.
pub(super) fn expected_lists(jobs: &[PackingJob]) -> Vec<ExpectedList> {
    jobs.iter()
        .map(|job| {
            let (orders, units, runs) = job.list.counts();
            ExpectedList {
                file: job.name.clone(),
                orders,
                units,
                runs,
            }
        })
        .collect()
}

/// Read the files written in `folder` back and compare them with the plan (07 › *Check before
/// saving*), saving nothing. Public so the check can be run on its own.
pub fn check_saved(
    worker: &PdfWorker,
    folder: &Path,
    draft: &Draft,
    plan: &PlanResult,
    packing: &PackingListChoice,
) -> Result<Check, BatchError> {
    let index = order_index(draft);
    let mut sections = Vec::with_capacity(plan.picks.len());
    for pick in &plan.picks {
        sections.push(pick_section(pick, &index)?);
    }
    // Only the names and counts are compared, so the list belongs to no batch and no day.
    let jobs = packing_jobs(plan, &sections, packing, "", None, "");
    let expected = Expected {
        pdfs: engine_verify::expected_pdfs(plan, &draft.orders),
        lists: expected_lists(&jobs),
    };
    let (progress, _rx) = mpsc::channel();
    let written = crate::verify::read_back(worker, folder, &expected, progress)?;
    Ok(engine_verify::compare(&expected, &written))
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned())
}
