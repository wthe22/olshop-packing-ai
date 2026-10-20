//! The batch's own steps: read the chosen label files into a draft, map the plan into the packing
//! jobs, and save the batch atomically (07 › *What happens in one batch*). The checks that
//! surround them are in the modules beside this file.
//!
//! Everything PDF-related happens on the [`PdfWorker`] thread (PDFium is not thread-safe); the
//! mapping (a [`PlanResult`] into a [`PackingList`]) is plain functions so it is tested without a
//! PDF library.

use std::cell::{Cell, RefCell};
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::mpsc::Sender;

use packing_engine::day::{
    self, BatchWrite, DayError, DayState, FileEntry, Findings, PackingListChoice, PdfEntry,
    SavedOrder,
};
use packing_engine::groupings::Grouping;
use packing_engine::ids::BatchNumber;
use packing_engine::names;
use packing_engine::plan::PlanResult;
use packing_engine::verify::{self as engine_verify, Check, Expected};

use crate::packing_list::SavedPdfSection;
use crate::worker::{PdfWorker, SaveProgress, SaveStage};

mod check;
mod error;
mod mapping;
mod read;
mod summary;

pub use check::{FileCheck, check_files, check_saved};
pub use error::BatchError;
pub use read::{Draft, ReadRequest, read};
pub use summary::{SummaryInput, summary_text};

use self::check::expected_lists;
use self::mapping::{
    ListRequest, OrderIndex, PackingJob, category_pages, category_saved, category_section,
    order_index, packing_jobs, printed_time,
};

/// What a save produced (07 › *Save*, *Check before saving*).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SaveResult {
    pub batch: BatchNumber,
    /// The batch folder once the save was renamed (07 › *Day folder*); the path the batch would
    /// have taken when the check stopped the save.
    pub folder: PathBuf,
    /// The written saved-PDF file names, in printing order.
    pub pdf_files: Vec<String>,
    /// The writing packing-list file names.
    pub packing_lists: Vec<String>,
    /// The batch summary text, also written to `summary.txt`.
    pub summary: String,
    /// Whether the batch was renamed into place and written into `state.json`; false when the
    /// check found something and *Save anyway* was not chosen (07 › *Check before saving*).
    pub saved: bool,
    /// The check's errors, as text; empty on a clean save.
    pub errors: Vec<String>,
    /// The check's warnings, as text.
    pub warnings: Vec<String>,
}

/// Everything one save of a batch needs: the draft and its plan, the packing-list choice and the
/// batch's grouping, the day, and where the writing and checking progress goes.
pub struct SaveRequest<'a> {
    pub worker: &'a PdfWorker,
    pub draft: &'a Draft,
    pub plan: &'a PlanResult,
    pub packing: &'a PackingListChoice,
    pub grouping: &'a Grouping,
    pub day: &'a str,
    pub day_dir: &'a Path,
    /// *Save anyway* was chosen: a finding then renames the batch and stores the two lists with
    /// it (07 › *Check before saving*).
    pub save_anyway: bool,
    /// The writing and the checking progress (the app turns it into a `save-progress` event).
    pub progress: Sender<SaveProgress>,
}

