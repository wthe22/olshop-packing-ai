//! The batch save and the batch folder delete: the whole batch is written into a temporary folder
//! and renamed into place, or not written at all (07 › *Save*, *Revert* step 1).

use std::fs;
use std::path::{Path, PathBuf};

use jiff::Zoned;

use crate::groupings::Grouping;
use crate::ids::BatchNumber;
use crate::names;

use super::error::{DayError, io_error};
use super::state::{BatchEntry, DayState, FileEntry, PackingListChoice, PdfEntry, SavedOrder};

/// Everything a finished batch adds to the day, except the files a closure writes (07 › *Save*).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BatchWrite {
    pub batch: BatchNumber,
    pub save_time: String,
    /// The grouping the batch was packed with (04 › *Sets*).
    pub grouping: Grouping,
    pub files: Vec<FileEntry>,
    pub pdfs: Vec<PdfEntry>,
    pub packing_list: PackingListChoice,
    /// The read warnings of the batch, stored in `state.json` (07 › *`state.json`*).
    pub warnings: Vec<String>,
    pub saved: Vec<SavedOrder>,
}

/// The current time as `state.json` stores it: local time with the UTC offset
/// (`2026-10-07T07:40:12+07:00`).
pub fn now_save_time() -> String {
    Zoned::now().strftime("%Y-%m-%dT%H:%M:%S%:z").to_string()
}

/// The check before saving's findings, stored with the batch (07 › *Check before saving*):
/// errors and warnings, kept apart. Empty when the check found nothing.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Findings {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Write a batch completely or not at all (07 › *Save*): create `batch <b>.partial/`, let
/// `fill` write the saved PDFs, the packing list, `summary.txt` and `download/` into it and
/// return the check's [`Findings`], rename it to `batch <b>` (replacing an amended batch), then
/// add the batch to `state.json`. On any error before the rename the temporary folder is deleted
/// and the state is unchanged.
pub fn save_batch<F>(
    day_dir: &Path,
    state: &mut DayState,
    write: &BatchWrite,
    fill: F,
) -> Result<PathBuf, DayError>
where
    F: FnOnce(&Path) -> Result<Findings, DayError>,
{
    let temp = day_dir.join(names::temp_batch_folder(write.batch));
    if temp.exists() {
        fs::remove_dir_all(&temp).map_err(|error| io_error(&temp, error))?;
    }
    fs::create_dir_all(&temp).map_err(|error| io_error(&temp, error))?;

    let findings = match fill(&temp) {
        Ok(findings) => findings,
        Err(error) => {
            let _ = fs::remove_dir_all(&temp);
            return Err(error);
        }
    };

    let final_dir = day_dir.join(names::batch_folder(write.batch));
    if final_dir.exists() {
        fs::remove_dir_all(&final_dir).map_err(|error| io_error(&final_dir, error))?;
    }
    if let Err(error) = fs::rename(&temp, &final_dir) {
        let _ = fs::remove_dir_all(&temp);
        return Err(io_error(&final_dir, error));
    }

    let mut warnings = write.warnings.clone();
    warnings.extend(findings.warnings);
    state.batches.push(BatchEntry {
        batch: write.batch,
        save_time: write.save_time.clone(),
        grouping: write.grouping.clone(),
        files: write.files.clone(),
        pdfs: write.pdfs.clone(),
        packing_list: write.packing_list.clone(),
        // A batch written by a save (or written again by an amend) starts with no ticks: the
        // amend writes the batch afresh (07 › *`state.json`*).
        printed: Vec::new(),
        warnings,
        errors: findings.errors,
    });
    state.saved.extend(write.saved.iter().cloned());
    state.save(day_dir)?;
    Ok(final_dir)
}

/// Delete a batch folder (07 › *Revert* step 1); a file open in another program is a
/// [`DayError::FileInUse`].
pub fn delete_batch_folder(day_dir: &Path, batch: BatchNumber) -> Result<(), DayError> {
    let path = day_dir.join(names::batch_folder(batch));
    if !path.exists() {
        return Ok(());
    }
    fs::remove_dir_all(&path).map_err(|error| io_error(&path, error))
}
