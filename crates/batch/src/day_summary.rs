//! The day summary (day-summary.md): written on request from a date's fold into
//! `{date}/day summary.pdf`, through a `.partial` file renamed over the old one. A view of the
//! journal: no record, no verifier.

use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

use packing_pdf::day_summary::{DayBatch, DayFacts, DayReprint, write_day_summary};
use packing_pdf::worker::PdfWorker;
use packing_store::{DateFolder, Fold, StoreError};

use crate::BatchError;

pub const DAY_SUMMARY_PDF: &str = "day summary.pdf";

/// Write the date's day summary and return its path. A day summary that a PDF viewer holds open
/// stops the write with the file named; the old file stays.
pub fn day_summary(
    worker: &PdfWorker,
    folder: &DateFolder,
    fold: &Fold,
) -> Result<PathBuf, BatchError> {
    if fold.live.is_empty() {
        return Err(BatchError::NoBatch);
    }
    let target = folder.dir().join(DAY_SUMMARY_PDF);
    let partial = folder.dir().join(format!("{DAY_SUMMARY_PDF}.partial"));
    let (fold, out) = (fold.clone(), partial.clone());
    let printed = jiff::Zoned::now().strftime("%H:%M").to_string();
    let written = worker
        .run(move |pdfium| {
            let hh_mm = |time: &packing_engine::record::Datetime| {
                time.zoned().strftime("%H:%M").to_string()
            };
            let facts = DayFacts {
                workspace: &fold.workspace,
                date: fold.date,
                batches: fold
                    .live
                    .values()
                    .rev()
                    .map(|live| DayBatch {
                        record: &live.saved,
                        time: hh_mm(&live.modify_time),
                        printed: &live.printed,
                    })
                    .collect(),
                reprints: fold
                    .reprints
                    .iter()
                    .map(|reprint| DayReprint {
                        made: &reprint.made,
                        time: hh_mm(&reprint.create_time),
                    })
                    .collect(),
                printed: &printed,
            };
            write_day_summary(pdfium, &out, &facts)
        })
        .map_err(BatchError::from)
        .and_then(|pages| pages.map_err(BatchError::from));
    if let Err(error) = written {
        let _ = fs::remove_file(&partial);
        return Err(error);
    }
    fs::rename(&partial, &target).map_err(|error| {
        let _ = fs::remove_file(&partial);
        let in_use =
            error.kind() == ErrorKind::PermissionDenied || error.raw_os_error() == Some(32);
        BatchError::Store(if in_use {
            StoreError::FileInUse {
                path: target.clone(),
            }
        } else {
            StoreError::Io {
                path: target.clone(),
                error,
            }
        })
    })?;
    Ok(target)
}
