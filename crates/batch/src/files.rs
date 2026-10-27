//! The files chosen for a batch (step 3): each one's SHA-256, size, times and page count, and the
//! live batch of the date that read the same bytes — never the batch an amend redoes, whose own
//! files they may be. One file chosen twice is listed once.

use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

use jiff::Timestamp;
use jiff::tz::TimeZone;
use packing_engine::ids::BatchId;
use packing_engine::record::{Datetime, Sha256};
use packing_pdf::read;
use packing_pdf::worker::PdfWorker;
use packing_store::{Fold, StoreError, sha256_of};

use crate::BatchError;

#[derive(Clone, Debug, PartialEq)]
pub struct ChosenFile {
    pub path: PathBuf,
    /// The name it had where it was chosen.
    pub filename: String,
    pub sha256: Sha256,
    pub size: u64,
    pub page_count: u16,
    pub create_time: Datetime,
    pub modify_time: Datetime,
    /// The live batch of the date that read the same bytes, other than the one an amend redoes.
    pub used_in: Option<BatchId>,
}

/// The files at `paths` for a batch on `fold`'s date; `redo` is the batch an amend redoes.
pub fn check_files(
    worker: &PdfWorker,
    paths: &[PathBuf],
    fold: &Fold,
    redo: Option<BatchId>,
) -> Result<Vec<ChosenFile>, BatchError> {
    let mut files: Vec<ChosenFile> = Vec::new();
    for path in paths {
        let sha256 = sha256_of(path)?;
        if files.iter().any(|file| file.sha256 == sha256) {
            continue;
        }
        let meta = fs::metadata(path).map_err(|error| StoreError::Io {
            path: path.clone(),
            error,
        })?;
        files.push(ChosenFile {
            path: path.clone(),
            filename: path
                .file_name()
                .map_or_else(String::new, |name| name.to_string_lossy().into_owned()),
            used_in: used_in(fold, &sha256, redo),
            sha256,
            size: meta.len(),
            page_count: 0,
            create_time: time(meta.created().ok()),
            modify_time: time(meta.modified().ok()),
        });
    }
    let paths: Vec<PathBuf> = files.iter().map(|file| file.path.clone()).collect();
    let counts = worker.run(move |pdfium| {
        read::open_all(pdfium, &paths)
            .map(|opened| opened.iter().map(|file| file.pages).collect::<Vec<_>>())
    })??;
    for (file, pages) in files.iter_mut().zip(counts) {
        file.page_count = u16::try_from(pages).unwrap_or(u16::MAX);
    }
    Ok(files)
}

/// The live batch other than `redo` whose inputs hold these bytes.
pub(crate) fn used_in(fold: &Fold, sha256: &Sha256, redo: Option<BatchId>) -> Option<BatchId> {
    fold.live
        .iter()
        .filter(|(batch, _)| Some(**batch) != redo)
        .find(|(_, live)| live.saved.inputs.iter().any(|i| i.sha256 == *sha256))
        .map(|(batch, _)| *batch)
}

/// A file time on the PC's clock, to the second; the Unix epoch when the system gives none.
fn time(system: Option<SystemTime>) -> Datetime {
    let second = system
        .and_then(|time| Timestamp::try_from(time).ok())
        .map_or(0, |stamp| stamp.as_second());
    let stamp = Timestamp::from_second(second).unwrap_or(Timestamp::UNIX_EPOCH);
    Datetime::from(stamp.to_zoned(TimeZone::system()))
}
