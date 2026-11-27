//! The inputs: each label download stored in the date folder under the batch that names it, by its
//! content (04-save.md, step 11), and checked when a date opens.

use std::fs;
use std::path::{Path, PathBuf};

use packing_engine::ids::{BatchId, PdfId};
use packing_engine::record::Sha256;

use crate::StoreError;
use crate::date::DateFolder;
use crate::files::sha256_of;
use crate::fold::Fold;

/// `{batch} Input {pdf} {short hash}.pdf`, the number in two digits, so a file list sorted by name
/// shows a batch's inputs beside its outputs, in the order they were first read.
pub fn input_path(folder: &DateFolder, batch: BatchId, pdf: PdfId, sha256: &Sha256) -> PathBuf {
    folder.dir().join(input_name(batch, pdf, sha256))
}

fn input_name(batch: BatchId, pdf: PdfId, sha256: &Sha256) -> String {
    format!(
        "{} Input {:02} {}.pdf",
        batch.get(),
        pdf.get(),
        sha256.short()
    )
}

/// The file names of the live batches' inputs.
pub fn live_inputs(fold: &Fold) -> Vec<String> {
    fold.live
        .iter()
        .flat_map(|(batch, live)| {
            live.saved
                .inputs
                .iter()
                .map(|input| input_name(*batch, input.pdf, &input.sha256))
        })
        .collect()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredInput {
    pub sha256: Sha256,
    pub size: u64,
}

/// Store `source` in the date folder as input `pdf` of `batch` unless it is there: copied to a
/// `.partial` file, synced, hashed again and renamed. A copy whose hash differs from the source's
/// is not stored.
pub fn store_input(
    folder: &DateFolder,
    batch: BatchId,
    pdf: PdfId,
    source: &Path,
) -> Result<StoredInput, StoreError> {
    let sha256 = sha256_of(source)?;
    let size = fs::metadata(source)
        .map_err(|e| StoreError::io(source, e))?
        .len();
    let target = input_path(folder, batch, pdf, &sha256);
    if target.is_file() {
        return Ok(StoredInput { sha256, size });
    }
    let dir = folder.dir();
    fs::create_dir_all(dir).map_err(|e| StoreError::io(dir, e))?;
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
pub fn check_inputs(folder: &DateFolder, fold: &Fold) -> Result<Vec<InputProblem>, StoreError> {
    let mut problems = Vec::new();
    for (batch, live) in &fold.live {
        for input in &live.saved.inputs {
            let path = input_path(folder, *batch, input.pdf, &input.sha256);
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
