//! The batch's own steps: read the chosen label files into a draft, map the plan into the packing
//! jobs, and save the batch atomically (07 › *What happens in one batch*). The checks that
//! surround them are in the modules beside this file.
//!
//! Everything PDF-related happens on the [`PdfWorker`] thread (PDFium is not thread-safe); the
//! mapping (a [`PlanResult`] into a [`PackingList`]) is plain functions so it is tested without a
//! PDF library.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Sender;

use packing_engine::day::{
    self, BatchWrite, DayError, DayState, Duplicate, FileEntry, Findings, Layout as EngineLayout,
    PackingListChoice, PdfEntry, SavedItem, SavedOrder, Scope, duplicates_warning,
};
use packing_engine::label::CourierTable;
use packing_engine::names;
use packing_engine::orders::{self, Line, LogoAnswer, ReadOrder, UnseenLogo, Warning};
use packing_engine::plan::{PickResult, PlanResult};
use packing_engine::verify::{self as engine_verify, Check, Expected};

use crate::packing_list::{self, PackingLine, PackingList, RunOrder, RunRows, SavedPdfSection};
use crate::worker::{PdfWorker, Progress, SaveProgress, SaveStage};

mod check;
mod error;
mod summary;

pub use check::{FileCheck, check_files, check_saved};
pub use error::BatchError;
pub use summary::{SummaryInput, summary_text};

use self::check::expected_lists;
use self::error::missing;

/// A batch that has been read but not saved (07 › *Words used in this document*): the files,
/// the orders left after the duplicate guard, the duplicates and the warnings.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Draft {
    pub files: Vec<FileCheck>,
    pub orders: Vec<ReadOrder>,
    pub duplicates: Vec<Duplicate>,
    pub warnings: Vec<String>,
    /// Pages read over all files.
    pub pages: usize,
    /// Orders read, before the duplicate guard.
    pub total_orders: usize,
    /// Pages that carry a packing slip.
    pub slip_pages: usize,
    /// The header-band images the courier data does not know, for the logo prompt (D-LABEL-4).
    /// Each is asked about once per batch and then leaves the list.
    pub unseen_logos: Vec<UnseenLogo>,
    /// The read warnings, kept structured so the logo prompt can recompute the ones that depend
    /// on the courier (D-LABEL-4); [`Draft::warnings`] is their display text plus the duplicate
    /// warning.
    pub read_warnings: Vec<Warning>,
}

impl Draft {
    /// Apply the owner's answer about one unseen logo (08 › *New batch* → *The logo prompt*,
    /// D-LABEL-4): the orders carrying the logo are named where the two clues agree
    /// ([`UnseenLogo::apply`] on `table`), the logo leaves `unseen_logos`, and the warnings that
    /// depend on the courier are recomputed (the unknown-courier line shrinks or goes). Returns
    /// how many orders the answer named; an unknown `hash` changes nothing.
    pub fn answer_logo(&mut self, hash: &str, answer: &LogoAnswer, table: &CourierTable) -> usize {
        let Some(index) = self
            .unseen_logos
            .iter()
            .position(|entry| entry.hash == hash)
        else {
            return 0;
        };
        let entry = self.unseen_logos.remove(index);
        let named = entry.apply(&mut self.orders, answer, table);
        self.refresh_courier_warning();
        named
    }

    /// Rebuild the warnings after the couriers changed: the unknown-courier line is recomputed
    /// from the orders (08 › *Messages* names them by tracking ID) and keeps its place among the
    /// read warnings; the `Qty Total` and tracking-ID lines do not depend on the courier.
    fn refresh_courier_warning(&mut self) {
        let mut warnings: Vec<Warning> = self
            .read_warnings
            .iter()
            .filter(|warning| !matches!(warning, Warning::UnknownCourier { .. }))
            .cloned()
            .collect();
        if let Some(unknown) = Warning::unknown_courier(&self.orders) {
            // Read puts the unknown-courier line after the `Qty Total` ones and before the
            // tracking-ID ones (`orders::read_batch`); keep that place.
            let at = warnings
                .iter()
                .position(|warning| matches!(warning, Warning::TrackingIdMissing { .. }))
                .unwrap_or(warnings.len());
            warnings.insert(at, unknown);
        }
        self.read_warnings = warnings;
        self.warnings = self
            .read_warnings
            .iter()
            .map(|warning| warning.to_string())
            .collect();
        if !self.duplicates.is_empty() {
            self.warnings.push(duplicates_warning(&self.duplicates));
        }
    }
}

