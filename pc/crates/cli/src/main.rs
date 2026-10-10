//! Developer CLI: the batch steps of the PC app from the command line, for tests and sample
//! runs (07 › *Architecture*: `packing-cli` is the same engine, not for daily use).
//!
//! ```text
//! packing-cli prepare --labels <file>... [--rules <categories.toml>] [--day YYYY-MM-DD]
//!                     [--data <dir>] [--packing-list whole|per-pdf|none]
//!                     [--layout pick,full,summary] [--pdfium <dir>] [--quiet-ids]
//! packing-cli revert  [--day <day>] [--data <dir>]
//! packing-cli amend   [--day <day>] [--data <dir>] [--rules <categories.toml>]
//! ```

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;

use packing_engine::day::{DayState, Layout, PackingListChoice, Scope, delete_batch_folder};
use packing_engine::plan::{Plan, PlanResult};
use packing_engine::rules::{Category, load_rules};
use packing_pdf::batch::{self, BatchError, Draft};
use packing_pdf::worker::PdfWorker;

const USAGE: &str = "\
packing-cli — developer command line for the packing app (not for daily use)

USAGE:
  packing-cli prepare --labels <file>... [options]
  packing-cli revert  [options]
  packing-cli amend   [options]

OPTIONS (prepare):
  --labels <file>...           One or more label PDFs of this batch (required)
  --rules <categories.toml>    The picks, in order (default: <data>/categories.toml)
  --day YYYY-MM-DD             The day folder (default: today)
  --data <dir>                 The data folder holding categories.toml and labels/ (default: .)
  --packing-list <scope>       one per batch | one per PDF | none (default: whole)
  --layout <list>              Comma-separated: pick, full, summary (default: pick)
  --pdfium <dir>               The folder with pdfium.dll (default: next to the exe, else pc/vendor)
  --quiet-ids                  Print counts only; no order IDs, tracking IDs or names

OPTIONS (revert, amend):
  --day YYYY-MM-DD --data <dir> [--rules <categories.toml>] [--pdfium <dir>] [--quiet-ids]

The batch is written to <data>/labels/<day>/batch <b>/ (07 › *What happens in one batch*).
";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("--help" | "-h") | None => print!("{USAGE}"),
        Some("prepare") => finish(run_prepare(&args[1..])),
        Some("revert") => finish(run_revert(&args[1..])),
        Some("amend") => finish(run_amend(&args[1..])),
        Some(other) => {
            eprintln!("error: unknown command \"{other}\"; expected prepare, revert or amend");
            std::process::exit(1);
        }
    }
}

fn finish(result: Result<(), CliError>) {
    match result {
        Ok(()) => std::process::exit(0),
        Err(error) => {
            eprintln!("error: {error}");
            std::process::exit(1);
        }
    }
}

/// A failure that ends the command with exit code 1 and one message (03 › *Errors and warnings*).
#[derive(Debug)]
enum CliError {
    Usage(String),
    Rules(String),
    Io(String),
    Batch(BatchError),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::Usage(message) => f.write_str(message),
            CliError::Rules(message) => write!(f, "the categories have an error: {message}"),
            CliError::Io(message) => f.write_str(message),
            CliError::Batch(error) => error.fmt(f),
        }
    }
}

impl From<BatchError> for CliError {
    fn from(error: BatchError) -> Self {
        CliError::Batch(error)
    }
}

// --------------------------------------------------------------------------- arguments

/// The parsed options shared by the three commands.
struct Args {
    labels: Vec<PathBuf>,
    rules: Option<PathBuf>,
    day: Option<String>,
    data: PathBuf,
    scope: Scope,
    layouts: Vec<Layout>,
    pdfium: Option<PathBuf>,
    quiet_ids: bool,
}

