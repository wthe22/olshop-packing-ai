//! `prepare`, `revert` and `amend`: the batch steps of `packing-batch` on one date of the data
//! folder, the same calls the app makes.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;

use jiff::civil::Date;
use packing_batch::{
    BatchError, Draft, ReadRequest, SaveOutcome, SaveRequest, Saved, amend, check_files,
    read_draft, revert, save,
};
use packing_engine::ids::BatchId;
use packing_engine::workflows::{SortRule, Workflows};
use packing_pdf::worker::PdfWorker;
use packing_store::{DataLock, DateFolder, Journal, open_date};

use crate::accounts::answer_unlisted;
use crate::error::CliError;
use crate::options::{Options, pdfium_dir};
use crate::program;

const APP: &str = env!("CARGO_PKG_VERSION");

/// One date of the data folder, opened for writing.
struct Session {
    folder: DateFolder,
    journal: Journal,
    workflows: Workflows,
    _lock: DataLock,
}

impl Session {
    fn open(options: &Options) -> Result<Session, CliError> {
        let data = program::data_dir(&options.program, options.data.as_deref())?;
        let workflows = program::workflows(&data)?;
        let lock = DataLock::acquire(&data)?;
        let folder = DateFolder::new(&data, options.day());
        let journal = open_date(&folder)?.journal;
        Ok(Session {
            folder,
            journal,
            workflows,
            _lock: lock,
        })
    }

    fn date(&self) -> Date {
        self.folder.date()
    }

    fn read(
        &self,
        worker: &PdfWorker,
        paths: &[PathBuf],
        rule: &SortRule,
        redo: Option<BatchId>,
    ) -> Result<Draft, BatchError> {
        let fold = self.journal.fold();
        let files = check_files(worker, paths, &self.folder, fold, redo)?;
        let (progress, _unread) = mpsc::channel();
        read_draft(ReadRequest {
            worker,
            files,
            workflows: &self.workflows,
            rule,
            fold,
            accounts: &self.workflows.accounts,
            redo,
            cancel: Arc::new(AtomicBool::new(false)),
            progress,
        })
    }

    fn save(
        &mut self,
        worker: &PdfWorker,
        draft: &Draft,
        options: &Options,
    ) -> Result<(), CliError> {
        let request = SaveRequest {
            worker,
            folder: &self.folder,
            draft,
            save_anyway: options.save_anyway,
            app: APP,
            progress: &|_| {},
        };
        match save(&request, &mut self.journal)? {
            SaveOutcome::Saved(saved) => {
                report(&saved, draft, self.date(), options.quiet_ids);
                Ok(())
            }
            SaveOutcome::Findings(findings) => Err(CliError::Findings(refusal(
                &findings.errors,
                &findings.warnings,
                options.quiet_ids,
            ))),
        }
    }
}

fn spawn_worker(options: &Options) -> Result<PdfWorker, CliError> {
    Ok(PdfWorker::spawn(pdfium_dir(options.pdfium.as_deref()))?)
}

pub fn prepare(options: &Options) -> Result<(), CliError> {
    let labels = options.labels_in_order()?;
    let mut session = Session::open(options)?;
    let rule = session
        .workflows
        .sort_rule(&options.rule)
        .cloned()
        .ok_or_else(|| CliError::Usage(format!("there is no sort rule \"{}\"", options.rule)))?;
    let worker = spawn_worker(options)?;
    let mut draft = session.read(&worker, &labels, &rule, None)?;
    answer_unlisted(&mut draft, options.account.as_deref(), options.quiet_ids)?;
    session.save(&worker, &draft, options)
}

pub fn revert_last(options: &Options) -> Result<(), CliError> {
    let mut session = Session::open(options)?;
    match revert(&session.folder, &mut session.journal, APP) {
        Ok(batch) => println!("reverted batch {batch} on {}", session.date()),
        Err(BatchError::NoBatch) => println!("no batch to revert on {}", session.date()),
        Err(error) => return Err(error.into()),
    }
    Ok(())
}

/// Redo the newest batch from its stored inputs, with the sort rule it was made with.
pub fn amend_last(options: &Options) -> Result<(), CliError> {
    let mut session = Session::open(options)?;
    let redo = match amend(&session.folder, session.journal.fold(), &session.workflows) {
        Ok(redo) => redo,
        Err(BatchError::NoBatch) => {
            println!("no batch to amend on {}", session.date());
            return Ok(());
        }
        Err(error) => return Err(error.into()),
    };
    let recorded = session.journal.fold().live[&redo.batch].saved.clone();
    let paths: Vec<PathBuf> = redo.files.iter().map(|file| file.path.clone()).collect();
    let worker = spawn_worker(options)?;
    let mut draft = session.read(&worker, &paths, &redo.rule, Some(redo.batch))?;
    draft.keep_as_recorded(&recorded)?;
    answer_unlisted(&mut draft, options.account.as_deref(), options.quiet_ids)?;
    session.save(&worker, &draft, options)
}

/// What the verifier found when `--save-anyway` was not given; `--quiet-ids` gives counts only,
/// because a finding names orders and pages.
fn refusal(errors: &[String], warnings: &[String], quiet_ids: bool) -> String {
    let mut text = String::from(
        "the saved PDFs do not match the plan; nothing was saved (--save-anyway saves with the findings)",
    );
    if quiet_ids {
        text.push_str(&format!(
            "\n  {} errors, {} warnings",
            errors.len(),
            warnings.len()
        ));
        return text;
    }
    for finding in errors.iter().chain(warnings) {
        text.push_str(&format!("\n  {finding}"));
    }
    text
}

fn report(saved: &Saved, draft: &Draft, date: Date, quiet_ids: bool) {
    let record = &saved.record;
    println!("saved batch {} on {date}", record.batch);
    println!(
        "  {} input files, {} orders, {} repeats left out",
        record.inputs.len(),
        record.orders.len(),
        record.left_out.len()
    );
    for output in &record.outputs {
        println!("  {} ({} pages)", output.file, output.pages);
    }
    if !draft.unseen_logos.is_empty() {
        println!(
            "  {} header images are not known to the rules",
            draft.unseen_logos.len()
        );
    }
    if quiet_ids {
        println!(
            "  {} warnings, {} errors",
            record.warnings.len(),
            record.errors.len()
        );
    } else {
        for finding in &record.warnings {
            println!("  warning: {}", finding.text);
        }
        for finding in &record.errors {
            println!("  error: {}", finding.text);
        }
    }
    for problem in &saved.after_commit {
        println!("  after the save: {problem}");
    }
}
