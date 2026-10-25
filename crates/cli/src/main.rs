//! Developer CLI: the batch steps of the PC app from the command line, for tests and sample
//! runs (`packing-cli` is the same engine, not for daily use).
//!
//! ```text
//! packing-cli prepare --labels <file>... [--rules <categories.toml>] [--couriers <couriers.toml>]
//!                     [--day YYYY-MM-DD]
//!                     [--data <dir>] [--packing-list whole|per-pdf|none]
//!                     [--layout pick,full,summary] [--pdfium <dir>] [--quiet-ids]
//! packing-cli revert  [--day <day>] [--data <dir>]
//! packing-cli amend   [--day <day>] [--data <dir>] [--rules <categories.toml>]
//!                     [--couriers <couriers.toml>]
//! packing-cli dump    --labels <file>... [--couriers <couriers.toml>] [--pdfium <dir>]
//! ```

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;

mod dump;

use packing_engine::day::{DayState, Layout, PackingListChoice, Scope, delete_batch_folder};
use packing_engine::groupings::Grouping;
use packing_engine::label::{CourierTable, courier_data, load_couriers_file};
use packing_engine::plan::{Plan, PlanResult};
use packing_engine::rules::{Category, catch_all_category, load_rules};
use packing_pdf::batch::{self, BatchError, Draft, SaveRequest};
use packing_pdf::worker::PdfWorker;

const USAGE: &str = "\
packing-cli — developer command line for the packing app (not for daily use)

USAGE:
  packing-cli prepare --labels <file>... [options]
  packing-cli revert  [options]
  packing-cli amend   [options]
  packing-cli dump    --labels <file>... [--couriers <couriers.toml>] [--pdfium <dir>]

OPTIONS (prepare):
  --labels <file>...           One or more label PDFs of this batch (required)
  --rules <categories.toml>    The picks, in order (default: <data>/categories.toml)
  --couriers <couriers.toml>   The courier data (default: <data>/couriers.toml, else the built-in file)
  --day YYYY-MM-DD             The day folder (default: today)
  --data <dir>                 The data folder holding categories.toml, couriers.toml and labels/ (default: .)
  --packing-list <scope>       one per batch | one per PDF | none (default: whole)
  --layout <list>              Comma-separated: pick, full, summary (default: pick)
  --pdfium <dir>               The folder with pdfium.dll (default: next to the exe, else vendor)
  --quiet-ids                  Print counts only; no order IDs, tracking IDs or names

OPTIONS (revert, amend):
  --day YYYY-MM-DD --data <dir> [--rules <categories.toml>] [--couriers <couriers.toml>]
  [--pdfium <dir>] [--quiet-ids]