impl Args {
    fn parse(args: &[String]) -> Result<Args, CliError> {
        let mut parsed = Args {
            labels: Vec::new(),
            rules: None,
            day: None,
            data: std::env::current_dir().map_err(|e| CliError::Io(e.to_string()))?,
            scope: Scope::Whole,
            layouts: Vec::new(),
            pdfium: None,
            quiet_ids: false,
        };
        let mut index = 0;
        while index < args.len() {
            let flag = args[index].as_str();
            index += 1;
            match flag {
                "--labels" => {
                    while index < args.len() && !args[index].starts_with("--") {
                        parsed.labels.push(PathBuf::from(&args[index]));
                        index += 1;
                    }
                }
                "--rules" => parsed.rules = Some(PathBuf::from(value(args, &mut index, flag)?)),
                "--day" => parsed.day = Some(value(args, &mut index, flag)?),
                "--data" => parsed.data = PathBuf::from(value(args, &mut index, flag)?),
                "--pdfium" => parsed.pdfium = Some(PathBuf::from(value(args, &mut index, flag)?)),
                "--quiet-ids" => parsed.quiet_ids = true,
                "--packing-list" => {
                    let name = value(args, &mut index, flag)?;
                    parsed.scope = batch::parse_scope(&name).ok_or_else(|| {
                        CliError::Usage(format!(
                            "unknown packing-list scope \"{name}\"; expected one per batch, \
                             one per PDF or none"
                        ))
                    })?;
                }
                "--layout" => {
                    let list = value(args, &mut index, flag)?;
                    for name in list.split(',').map(str::trim).filter(|s| !s.is_empty()) {
                        let layout = batch::parse_layout(name).ok_or_else(|| {
                            CliError::Usage(format!(
                                "unknown layout \"{name}\"; expected pick, full or summary"
                            ))
                        })?;
                        parsed.layouts.push(layout);
                    }
                }
                other => return Err(CliError::Usage(format!("unknown option \"{other}\""))),
            }
        }
        if parsed.layouts.is_empty() {
            parsed.layouts.push(Layout::Pick);
        }
        Ok(parsed)
    }

    fn packing(&self) -> PackingListChoice {
        PackingListChoice {
            scope: self.scope,
            layouts: self.layouts.clone(),
        }
    }

    fn day(&self) -> String {
        self.day
            .clone()
            .unwrap_or_else(|| jiff::Zoned::now().strftime("%Y-%m-%d").to_string())
    }

    /// The day folder `<data>/labels/<day>/` (07 › *Day folder*).
    fn day_dir(&self, day: &str) -> PathBuf {
        self.data.join("labels").join(day)
    }

    fn rules_path(&self) -> PathBuf {
        self.rules
            .clone()
            .unwrap_or_else(|| self.data.join("categories.toml"))
    }
}

fn value(args: &[String], index: &mut usize, flag: &str) -> Result<String, CliError> {
    let value = args
        .get(*index)
        .filter(|value| !value.starts_with("--"))
        .ok_or_else(|| CliError::Usage(format!("{flag} needs a value")))?;
    *index += 1;
    Ok(value.clone())
}

// --------------------------------------------------------------------------- commands

fn run_prepare(args: &[String]) -> Result<(), CliError> {
    let args = Args::parse(args)?;
    if args.labels.is_empty() {
        return Err(CliError::Usage(
            "prepare needs at least one --labels <file>".to_string(),
        ));
    }
    let day = args.day();
    let day_dir = args.day_dir(&day);
    let categories = load_categories(&args)?;
    let mut state = DayState::load(&day_dir).map_err(|e| CliError::Batch(e.into()))?;

    let worker = spawn_worker(&args)?;
    // Files are read in file-name order (= download order, 07 › *Read*).
    let mut labels = args.labels.clone();
    labels.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
    let (draft, plan) = read_and_plan(&worker, &categories, &state, &labels)?;

    let packing = args.packing();
    let result = batch::save(&worker, &draft, &plan, &packing, &day, &day_dir, &mut state)?;
    report(&result, &draft, &plan, &day, &day_dir, args.quiet_ids);
    Ok(())
}

fn run_revert(args: &[String]) -> Result<(), CliError> {
    let args = Args::parse(args)?;
    let day = args.day();
    let day_dir = args.day_dir(&day);
    let mut state = DayState::load(&day_dir).map_err(|e| CliError::Batch(e.into()))?;
    match state.revert_last() {
        None => println!("no batch to revert on {day}"),
        Some(entry) => {
            delete_batch_folder(&day_dir, entry.batch).map_err(|e| CliError::Batch(e.into()))?;
            state
                .save(&day_dir)
                .map_err(|e| CliError::Batch(e.into()))?;
            println!("reverted batch {} on {day}", entry.batch);
        }
    }
    Ok(())
}