/// Open the files on the worker, read every page with progress and a cancel flag, group the
/// pages into orders, then apply the duplicate guard (07 › *What happens in one batch* steps
/// 2-4). `state` supplies the duplicate guard (amend passes its state without the batch);
/// `couriers` is the data folder's courier data.
pub fn read(
    worker: &PdfWorker,
    files: &[FileCheck],
    state: &DayState,
    cancel: Arc<AtomicBool>,
    progress: Sender<Progress>,
    couriers: &CourierTable,
) -> Result<Draft, BatchError> {
    let paths: Vec<PathBuf> = files.iter().map(|file| file.path.clone()).collect();
    worker.open(&paths)?;
    let items = worker.read(cancel, progress, couriers.clone())?;
    let pages = items.len();

    let names: Vec<String> = files.iter().map(|file| file.name.clone()).collect();
    let name_refs: Vec<&str> = names.iter().map(String::as_str).collect();
    let reading = orders::read_batch(items, &name_refs, couriers)?;

    let slip_pages = reading
        .orders
        .iter()
        .filter(|order| order.has_slip)
        .map(|order| order.pages.len())
        .sum();

    let (kept, duplicates) = state.drop_saved(&reading.orders);
    if kept.is_empty() {
        if reading.orders.is_empty() {
            return Err(BatchError::Message("no orders were read".to_string()));
        }
        return Err(BatchError::AllSaved {
            orders: reading.orders.len(),
            batch: duplicates.first().map(|dup| dup.batch).unwrap_or(0),
        });
    }

    let mut warnings: Vec<String> = reading
        .warnings
        .iter()
        .map(|warning| warning.to_string())
        .collect();
    if !duplicates.is_empty() {
        warnings.push(duplicates_warning(&duplicates));
    }

    let total_orders = reading.orders.len();
    Ok(Draft {
        files: files.to_vec(),
        orders: kept,
        duplicates,
        warnings,
        pages,
        total_orders,
        slip_pages,
        unseen_logos: reading.unseen_logos,
        read_warnings: reading.warnings,
    })
}

/// What a save produced (07 › *Save*, *Check before saving*).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SaveResult {
    pub batch: i64,
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

