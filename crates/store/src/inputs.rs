//! `input/{sha256}.pdf`: each label download stored once, named by its content (04-save.md,
//! step 11), and checked when a date opens.

use std::fs;
use std::path::{Path, PathBuf};

use packing_engine::ids::{BatchId, PdfId};
use packing_engine::record::Sha256;

use crate::StoreError;
use crate::files::sha256_of;
use crate::fold::Fold;

pub const INPUT_DIR: &str = "input";

pub fn input_path(date_dir: &Path, sha256: &Sha256) -> PathBuf {
    date_dir.join(INPUT_DIR).join(format!("{sha256}.pdf"))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredInput {
    pub sha256: Sha256,
    pub size: u64,
}

/// Store `source` in the date's `input/` unless it is there: copied to a `.partial` file, synced,
/// hashed again and renamed. A copy whose hash differs from the source's is not stored.
pub fn store_input(date_dir: &Path, source: &Path) -> Result<StoredInput, StoreError> {
    let sha256 = sha256_of(source)?;
    let size = fs::metadata(source)
        .map_err(|e| StoreError::io(source, e))?
        .len();
    let target = input_path(date_dir, &sha256);
    if target.is_file() {
        return Ok(StoredInput { sha256, size });
    }
    let dir = date_dir.join(INPUT_DIR);
    fs::create_dir_all(&dir).map_err(|e| StoreError::io(&dir, e))?;
    let partial = target.with_extension("pdf.partial");
    let mut from = fs::File::open(source).map_err(|e| StoreError::io(source, e))?;
    let mut copy = fs::File::create(&partial).map_err(|e| StoreError::io(&partial, e))?;
    std::io::copy(&mut from, &mut copy)
        .and_then(|_| copy.sync_all())
        .map_err(|e| StoreError::io(&partial, e))?;
    drop(copy);
    if sha256_of(&partial)? != sha256 {
        let _ = fs::remove_file(&partial);
        return Err(StoreError::InputChanged {
            path: source.to_path_buf(),
        });
    }
    fs::rename(&partial, &target).map_err(|e| StoreError::io(&target, e))?;
    Ok(StoredInput { sha256, size })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputState {
    Missing,
    Changed,
}

/// An input of a live batch that cannot be used: amend, reprint and restore stop on it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InputProblem {
    pub batch: BatchId,
    pub pdf: PdfId,
    pub filename: String,
    pub state: InputState,
}

/// Every input of a live batch, hashed and compared with its record.
pub fn check_inputs(date_dir: &Path, fold: &Fold) -> Result<Vec<InputProblem>, StoreError> {
    let mut problems = Vec::new();
    for (batch, live) in &fold.live {
        for input in &live.saved.inputs {
            let path = input_path(date_dir, &input.sha256);
            let state = if !path.is_file() {
                Some(InputState::Missing)
            } else if sha256_of(&path)? != input.sha256 {
                Some(InputState::Changed)
            } else {
                None
            };
            if let Some(state) = state {
                problems.push(InputProblem {
                    batch: *batch,
                    pdf: input.pdf,
                    filename: input.filename.clone(),
                    state,
                });
            }
        }
    }
    Ok(problems)
}
