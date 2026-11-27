//! The summaries made on request (views.md): the day summary of all accounts or of one
//! (day-summary.md) and one account's summary of a batch (summary-pdf.md, *One account*), written
//! into the date's folder through a `.partial` file renamed over the old one. Views of the journal:
//! no record. An account's batch summary is checked by the verifier (V6) before it is kept.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use packing_engine::accounts::{Account, account_of_shop};
use packing_engine::ids::BatchId;
use packing_engine::record::BatchSaved;
use packing_engine::workflows::Courier;
use packing_pdf::PdfError;
use packing_pdf::day_summary::{DayBatch, DayFacts, write_day_summary};
use packing_pdf::summary_pdf::{SummaryFacts, write_summary};
use packing_pdf::worker::PdfWorker;
use packing_store::{DateFolder, Fold, Journal, StoreError};
use packing_verify::{AccountSummaryCheck, Findings, verify_account_summary};

use crate::BatchError;

pub const DAY_SUMMARY_PDF: &str = "Day Summary.pdf";

/// Whether `record` holds an order of `account`, as `accounts` gives each its account.
fn holds_account(record: &BatchSaved, account: &str, accounts: &[Account]) -> bool {
    record
        .orders
        .iter()
        .any(|order| account_of_shop(order.shop.as_deref(), accounts) == account)
}

/// The accounts as the verifier takes them: data it reads, not code it shares.
pub(crate) fn for_verifier(accounts: &[Account]) -> Vec<packing_verify::Account> {
    accounts
        .iter()
        .map(|account| packing_verify::Account {
            name: account.name.clone(),
            shops: account.shops.clone(),
        })
        .collect()
}

/// The day summary's file: of all accounts, or `Day Summary - {account}.pdf`.
pub fn day_summary_file(account: Option<&str>) -> String {
    account.map_or_else(
        || DAY_SUMMARY_PDF.to_string(),
        |account| format!("Day Summary - {account}.pdf"),
    )
}

/// An account's summary of a batch: `{batch} Summary - {account}.pdf`.
pub fn account_summary_file(batch: BatchId, account: &str) -> String {
    format!("{batch} Summary - {account}.pdf")
}

/// What a day summary is made from: the date's fold, the couriers that name its codes, the accounts
/// that give each order its account, and the account it is of (none: all accounts).
pub struct DaySummaryRequest<'a> {
    pub worker: &'a PdfWorker,
    pub folder: &'a DateFolder,
    pub fold: &'a Fold,
    pub couriers: &'a [Courier],
    pub accounts: &'a [Account],
    pub account: Option<&'a str>,
}

/// Write the date's day summary and return its path. A day summary that a PDF viewer holds open
/// stops the write with the file named; the old file stays.
pub fn day_summary(request: &DaySummaryRequest<'_>) -> Result<PathBuf, BatchError> {
    let DaySummaryRequest {
        worker,
        folder,
        fold,
        couriers,
        accounts,
        account,
    } = *request;
    let holds = |account: &str| {
        fold.live
            .values()
            .any(|live| holds_account(&live.saved, account, accounts))
    };
    if fold.live.is_empty() || account.is_some_and(|account| !holds(account)) {
        return Err(BatchError::NoBatch);
    }
    let target = folder.dir().join(day_summary_file(account));
    let partial = partial_of(&target);
    let (fold, out, couriers) = (fold.clone(), partial.clone(), couriers.to_vec());
    let accounts = accounts.to_vec();
    let account = account.map(str::to_string);
    let printed = now();
    let written = worker.run(move |pdfium| {
        let facts = DayFacts {
            date: fold.date,
            batches: fold
                .live
                .values()
                .rev()
                .map(|live| DayBatch {
                    record: &live.saved,
                    time: live.modify_time.zoned().strftime("%H:%M").to_string(),
                })
                .collect(),
            printed: &printed,
            couriers: &couriers,
            account: account.as_deref(),
            accounts: &accounts,
        };
        write_day_summary(pdfium, &out, &facts)
    });
    flatten(written, &partial)?;
    replace(&partial, &target)?;
    Ok(target)
}

/// What an account's summary of a batch came to: written and kept, or not kept for the
/// verifier's findings.
#[derive(Debug)]
pub enum AccountSummaryOutcome {
    Written(PathBuf),
    Findings(Findings),
}