The batch is written to <data>/labels/<day>/batch <b>/. `dump` saves nothing: it prints what
reading the files gives as JSON, with the built-in courier list unless --couriers is given.
";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("--help" | "-h") | None => print!("{USAGE}"),
        Some("prepare") => finish(run_prepare(&args[1..])),
        Some("revert") => finish(run_revert(&args[1..])),
        Some("amend") => finish(run_amend(&args[1..])),
        Some("dump") => finish(run_dump(&args[1..])),
        Some(other) => {
            eprintln!(
                "error: unknown command \"{other}\"; expected prepare, revert, amend or dump"
            );
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

/// A failure that ends the command with exit code 1 and one message.
#[derive(Debug)]
enum CliError {
    Usage(String),
    Rules(String),
    Couriers(String),
    Io(String),
    Batch(BatchError),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::Usage(message) => f.write_str(message),
            CliError::Rules(message) => write!(f, "the categories have an error: {message}"),
            CliError::Couriers(message) => write!(f, "the couriers have an error: {message}"),
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

/// The parsed options shared by the three commands.
struct Args {
    labels: Vec<PathBuf>,
    rules: Option<PathBuf>,
    couriers: Option<PathBuf>,
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
            couriers: None,
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
                "--labels" => collect_labels(args, &mut index, &mut parsed.labels),
                "--rules" => parsed.rules = Some(PathBuf::from(value(args, &mut index, flag)?)),
                "--couriers" => {
                    parsed.couriers = Some(PathBuf::from(value(args, &mut index, flag)?));
                }
                "--day" => parsed.day = Some(value(args, &mut index, flag)?),
                "--data" => parsed.data = PathBuf::from(value(args, &mut index, flag)?),
                "--pdfium" => parsed.pdfium = Some(PathBuf::from(value(args, &mut index, flag)?)),
                "--quiet-ids" => parsed.quiet_ids = true,
                "--packing-list" => parsed.scope = parse_scope(args, &mut index, flag)?,
                "--layout" => parse_layouts(args, &mut index, flag, &mut parsed.layouts)?,
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

    /// The day folder `<data>/labels/<day>/`.
    fn day_dir(&self, day: &str) -> PathBuf {
        self.data.join("labels").join(day)
    }

    fn rules_path(&self) -> PathBuf {
        self.rules
            .clone()
            .unwrap_or_else(|| self.data.join("categories.toml"))
    }

    /// The courier file: `--couriers`, else `<data>/couriers.toml`, else (no file) the built-in
    /// list.
    fn couriers_path(&self) -> PathBuf {
        self.couriers
            .clone()
            .unwrap_or_else(|| self.data.join("couriers.toml"))
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

/// Every `--labels` value up to the next flag.
fn collect_labels(args: &[String], index: &mut usize, labels: &mut Vec<PathBuf>) {
    while *index < args.len() && !args[*index].starts_with("--") {
        labels.push(PathBuf::from(&args[*index]));
        *index += 1;
    }
}

/// The `--packing-list` scope.
fn parse_scope(args: &[String], index: &mut usize, flag: &str) -> Result<Scope, CliError> {
    let name = value(args, index, flag)?;
    name.parse::<Scope>().map_err(|_| {
        CliError::Usage(format!(
            "unknown packing-list scope \"{name}\"; expected one per batch, \
             one per PDF or none"
        ))
    })
}

/// The comma-separated `--layout` list.
fn parse_layouts(
    args: &[String],
    index: &mut usize,
    flag: &str,
    layouts: &mut Vec<Layout>,
) -> Result<(), CliError> {
    let list = value(args, index, flag)?;
    for name in list.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        let layout = name.parse::<Layout>().map_err(|_| {
            CliError::Usage(format!(
                "unknown layout \"{name}\"; expected pick, full or summary"
            ))
        })?;
        layouts.push(layout);
    }
    Ok(())
}

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
    let couriers = load_couriers(&args)?;
    let mut state = DayState::load(&day_dir).map_err(|e| CliError::Batch(e.into()))?;

    let worker = spawn_worker(&args)?;
    // Files are read in file-name order (= download order).
    let mut labels = args.labels.clone();
    labels.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
    let (draft, plan) = read_and_plan(&worker, &categories, &couriers, &state, &labels)?;

    let packing = args.packing();
    let grouping = Grouping::default();
    let (progress, _progress_rx) = mpsc::channel();
    save_and_report(
        &SaveRequest {
            worker: &worker,
            draft: &draft,
            plan: &plan,
            packing: &packing,
            grouping: &grouping,
            day: &day,
            day_dir: &day_dir,
            save_anyway: false,
            progress,
        },
        &mut state,
        args.quiet_ids,
    )?;
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
    let couriers = load_couriers(&args)?;
    let state = DayState::load(&day_dir).map_err(|e| CliError::Batch(e.into()))?;
    let Some(amend) = state.amend(&day_dir) else {
        println!("no batch to amend on {day}");
        return Ok(());
    };

    let worker = spawn_worker(&args)?;
    // The duplicate guard ignores the amended batch's own orders.
    let (draft, plan) = read_and_plan(&worker, &categories, &couriers, &amend.state, &amend.files)?;
    let packing = args.packing();
    let mut amend_state = amend.state;
    let (progress, _progress_rx) = mpsc::channel();
    // The amend keeps the batch's own grouping, as the app's amend does.
    save_and_report(
        &SaveRequest {
            worker: &worker,
            draft: &draft,
            plan: &plan,
            packing: &packing,
            grouping: &amend.grouping,
            day: &day,
            day_dir: &day_dir,
            save_anyway: false,
            progress,
        },
        &mut amend_state,
        args.quiet_ids,
    )?;
    Ok(())
}

/// The picks: `--rules`, else `<data>/categories.toml`, else (no file) the one catch-all entry,
/// so every order goes into one PDF set.
fn load_categories(args: &Args) -> Result<Vec<Category>, CliError> {
    let path = args.rules_path();
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(vec![catch_all_category()]);
        }
        Err(error) => {
            return Err(CliError::Io(format!(
                "cannot read {}: {error}",
                path.display()
            )));
        }
    };
    load_rules(&text).map_err(|e| CliError::Rules(e.to_string()))
}

