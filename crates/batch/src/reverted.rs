//! The copies of a reverted batch's PDFs the owner chose to save (05-after-save.md, step 17):
//! its group PDFs, summary PDF and account summaries, copied before the revert's commit into
//! `reverted/batch {n} reverted {hh.mm}/` of the date folder. Nothing reads them again.

use std::fs;
use std::path::{Path, PathBuf};

use packing_engine::ids::BatchId;
use packing_store::{OUTPUT_DIR, StoreError};

use crate::summaries::account_summaries;

pub const REVERTED_DIR: &str = "reverted";

/// What a revert does with the batch's PDFs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RevertedPdfs {
    /// Copied into `reverted/` before they leave `output/`.
    Save,
    Discard,
}

/// Copy the batch's `outputs` that are in `output/`, and its account summaries, into a new folder of
/// `reverted/` and return it. A copy that fails takes the folder away again.
pub(crate) fn save_copies(
    dir: &Path,
    batch: BatchId,
    outputs: &[String],
) -> Result<PathBuf, StoreError> {
    let mut sources: Vec<PathBuf> = outputs
        .iter()
        .map(|file| dir.join(OUTPUT_DIR).join(file))
        .filter(|path| path.is_file())
        .collect();
    sources.extend(account_summaries(dir, batch)?);
    let target = free_folder(&dir.join(REVERTED_DIR), batch);
    fs::create_dir_all(&target).map_err(|error| StoreError::Io {
        path: target.clone(),
        error,
    })?;
    for source in &sources {
        let copy = target.join(source.file_name().unwrap_or_default());
        if let Err(error) = fs::copy(source, &copy) {
            drop_copies(&target);
            return Err(StoreError::Io {
                path: source.clone(),
                error,
            });
        }
    }
    Ok(target)
}

/// Take away the copies of a revert that did not happen.
pub(crate) fn drop_copies(folder: &Path) {
    let _ = fs::remove_dir_all(folder);
}

/// `batch 2 reverted 14.05`, or with ` (2)`, ` (3)` after it when that folder is there.
fn free_folder(reverted: &Path, batch: BatchId) -> PathBuf {
    let name = format!(
        "batch {batch} reverted {}",
        jiff::Zoned::now().strftime("%H.%M")
    );
    let mut folder = reverted.join(&name);
    let mut n = 2;
    while folder.exists() {
        folder = reverted.join(format!("{name} ({n})"));
        n += 1;
    }
    folder
}
