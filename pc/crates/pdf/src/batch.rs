//! The batch steps shared by the developer CLI (now) and the Tauri app (later), so the app
//! does not duplicate them: check the chosen files, read them with progress, the duplicate
//! guard, and save a batch atomically (07 › *What happens in one batch*).
//!
//! Everything PDF-related happens on the [`PdfWorker`] thread (PDFium is not thread-safe); the
//! mapping (a [`PlanResult`] into a [`PackingList`]) and the summary text are plain functions
//! so they are tested without a PDF library.

use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Sender;

use packing_engine::day::{
    self, BatchWrite, DayError, DayState, Duplicate, FileEntry, Fingerprint,
    Layout as EngineLayout, PackingListChoice, PdfEntry, SavedOrder, Scope, duplicates_warning,
};
use packing_engine::names;
use packing_engine::orders::{self, Line, ReadError, ReadOrder};
use packing_engine::plan::{PickResult, PlanResult, REST_CODE};

use crate::PdfError;
use crate::packing_list::{self, PackingLine, PackingList, RunOrder, RunRows, SavedPdfSection};
use crate::worker::{PdfWorker, Progress};

/// A batch step failed. The stop texts are those of 08 › *Messages*.
#[derive(Debug)]
pub enum BatchError {
    /// A PDF job failed (a file cannot be opened, a write failed).
    Pdf(PdfError),
    /// Saving into the day folder failed.
    Day(DayError),
    /// Reading stopped the batch (no Order ID on a first page, a page with no packing slip).
    Read(ReadError),
    /// Every order was already saved today (08 › *Messages*).
    AllSaved { orders: usize, batch: i64 },
    /// A plain message.
    Message(String),
}

impl fmt::Display for BatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BatchError::Pdf(error) => error.fmt(f),
            BatchError::Day(error) => error.fmt(f),
            BatchError::Read(error) => error.fmt(f),
            BatchError::AllSaved { orders, batch } => write!(
                f,
                "All {orders} orders were already saved today (batch {batch}). \
                 These labels were prepared before; nothing to do."
            ),
            BatchError::Message(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for BatchError {}

impl From<PdfError> for BatchError {
    fn from(error: PdfError) -> Self {
        BatchError::Pdf(error)
    }
}

impl From<DayError> for BatchError {
    fn from(error: DayError) -> Self {
        BatchError::Day(error)
    }
}

impl From<ReadError> for BatchError {
    fn from(error: ReadError) -> Self {
        BatchError::Read(error)
    }
}

fn missing(id: &str) -> BatchError {
    BatchError::Message(format!("order {id} is not in the draft"))
}

// ---------------------------------------------------------------------- check files

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
    pub used_in_batch: Option<i64>,
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

// ---------------------------------------------------------------------------- read

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
}

/// Open the files on the worker, read every page with progress and a cancel flag, group the
/// pages into orders, then apply the duplicate guard (07 › *What happens in one batch* steps
/// 2-4). `state` supplies the duplicate guard (amend passes its state without the batch).
pub fn read(
    worker: &PdfWorker,
    files: &[FileCheck],
    state: &DayState,
    cancel: Arc<AtomicBool>,
    progress: Sender<Progress>,
) -> Result<Draft, BatchError> {
    let paths: Vec<PathBuf> = files.iter().map(|file| file.path.clone()).collect();
    worker.open(&paths)?;
    let items = worker.read(cancel, progress)?;
    let pages = items.len();

    let names: Vec<String> = files.iter().map(|file| file.name.clone()).collect();
    let name_refs: Vec<&str> = names.iter().map(String::as_str).collect();
    let reading = orders::read_batch(items, &name_refs)?;

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

    Ok(Draft {
        files: files.to_vec(),
        orders: kept,
        duplicates,
        warnings,
        pages,
        total_orders: reading.orders.len(),
        slip_pages,
    })
}

// ---------------------------------------------------------------------------- save