/// Write a batch into `batch <b>.partial/`, check it, then rename it and add it to the state
/// (07 › *Save* step 6, *Check before saving*). The saved PDFs are the picks' pages in run order
/// (03 › *A saved PDF*), the packing lists follow the scope and layouts, `summary.txt` is
/// written and the label files are copied into `download/`. `set` is the name of the picks set
/// the batch was packed with, stored with it (04 › *Sets*).
///
/// The check before saving runs on every save (D-APP-2): the written files are read back and
/// compared with the plan. A finding stops the save — nothing is renamed, the temporary folder
/// is deleted, `state.json` is untouched and the findings are returned — unless `save_anyway` is
/// set, which renames the batch and stores the two lists with it. `progress` reports the writing
/// and the checking (the app turns it into a `save-progress` event).
#[allow(clippy::too_many_arguments)] // the batch's whole step-6 context, as `read` already takes
pub fn save(
    worker: &PdfWorker,
    draft: &Draft,
    plan: &PlanResult,
    packing: &PackingListChoice,
    set: &str,
    day: &str,
    day_dir: &Path,
    state: &mut DayState,
    save_anyway: bool,
    progress: Sender<SaveProgress>,
) -> Result<SaveResult, BatchError> {
    plan.check_file_names()
        .map_err(|error| BatchError::Message(error.to_string()))?;

    let batch = state.next_batch_number();
    let save_time = day::now_save_time();
    let printed = printed_time(&save_time);
    let index = order_index(draft);

    let mut pdf_pages: Vec<Vec<(usize, usize)>> = Vec::with_capacity(plan.picks.len());
    let mut sections: Vec<SavedPdfSection> = Vec::with_capacity(plan.picks.len());
    let mut saved: Vec<SavedOrder> = Vec::new();
    let mut pdfs: Vec<PdfEntry> = Vec::with_capacity(plan.picks.len());
    let mut page_total = 0usize;
    for pick in &plan.picks {
        let pages = pick_pages(pick, &index)?;
        page_total += pages.len();
        pdf_pages.push(pages);
        sections.push(pick_section(pick, &index)?);
        saved.extend(pick_saved(pick, &index, batch)?);
        pdfs.push(PdfEntry {
            number: pick.number,
            code: pick.code.clone(),
            name: pick.name.clone(),
            file: pick.file_name.clone(),
            orders: pick.orders.len(),
            runs: pick.runs.len(),
        });
    }

    let files: Vec<FileEntry> = draft
        .files
        .iter()
        .map(|file| FileEntry {
            name: file.name.clone(),
            size: file.size,
            xxh3: file.fingerprint.xxh3.clone(),
            pages: file.pages,
            legacy_sha256: None,
        })
        .collect();

    let jobs = packing_jobs(plan, &sections, packing, day, batch, &printed);
    let packing_lists: Vec<String> = jobs.iter().map(|job| job.name.clone()).collect();
    let expected = Expected {
        pdfs: engine_verify::expected_pdfs(plan, &draft.orders),
        lists: expected_lists(&jobs),
    };

    let write = BatchWrite {
        batch,
        save_time,
        set: set.to_string(),
        files,
        pdfs,
        packing_list: packing.clone(),
        warnings: draft.warnings.clone(),
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

    let folder = day::save_batch(day_dir, state, &write, move |temp| {
        let download = temp.join(names::DOWNLOAD_DIR);
        fs::create_dir_all(&download).map_err(|error| write_error(&download, error))?;
        let mut written = 0usize;
        for (pick, pages) in plan.picks.iter().zip(&pdf_pages) {
            let out = temp.join(&pick.file_name);
            worker
                .write_pages(pages, &out)
                .map_err(|error| DayError::Write {
                    folder: out,
                    message: error.to_string(),
                })?;
            written += pages.len();
            let _ = progress.send(SaveProgress {
                stage: SaveStage::Writing,
                page: written,
                pages: page_total,
            });
        }
        for file in &draft.files {
            let out = download.join(&file.name);
            fs::copy(&file.path, &out).map_err(|error| write_error(&out, error))?;
        }
        for job in &jobs {
            let out = temp.join(&job.name);
            worker
                .write_packing_list(job.layout, &out, &job.list)
                .map_err(|error| DayError::Write {
                    folder: out,
                    message: error.to_string(),
                })?;
        }

        // Check before saving: read the written files back and compare them with the plan.
        let written = crate::verify::read_back(worker, temp, &expected, progress.clone()).map_err(
            |error| DayError::Write {
                folder: temp.to_path_buf(),
                message: error.to_string(),
            },
        )?;
        let check = engine_verify::compare(&expected, &written);
        let stop = !save_anyway && !check.is_empty();
        *findings_fill.borrow_mut() = Some(check.clone());
        if stop {
            // Nothing is renamed: the temporary folder goes, `state.json` is untouched.
            stopped_fill.set(true);
            return Err(DayError::Write {
                folder: temp.to_path_buf(),
                message: "the check before saving stopped the batch".to_string(),
            });
        }

        let summary = summary_text(&SummaryInput {
            day,
            files: draft.files.len(),
            pages: draft.pages,
            orders: draft.total_orders,
            slip_pages: draft.slip_pages,
            picks: &plan.picks,
            duplicates: draft.duplicates.len(),
            warnings: &draft.warnings,
            check_errors: &check.error_texts(),
            check_warnings: &check.warning_texts(),
            day_dir,
            packing_lists: &packing_lists_fill,
            quiet_ids: false,
        });
        let out = temp.join(names::SUMMARY_FILE);
        fs::write(&out, &summary).map_err(|error| write_error(&out, error))?;
        *summary_fill.borrow_mut() = summary;
        Ok(Findings {
            errors: check.error_texts(),
            warnings: check.warning_texts(),
        })
    });

    let check = findings.borrow().clone().unwrap_or_default();
    let mut result_folder = day_dir.join(names::batch_folder(batch));
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
        pdf_files: plan.picks.iter().map(|p| p.file_name.clone()).collect(),
        packing_lists,
        summary: summary_slot.borrow().clone(),
        saved,
        errors: check.error_texts(),
        warnings: check.warning_texts(),
    })
}

fn write_error(path: &Path, error: std::io::Error) -> DayError {
    DayError::Write {
        folder: path.to_path_buf(),
        message: error.to_string(),
    }
}

/// One packing list to write: its file name, layout and built list.
struct PackingJob {
    name: String,
    layout: packing_list::Layout,
    list: PackingList,
}

fn packing_jobs(
    plan: &PlanResult,
    sections: &[SavedPdfSection],
    packing: &PackingListChoice,
    day: &str,
    batch: i64,
    printed: &str,
) -> Vec<PackingJob> {
    let several = packing.layouts.len() > 1;
    let mut jobs = Vec::new();
    let mut make = |number: Option<i64>, section: Vec<SavedPdfSection>| {
        for layout in &packing.layouts {
            let suffix = several.then(|| layout_name(*layout));
            let name = names::packing_list_file_name(number, suffix);
            jobs.push(PackingJob {
                name,
                layout: to_pdf_layout(*layout),
                list: PackingList {
                    day: day.to_string(),
                    batch: Some(batch as u32),
                    printed: printed.to_string(),
                    sections: section.clone(),
                },
            });
        }
    };
    match packing.scope {
        Scope::None => {}
        Scope::Whole => make(None, sections.to_vec()),
        Scope::PerPdf => {
            for (pick, section) in plan.picks.iter().zip(sections) {
                make(Some(pick.number), vec![section.clone()]);
            }
        }
    }
    jobs
}