/// Write a batch into `batch <b>.partial/`, check it, then rename it and add it to the state
/// (07 › *Save* step 6, *Check before saving*). The saved PDFs are the picks' pages in run order
/// (03 › *A saved PDF*), the packing lists follow the scope and layouts, `summary.txt` is
/// written and the label files are copied into `download/`. `grouping` is the grouping the batch
/// was packed with, stored with it (04 › *Sets*).
///
/// The check before saving runs on every save (D-APP-2): the written files are read back and
/// compared with the plan. A finding stops the save — nothing is renamed, the temporary folder
/// is deleted, `state.json` is untouched and the findings are returned — unless `save_anyway` is
/// set, which renames the batch and stores the two lists with it.
pub fn save(request: &SaveRequest<'_>, state: &mut DayState) -> Result<SaveResult, BatchError> {
    request
        .plan
        .check_file_names()
        .map_err(|error| BatchError::Message(error.to_string()))?;

    let batch = state.next_batch_number();
    let save_time = day::now_save_time();
    let printed = printed_time(&save_time);
    let index = order_index(request.draft);

    let CategoryPieces {
        pdf_pages,
        sections,
        saved,
        pdfs,
        page_total,
    } = category_pieces(request.plan, &index, batch)?;
    let files = file_entries(request.draft);

    let jobs = packing_jobs(
        request.plan,
        &sections,
        ListRequest {
            packing: request.packing,
            day: request.day,
            batch: Some(batch),
            printed: &printed,
        },
    );
    let packing_lists: Vec<String> = jobs.iter().map(|job| job.name.clone()).collect();
    let expected = Expected {
        pdfs: engine_verify::expected_pdfs(request.plan, &request.draft.orders),
        lists: expected_lists(&jobs),
    };

    let write = BatchWrite {
        batch,
        save_time,
        grouping: request.grouping.clone(),
        files,
        pdfs,
        packing_list: request.packing.clone(),
        warnings: request.draft.warnings.clone(),
        saved,
    };

    let findings: Rc<RefCell<Option<Check>>> = Rc::new(RefCell::new(None));
    let stopped = Rc::new(Cell::new(false));
    let summary_slot = Rc::new(RefCell::new(String::new()));
    let packing_lists_fill = packing_lists.clone();
    let (findings_fill, stopped_fill, summary_fill) = (
        Rc::clone(&findings),
        Rc::clone(&stopped),
        Rc::clone(&summary_slot),
    );

    let folder = day::save_batch(request.day_dir, state, &write, move |temp| {
        write_batch_files(temp, request, &pdf_pages, page_total, &jobs)?;

        // Check before saving: read the written files back and compare them with the plan.
        let check = check_written(request, temp, &expected)?;
        *findings_fill.borrow_mut() = Some(check.clone());
        if !request.save_anyway && !check.is_empty() {
            // Nothing is renamed: the temporary folder goes, `state.json` is untouched.
            stopped_fill.set(true);
            return Err(DayError::Write {
                folder: temp.to_path_buf(),
                message: "the check before saving stopped the batch".to_string(),
            });
        }

        *summary_fill.borrow_mut() = write_summary(request, temp, &check, &packing_lists_fill)?;
        Ok(Findings {
            errors: check.error_texts(),
            warnings: check.warning_texts(),
        })
    });

    let check = findings.borrow().clone().unwrap_or_default();
    let mut result_folder = request.day_dir.join(names::batch_folder(batch));
    let saved = match folder {
        Ok(folder) => {
            result_folder = folder;
            true
        }
        Err(error) => {
            if !stopped.get() {
                return Err(BatchError::Day(error));
            }
            false
        }
    };

    Ok(SaveResult {
        batch,
        folder: result_folder,
        pdf_files: request
            .plan
            .categories
            .iter()
            .map(|p| p.file_name.clone())
            .collect(),
        packing_lists,
        summary: summary_slot.borrow().clone(),
        saved,
        errors: check.error_texts(),
        warnings: check.warning_texts(),
    })
}

/// The pieces one save builds from the plan before the batch is written: the saved PDFs' pages in
/// run order, the packing-list sections, the state's saved orders and saved-PDF entries, and the
/// total page count.
struct CategoryPieces {
    pdf_pages: Vec<Vec<(usize, usize)>>,
    sections: Vec<SavedPdfSection>,
    saved: Vec<SavedOrder>,
    pdfs: Vec<PdfEntry>,
    page_total: usize,
}