/// What a finished save wrote (07 › *Save*).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SaveResult {
    pub batch: i64,
    pub folder: PathBuf,
    /// The written saved-PDF file names, in printing order.
    pub pdf_files: Vec<String>,
    /// The writing packing-list file names.
    pub packing_lists: Vec<String>,
    /// The batch summary text, also written to `summary.txt`.
    pub summary: String,
}

/// Write a batch into `batch <b>.partial/`, then rename it and add it to the state
/// (07 › *Save* step 6). The saved PDFs are the picks' pages in run order (03 › *A saved PDF*),
/// the packing lists follow the scope and layouts, `summary.txt` is written and the label files
/// are copied into `download/`. Nothing is written when any step fails.
pub fn save(
    worker: &PdfWorker,
    draft: &Draft,
    plan: &PlanResult,
    packing: &PackingListChoice,
    day: &str,
    day_dir: &Path,
    state: &mut DayState,
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
    for pick in &plan.picks {
        pdf_pages.push(pick_pages(pick, &index)?);
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
            sha256: file.fingerprint.sha256.clone(),
            pages: file.pages,
        })
        .collect();

    let jobs = packing_jobs(plan, &sections, packing, day, batch, &printed);
    let packing_lists: Vec<String> = jobs.iter().map(|job| job.name.clone()).collect();

    let summary = summary_text(&SummaryInput {
        day,
        files: draft.files.len(),
        pages: draft.pages,
        orders: draft.total_orders,
        slip_pages: draft.slip_pages,
        picks: &plan.picks,
        duplicates: draft.duplicates.len(),
        warnings: &draft.warnings,
        day_dir,
        packing_lists: &packing_lists,
        quiet_ids: false,
    });

    let write = BatchWrite {
        batch,
        save_time,
        files,
        pdfs,
        packing_list: packing.clone(),
        saved,
    };

    let summary_for_file = summary.clone();
    let folder = day::save_batch(day_dir, state, &write, move |temp| {
        let download = temp.join(names::DOWNLOAD_DIR);
        fs::create_dir_all(&download).map_err(|error| write_error(&download, error))?;
        for (pick, pages) in plan.picks.iter().zip(&pdf_pages) {
            let out = temp.join(&pick.file_name);
            worker
                .write_pages(pages, &out)
                .map_err(|error| DayError::Write {
                    folder: out,
                    message: error.to_string(),
                })?;
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
        let out = temp.join(names::SUMMARY_FILE);
        fs::write(&out, &summary_for_file).map_err(|error| write_error(&out, error))?;
        Ok(())
    })?;

    Ok(SaveResult {
        batch,
        folder,
        pdf_files: plan.picks.iter().map(|p| p.file_name.clone()).collect(),
        packing_lists,
        summary,
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

// ------------------------------------------------------------------------- summary

/// The pieces the batch summary is built from (03 › *Screen summary*).
pub struct SummaryInput<'a> {
    pub day: &'a str,
    pub files: usize,
    pub pages: usize,
    pub orders: usize,
    pub slip_pages: usize,
    pub picks: &'a [PickResult],
    pub duplicates: usize,
    pub warnings: &'a [String],
    pub day_dir: &'a Path,
    pub packing_lists: &'a [String],
    /// The developer's `--quiet-ids`: keep the warning count, drop the detail lines that name
    /// orders and tracking IDs.
    pub quiet_ids: bool,
}

/// The batch summary (03 › *Screen summary*), the text printed and written to `summary.txt`.
pub fn summary_text(input: &SummaryInput<'_>) -> String {
    let file_word = if input.files == 1 {
        "label file"
    } else {
        "label files"
    };
    let mut lines = vec![format!(
        "Day {} · {} {} · no CSV",
        input.day, input.files, file_word
    )];
    lines.push(format!(
        "  Pages: {} read · {} orders",
        input.pages, input.orders
    ));
    let items = if input.slip_pages > 0 {
        format!("slips ({} pages)", input.slip_pages)
    } else {
        "none".to_string()
    };
    lines.push(format!("  Item data: {items}"));
    for pick in input.picks {
        let kind = if pick.code == REST_CODE {
            "Rest"
        } else {
            "Pick"
        };
        lines.push(format!(
            "  {kind} {}  {}: {} orders, {} runs → {}",
            pick.code,
            pick.name,
            pick.orders.len(),
            pick.runs.len(),
            pick.file_name
        ));
    }
    lines.push(format!("  Duplicates: {}", input.duplicates));
    if input.warnings.is_empty() {
        lines.push("  Warnings: none".to_string());
    } else {
        lines.push(format!("  Warnings: {}", input.warnings.len()));
        if !input.quiet_ids {
            for warning in input.warnings {
                lines.push(format!("    - {warning}"));
            }
        }
    }
    let pdf_word = if input.picks.len() == 1 {
        "saved PDF"
    } else {
        "saved PDFs"
    };
    let packing = if input.packing_lists.is_empty() {
        "no packing list".to_string()
    } else {
        input.packing_lists.join(", ")
    };
    lines.push(format!(
        "Written: {}/  ({} {}, {})",
        input.day_dir.display(),
        input.picks.len(),
        pdf_word,
        packing
    ));
    lines.join("\n") + "\n"
}

// --------------------------------------------------------------------------- helpers

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned())
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
mod tests {
    use super::*;
    use packing_engine::orders::Order;
    use packing_engine::plan::{PickResult, RunResult};

    fn line(name: &str, variation: &str, quantity: i64, display: &str) -> Line {
        Line {
            name: name.to_string(),
            variation: variation.to_string(),
            seller_sku: String::new(),
            quantity,
            display_name: display.to_string(),
        }
    }

    fn read_order(id: &str, tracking: &str, lines: Vec<Line>) -> ReadOrder {
        ReadOrder {
            order: Order {
                order_id: id.to_string(),
                tracking_id: tracking.to_string(),
                courier: "J&T Express".to_string(),
                ship_by: None,
                lines,
                customer_message: String::new(),
            },
            pages: vec![packing_engine::orders::PageRef { file: 0, page: 0 }],
            position: 1,
            has_slip: true,
            qty_total: None,
        }
    }

    fn pick() -> (PickResult, Vec<ReadOrder>) {
        let orders = vec![
            read_order(
                "580000000000000001",
                "JY0000000101",
                vec![line("Sepatu", "honda", 1, "Sepatu — honda")],
            ),
            read_order(
                "580000000000000002",
                "JY0000000102",
                vec![line("Sepatu", "honda", 1, "Sepatu — honda")],
            ),
        ];
        let pick = PickResult {
            number: 1,
            code: "A".to_string(),
            name: "Sepatu".to_string(),
            orders: vec![
                "580000000000000001".to_string(),
                "580000000000000002".to_string(),
            ],
            runs: vec![RunResult {
                number: 1,
                orders: vec![
                    "580000000000000001".to_string(),
                    "580000000000000002".to_string(),
                ],
            }],
            pages: 2,
            file_name: "1 A Sepatu ×2.pdf".to_string(),
        };
        (pick, orders)
    }

    #[test]
    fn pick_section_maps_runs_and_lines() {
        let (pick, orders) = pick();
        let draft = Draft {
            files: Vec::new(),
            orders,
            duplicates: Vec::new(),
            warnings: Vec::new(),
            pages: 2,
            total_orders: 2,
            slip_pages: 2,
        };
        let index = order_index(&draft);
        let section = pick_section(&pick, &index).expect("section");

        assert_eq!(section.number, 1);
        assert_eq!(section.code, "A");
        assert_eq!(section.name, "Sepatu");
        assert_eq!(section.runs.len(), 1);
        assert_eq!(section.runs[0].number, 1);
        assert_eq!(section.runs[0].orders.len(), 2);
        assert_eq!(section.runs[0].orders[0].tracking_id, "JY0000000101");
        assert_eq!(section.runs[0].orders[0].lines[0].quantity, 1);
        assert_eq!(
            section.runs[0].orders[0].lines[0].display_name,
            "Sepatu — honda"
        );
    }

    #[test]
    fn pick_pages_and_saved_follow_the_runs() {
        let (pick, orders) = pick();
        let draft = Draft {
            files: Vec::new(),
            orders,
            duplicates: Vec::new(),
            warnings: Vec::new(),
            pages: 2,
            total_orders: 2,
            slip_pages: 2,
        };
        let index = order_index(&draft);

        assert_eq!(
            pick_pages(&pick, &index).expect("pages"),
            vec![(0, 0), (0, 0)]
        );
        let saved = pick_saved(&pick, &index, 1).expect("saved");
        assert_eq!(saved.len(), 2);
        assert_eq!(saved[0].pdf, 1);
        assert_eq!(saved[0].run, 1);
        assert_eq!(saved[1].order_id, "580000000000000002");
    }

    #[test]
    fn summary_text_lists_the_picks_and_the_written_folder() {
        let (pick, orders) = pick();
        let rest = PickResult {
            number: 2,
            code: REST_CODE.to_string(),
            name: "Uncategorised".to_string(),
            orders: Vec::new(),
            runs: Vec::new(),
            pages: 0,
            file_name: "2 - Uncategorised ×0.pdf".to_string(),
        };
        let picks = vec![pick, rest];
        let warnings = vec!["no courier could be deduced for: 580000000000000009".to_string()];
        let packing_lists = vec!["packing-list.pdf".to_string()];
        let day_dir = Path::new("data/labels/2026-10-07");

        let text = summary_text(&SummaryInput {
            day: "2026-10-07",
            files: 1,
            pages: 2,
            orders: 2,
            slip_pages: 2,
            picks: &picks,
            duplicates: 1,
            warnings: &warnings,
            day_dir,
            packing_lists: &packing_lists,
            quiet_ids: false,
        });

        assert!(text.starts_with("Day 2026-10-07 · 1 label file · no CSV\n"));
        assert!(text.contains("  Pages: 2 read · 2 orders\n"));
        assert!(text.contains("  Item data: slips (2 pages)\n"));
        assert!(text.contains("  Pick A  Sepatu: 2 orders, 1 runs → 1 A Sepatu ×2.pdf\n"));
        assert!(
            text.contains("  Rest ?  Uncategorised: 0 orders, 0 runs → 2 - Uncategorised ×0.pdf\n")
        );
        assert!(text.contains("  Duplicates: 1\n"));
        assert!(text.contains("  Warnings: 1\n"));
        assert!(text.contains("    - no courier could be deduced for: 580000000000000009\n"));
        assert!(
            text.contains("Written: data/labels/2026-10-07/  (2 saved PDFs, packing-list.pdf)\n")
        );
        // Keeps the orders variable referenced even though it is only used for the index above.
        assert_eq!(orders.len(), 2);
    }

    #[test]
    fn quiet_summary_hides_the_id_detail_lines() {
        let warnings = vec!["no tracking ID for: 580000000000000004".to_string()];
        let text = summary_text(&SummaryInput {
            day: "2026-10-07",
            files: 1,
            pages: 1,
            orders: 1,
            slip_pages: 1,
            picks: &[],
            duplicates: 0,
            warnings: &warnings,
            day_dir: Path::new("d"),
            packing_lists: &[],
            quiet_ids: true,
        });
        assert!(text.contains("  Warnings: 1\n"));
        assert!(!text.contains("580000000000000004"), "no IDs: {text}");
        assert!(text.contains("Written: d/  (0 saved PDFs, no packing list)\n"));
    }

    #[test]
    fn parse_helpers_accept_the_documented_names() {
        assert_eq!(parse_scope("whole"), Some(Scope::Whole));
        assert_eq!(parse_scope("per-pdf"), Some(Scope::PerPdf));
        assert_eq!(parse_scope("none"), Some(Scope::None));
        assert_eq!(parse_scope("nope"), None);
        assert_eq!(parse_layout("pick"), Some(EngineLayout::Pick));
        assert_eq!(parse_layout("nope"), None);
    }

    #[test]
    fn printed_time_slices_the_clock() {
        assert_eq!(printed_time("2026-10-07T07:40:12+07:00"), "07:40");
    }
}