/// A saved PDF's pages in run order (03 › *A saved PDF*).
type OrderIndex<'a> = HashMap<&'a str, &'a ReadOrder>;

fn order_index(draft: &Draft) -> OrderIndex<'_> {
    draft
        .orders
        .iter()
        .map(|order| (order.order.order_id.as_str(), order))
        .collect()
}

fn pick_pages(
    pick: &PickResult,
    index: &OrderIndex<'_>,
) -> Result<Vec<(usize, usize)>, BatchError> {
    let mut pages = Vec::new();
    for run in &pick.runs {
        for id in &run.orders {
            let order = index.get(id.as_str()).ok_or_else(|| missing(id))?;
            for page in &order.pages {
                pages.push((page.file, page.page));
            }
        }
    }
    Ok(pages)
}

fn packing_line(line: &Line) -> PackingLine {
    PackingLine {
        name: line.name.clone(),
        variation: line.variation.clone(),
        quantity: line.quantity,
        display_name: line.display_name.clone(),
    }
}

/// Map a pick and its orders into a packing-list section; the run structure is the pick's, so
/// the printed runs and the saved Pages match (03 › *Packing list (A4)*).
fn pick_section(pick: &PickResult, index: &OrderIndex<'_>) -> Result<SavedPdfSection, BatchError> {
    let mut runs = Vec::with_capacity(pick.runs.len());
    for run in &pick.runs {
        let mut orders = Vec::with_capacity(run.orders.len());
        for id in &run.orders {
            let order = index.get(id.as_str()).ok_or_else(|| missing(id))?;
            orders.push(RunOrder {
                tracking_id: order.order.tracking_id.clone(),
                lines: order.order.lines.iter().map(packing_line).collect(),
            });
        }
        runs.push(RunRows {
            number: run.number as u32,
            orders,
        });
    }
    Ok(SavedPdfSection {
        number: pick.number as u32,
        code: pick.code.clone(),
        name: pick.name.clone(),
        runs,
    })
}

fn pick_saved(
    pick: &PickResult,
    index: &OrderIndex<'_>,
    batch: i64,
) -> Result<Vec<SavedOrder>, BatchError> {
    let mut entries = Vec::with_capacity(pick.orders.len());
    for run in &pick.runs {
        for id in &run.orders {
            let order = index.get(id.as_str()).ok_or_else(|| missing(id))?;
            entries.push(SavedOrder {
                order_id: id.clone(),
                tracking_id: order.order.tracking_id.clone(),
                batch,
                pdf: pick.number,
                run: run.number,
                // The fields the Search panel finds the order by, written at save (07 ›
                // *`state.json`*): its lines and the order-level fields.
                items: order
                    .order
                    .lines
                    .iter()
                    .map(|line| SavedItem {
                        name: line.name.clone(),
                        variation: line.variation.clone(),
                        seller_sku: line.seller_sku.clone(),
                        quantity: line.quantity,
                    })
                    .collect(),
                courier: order.order.courier.clone(),
                total_quantity: order.order.total_quantity(),
                distinct_items: order.order.distinct_items(),
                ship_by: order.order.ship_by.map(|time| time.to_string()),
            });
        }
    }
    Ok(entries)
}

/// `HH:MM` of an ISO save time (`2026-10-07T07:40:12+07:00` → `07:40`).
fn printed_time(save_time: &str) -> String {
    save_time.chars().skip(11).take(5).collect()
}

fn layout_name(layout: EngineLayout) -> &'static str {
    match layout {
        EngineLayout::Full => "full",
        EngineLayout::Summary => "summary",
        EngineLayout::Pick => "pick",
    }
}

fn to_pdf_layout(layout: EngineLayout) -> packing_list::Layout {
    match layout {
        EngineLayout::Full => packing_list::Layout::Full,
        EngineLayout::Summary => packing_list::Layout::Summary,
        EngineLayout::Pick => packing_list::Layout::Pick,
    }
}

/// Parse a packing-list scope name (`--packing-list`).
pub fn parse_scope(name: &str) -> Option<Scope> {
    match name {
        "per-pdf" | "per_pdf" | "perpdf" => Some(Scope::PerPdf),
        "whole" => Some(Scope::Whole),
        "none" => Some(Scope::None),
        _ => None,
    }
}

/// Parse a packing-list layout name (`--layout`).
pub fn parse_layout(name: &str) -> Option<EngineLayout> {
    match name {
        "full" => Some(EngineLayout::Full),
        "summary" => Some(EngineLayout::Summary),
        "pick" => Some(EngineLayout::Pick),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