fn run_amend(args: &[String]) -> Result<(), CliError> {
    let args = Args::parse(args)?;
    let day = args.day();
    let day_dir = args.day_dir(&day);
    let categories = load_categories(&args)?;
    let state = DayState::load(&day_dir).map_err(|e| CliError::Batch(e.into()))?;
    let Some(amend) = state.amend(&day_dir) else {
        println!("no batch to amend on {day}");
        return Ok(());
    };

    let worker = spawn_worker(&args)?;
    // The duplicate guard ignores the amended batch's own orders (07 › *Amend*).
    let (draft, plan) = read_and_plan(&worker, &categories, &amend.state, &amend.files)?;
    let packing = args.packing();
    let mut amend_state = amend.state;
    let result = batch::save(
        &worker,
        &draft,
        &plan,
        &packing,
        &day,
        &day_dir,
        &mut amend_state,
    )?;
    report(&result, &draft, &plan, &day, &day_dir, args.quiet_ids);
    Ok(())
}

// ---------------------------------------------------------------------------- shared

fn load_categories(args: &Args) -> Result<Vec<Category>, CliError> {
    let path = args.rules_path();
    let text = fs::read_to_string(&path)
        .map_err(|e| CliError::Io(format!("cannot read {}: {e}", path.display())))?;
    load_rules(&text).map_err(|e| CliError::Rules(e.to_string()))
}

fn spawn_worker(args: &Args) -> Result<PdfWorker, CliError> {
    PdfWorker::spawn(pdfium_dir(args.pdfium.as_deref())).map_err(|e| CliError::Batch(e.into()))
}

/// `check_files`, then `read` (progress on a private channel), then apply the rules
/// (07 › *What happens in one batch* steps 1-5).
fn read_and_plan(
    worker: &PdfWorker,
    categories: &[Category],
    state: &DayState,
    files: &[PathBuf],
) -> Result<(Draft, PlanResult), CliError> {
    let checks = batch::check_files(worker, files, state)?;
    let (progress, _rx) = mpsc::channel();
    let draft = batch::read(
        worker,
        &checks,
        state,
        Arc::new(AtomicBool::new(false)),
        progress,
    )?;
    let plan = Plan::from_categories(categories).count(&draft.orders, state.next_pdf_number());
    Ok((draft, plan))
}

/// Print the summary; `--quiet-ids` prints counts only, never the warning details that name
/// orders and tracking IDs.
fn report(
    result: &batch::SaveResult,
    draft: &Draft,
    plan: &PlanResult,
    day: &str,
    day_dir: &Path,
    quiet_ids: bool,
) {
    if quiet_ids {
        let text = batch::summary_text(&batch::SummaryInput {
            day,
            files: draft.files.len(),
            pages: draft.pages,
            orders: draft.total_orders,
            slip_pages: draft.slip_pages,
            picks: &plan.picks,
            duplicates: draft.duplicates.len(),
            warnings: &draft.warnings,
            day_dir,
            packing_lists: &result.packing_lists,
            quiet_ids: true,
        });
        print!("{text}");
    } else {
        print!("{}", result.summary);
    }
}

/// The `pdfium.dll` folder: `--pdfium`, else the folder of the exe, else `pc/vendor` next to the
/// crate (found at build time), so `cargo run` and the built binary both work.
fn pdfium_dir(override_dir: Option<&Path>) -> PathBuf {
    if let Some(dir) = override_dir {
        return dir.to_path_buf();
    }
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
        && dir.join("pdfium.dll").exists()
    {
        return dir.to_path_buf();
    }
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn parses_labels_and_defaults() {
        let parsed = Args::parse(&args(&["--labels", "a.pdf", "b.pdf"])).expect("parse");
        assert_eq!(parsed.labels.len(), 2);
        assert_eq!(parsed.scope, Scope::Whole);
        assert_eq!(parsed.layouts, vec![Layout::Pick]);
    }

    #[test]
    fn parses_scope_layouts_and_day() {
        let parsed = Args::parse(&args(&[
            "--labels",
            "a.pdf",
            "--layout",
            "full,summary",
            "--packing-list",
            "per-pdf",
            "--day",
            "2026-10-07",
        ]))
        .expect("parse");
        assert_eq!(parsed.scope, Scope::PerPdf);
        assert_eq!(parsed.layouts, vec![Layout::Full, Layout::Summary]);
        assert_eq!(parsed.day.as_deref(), Some("2026-10-07"));
    }

    #[test]
    fn rejects_an_unknown_scope() {
        let error = Args::parse(&args(&["--labels", "a.pdf", "--packing-list", "nope"]));
        assert!(matches!(error, Err(CliError::Usage(_))));
    }
}