fn category_pieces(
    plan: &PlanResult,
    index: &OrderIndex<'_>,
    batch: BatchNumber,
) -> Result<CategoryPieces, BatchError> {
    let mut pdf_pages: Vec<Vec<(usize, usize)>> = Vec::with_capacity(plan.categories.len());
    let mut sections: Vec<SavedPdfSection> = Vec::with_capacity(plan.categories.len());
    let mut saved: Vec<SavedOrder> = Vec::new();
    let mut pdfs: Vec<PdfEntry> = Vec::with_capacity(plan.categories.len());
    let mut page_total = 0usize;
    for category in &plan.categories {
        let pages = category_pages(category, index)?;
        page_total += pages.len();
        pdf_pages.push(pages);
        sections.push(category_section(category, index)?);
        saved.extend(category_saved(category, index, batch)?);
        pdfs.push(PdfEntry {
            number: category.number,
            code: category.code.clone(),
            name: category.name.clone(),
            file: category.file_name.clone(),
            orders: category.orders.len(),
            runs: category.runs.len(),
        });
    }
    Ok(CategoryPieces {
        pdf_pages,
        sections,
        saved,
        pdfs,
        page_total,
    })
}

/// The label files of the batch as the state holds them.
fn file_entries(draft: &Draft) -> Vec<FileEntry> {
    draft
        .files
        .iter()
        .map(|file| FileEntry {
            name: file.name.clone(),
            size: file.size,
            xxh3: file.fingerprint.xxh3.clone(),
            pages: file.pages,
            legacy_sha256: None,
        })
        .collect()
}

/// Write the saved PDFs, the copied label files and the packing lists into `temp`.
fn write_batch_files(
    temp: &Path,
    request: &SaveRequest<'_>,
    pdf_pages: &[Vec<(usize, usize)>],
    page_total: usize,
    jobs: &[PackingJob],
) -> Result<(), DayError> {
    let download = temp.join(names::DOWNLOAD_DIR);
    fs::create_dir_all(&download).map_err(|error| write_error(&download, error))?;
    let mut written = 0usize;
    for (category, pages) in request.plan.categories.iter().zip(pdf_pages) {
        let out = temp.join(&category.file_name);
        request
            .worker
            .write_pages(pages, &out)
            .map_err(|error| DayError::Write {
                folder: out,
                message: error.to_string(),
            })?;
        written += pages.len();
        let _ = request.progress.send(SaveProgress {
            stage: SaveStage::Writing,
            page: written,
            pages: page_total,
        });
    }
    for file in &request.draft.files {
        let out = download.join(&file.name);
        fs::copy(&file.path, &out).map_err(|error| write_error(&out, error))?;
    }
    for job in jobs {
        let out = temp.join(&job.name);
        request
            .worker
            .write_packing_list(job.layout, &out, &job.list)
            .map_err(|error| DayError::Write {
                folder: out,
                message: error.to_string(),
            })?;
    }
    Ok(())
}

/// Read the written files of `temp` back and compare them with the plan
/// (07 › *Check before saving*).
fn check_written(
    request: &SaveRequest<'_>,
    temp: &Path,
    expected: &Expected,
) -> Result<Check, DayError> {
    let written =
        crate::verify::read_back(request.worker, temp, expected, request.progress.clone())
            .map_err(|error| DayError::Write {
                folder: temp.to_path_buf(),
                message: error.to_string(),
            })?;
    Ok(engine_verify::compare(expected, &written))
}

/// Write the batch summary into `temp` and return its text.
fn write_summary(
    request: &SaveRequest<'_>,
    temp: &Path,
    check: &Check,
    packing_lists: &[String],
) -> Result<String, DayError> {
    let summary = summary_text(&SummaryInput {
        day: request.day,
        files: request.draft.files.len(),
        pages: request.draft.pages,
        orders: request.draft.total_orders,
        slip_pages: request.draft.slip_pages,
        categories: &request.plan.categories,
        duplicates: request.draft.duplicates.len(),
        warnings: &request.draft.warnings,
        check_errors: &check.error_texts(),
        check_warnings: &check.warning_texts(),
        day_dir: request.day_dir,
        packing_lists,
        quiet_ids: false,
    });
    let out = temp.join(names::SUMMARY_FILE);
    fs::write(&out, &summary).map_err(|error| write_error(&out, error))?;
    Ok(summary)
}

fn write_error(path: &Path, error: std::io::Error) -> DayError {
    DayError::Write {
        folder: path.to_path_buf(),
        message: error.to_string(),
    }
}

#[cfg(test)]
mod tests;