/// What an account's summary of a batch is made from.
pub struct AccountSummaryRequest<'a> {
    pub worker: &'a PdfWorker,
    pub folder: &'a DateFolder,
    pub journal: &'a Journal,
    pub batch: BatchId,
    pub account: &'a str,
    pub couriers: &'a [Courier],
    /// The accounts of `config.toml`, which give each order its account by its shop.
    pub accounts: &'a [Account],
    pub app: &'a str,
}

/// Write `account`'s summary of the live `batch` from its record, check it (V6 on the account's
/// orders) and keep it; a finding deletes it.
pub fn account_summary(
    request: &AccountSummaryRequest<'_>,
) -> Result<AccountSummaryOutcome, BatchError> {
    let fold = request.journal.fold();
    let live = fold.live.get(&request.batch).ok_or(BatchError::NoBatch)?;
    if !holds_account(&live.saved, request.account, request.accounts) {
        return Err(BatchError::NoBatch);
    }
    let target = request
        .folder
        .dir()
        .join(account_summary_file(request.batch, request.account));
    let partial = partial_of(&target);
    let (record, out, couriers) = (
        live.saved.clone(),
        partial.clone(),
        request.couriers.to_vec(),
    );
    let accounts = request.accounts.to_vec();
    let (date, account, app, printed) = (
        fold.date,
        request.account.to_string(),
        request.app.to_string(),
        now(),
    );
    let written = request.worker.run(move |pdfium| {
        let facts = SummaryFacts {
            date,
            record: &record,
            account: Some(&account),
            printed: &printed,
            app: &app,
            couriers: &couriers,
            accounts: &accounts,
        };
        write_summary(pdfium, &out, &facts)
    });
    flatten(written, &partial)?;
    let findings = verify_account_summary(&AccountSummaryCheck {
        journal: &request.journal.bytes()?,
        batch: request.batch.get(),
        account: request.account,
        accounts: &for_verifier(request.accounts),
        path: &partial,
    });
    if !findings.is_clean() {
        let _ = fs::remove_file(&partial);
        return Ok(AccountSummaryOutcome::Findings(findings));
    }
    replace(&partial, &target)?;
    Ok(AccountSummaryOutcome::Written(target))
}

/// The account summaries of `batch` in the date folder `dir`.
pub(crate) fn account_summaries(dir: &Path, batch: BatchId) -> Result<Vec<PathBuf>, StoreError> {
    let prefix = format!("{batch} Summary - ");
    let entries = fs::read_dir(dir).map_err(|error| StoreError::Io {
        path: dir.to_path_buf(),
        error,
    })?;
    Ok(entries
        .flatten()
        .filter(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            name.starts_with(&prefix) && name.ends_with(".pdf")
        })
        .map(|entry| entry.path())
        .collect())
}

/// Delete the account summaries of `batch` in `dir`: a revert or an amend makes them stale.
pub(crate) fn delete_account_summaries(dir: &Path, batch: BatchId) -> Result<(), StoreError> {
    for path in account_summaries(dir, batch)? {
        fs::remove_file(&path).map_err(|error| StoreError::Io { path, error })?;
    }
    Ok(())
}

fn now() -> String {
    jiff::Zoned::now().strftime("%H:%M").to_string()
}

fn partial_of(target: &Path) -> PathBuf {
    let mut name = target.as_os_str().to_owned();
    name.push(".partial");
    PathBuf::from(name)
}

/// The worker's result, the partial file deleted when it failed.
fn flatten<T>(
    written: Result<Result<T, PdfError>, PdfError>,
    partial: &Path,
) -> Result<T, BatchError> {
    let written = written
        .map_err(BatchError::from)
        .and_then(|pages| pages.map_err(BatchError::from));
    if written.is_err() {
        let _ = fs::remove_file(partial);
    }
    written
}

/// Rename `partial` over `target`; a file a PDF viewer holds open stops it with the file named.
fn replace(partial: &Path, target: &Path) -> Result<(), BatchError> {
    fs::rename(partial, target).map_err(|error| {
        let _ = fs::remove_file(partial);
        let in_use =
            error.kind() == ErrorKind::PermissionDenied || error.raw_os_error() == Some(32);
        if in_use {
            BatchError::Store(StoreError::FileInUse {
                path: target.to_path_buf(),
            })
        } else {
            BatchError::Store(StoreError::Io {
                path: target.to_path_buf(),
                error,
            })
        }
    })
}