/// The couriers of `<data>/couriers.toml`, or the built-in list when that file does not exist.
fn load_couriers(args: &Args) -> Result<CourierTable, CliError> {
    let path = args.couriers_path();
    load_couriers_file(&path).map_err(|e| CliError::Couriers(e.to_string()))
}

fn spawn_worker(args: &Args) -> Result<PdfWorker, CliError> {
    PdfWorker::spawn(pdfium_dir(args.pdfium.as_deref())).map_err(|e| CliError::Batch(e.into()))
}

fn run_dump(args: &[String]) -> Result<(), CliError> {
    let args = Args::parse(args)?;
    if args.labels.is_empty() {
        return Err(CliError::Usage(
            "dump needs at least one --labels <file>".to_string(),
        ));
    }
    let couriers = match &args.couriers {
        Some(path) => load_couriers_file(path).map_err(|e| CliError::Couriers(e.to_string()))?,
        None => courier_data().clone(),
    };
    let worker = spawn_worker(&args)?;
    let draft = read_draft(&worker, &couriers, &DayState::new(""), &args.labels)?;
    print!("{}", dump::draft_json(&draft));
    Ok(())
}

/// The draft of [`read_draft`], then the plan of its orders.
fn read_and_plan(
    worker: &PdfWorker,
    categories: &[Category],
    couriers: &CourierTable,
    state: &DayState,
    files: &[PathBuf],
) -> Result<(Draft, PlanResult), CliError> {
    let draft = read_draft(worker, couriers, state, files)?;
    let plan = Plan::from_categories(categories).count(&draft.orders, state.next_pdf_number());
    Ok((draft, plan))
}

/// `check_files`, then `read`, with progress on a private channel.
fn read_draft(
    worker: &PdfWorker,
    couriers: &CourierTable,
    state: &DayState,
    files: &[PathBuf],
) -> Result<Draft, CliError> {
    let checks = batch::check_files(worker, files, state)?;
    let (progress, _rx) = mpsc::channel();
    Ok(batch::read(&batch::ReadRequest {
        worker,
        files: &checks,
        state,
        cancel: Arc::new(AtomicBool::new(false)),
        progress,
        couriers,
    })?)
}

/// Save the batch and print its summary. The check runs on
/// every save; a finding stops the batch and the command ends with the findings, because the
/// command line has no *Save anyway*.
fn save_and_report(
    request: &SaveRequest<'_>,
    state: &mut DayState,
    quiet_ids: bool,
) -> Result<(), CliError> {
    let result = batch::save(request, state)?;
    if !result.saved {
        let mut message =
            String::from("the saved PDFs do not match the plan; the batch was not saved");
        for finding in result.errors.iter().chain(&result.warnings) {
            message.push_str(&format!("\n  {finding}"));
        }
        return Err(CliError::Io(message));
    }
    report(request, &result, quiet_ids);
    Ok(())
}

/// Print the summary; `--quiet-ids` prints counts only, never the warning details that name
/// orders and tracking IDs.
fn report(request: &SaveRequest<'_>, result: &batch::SaveResult, quiet_ids: bool) {
    if quiet_ids {
        let text = batch::summary_text(&batch::SummaryInput {
            day: request.day,
            files: request.draft.files.len(),
            pages: request.draft.pages,
            orders: request.draft.total_orders,
            slip_pages: request.draft.slip_pages,
            categories: &request.plan.categories,
            duplicates: request.draft.duplicates.len(),
            warnings: &request.draft.warnings,
            check_errors: &result.errors,
            check_warnings: &result.warnings,
            day_dir: request.day_dir,
            packing_lists: &result.packing_lists,
            quiet_ids: true,
        });
        print!("{text}");
    } else {
        print!("{}", result.summary);
    }
}

/// The `pdfium.dll` folder: `--pdfium`, else the folder of the exe, else `vendor` next to the
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
