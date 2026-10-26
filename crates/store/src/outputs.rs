//! `output/`: the outputs of an operation are written into `output/.partial/` and move in only
//! after the commit; old outputs an amend or a revert replaces wait in `.partial/replaced/`. When
//! a date opens, the journal's last record settles what an interrupted operation left there.

use std::fs;
use std::path::{Path, PathBuf};

use packing_engine::ids::BatchId;

use crate::StoreError;
use crate::files::{files_in, move_file, sha256_of};
use crate::fold::{Fold, LastWrites};

pub const OUTPUT_DIR: &str = "output";

pub fn partial_dir(date_dir: &Path) -> PathBuf {
    date_dir.join(OUTPUT_DIR).join(".partial")
}

pub fn replaced_dir(date_dir: &Path) -> PathBuf {
    partial_dir(date_dir).join("replaced")
}

/// Move these outputs from `output/` into `.partial/replaced/`. When Windows refuses one, the
/// files moved so far go back and the refused file is named.
pub fn move_aside(date_dir: &Path, files: &[String]) -> Result<(), StoreError> {
    let output = date_dir.join(OUTPUT_DIR);
    let replaced = replaced_dir(date_dir);
    fs::create_dir_all(&replaced).map_err(|e| StoreError::io(&replaced, e))?;
    let mut moved: Vec<&String> = Vec::new();
    for file in files {
        let from = output.join(file);
        if !from.is_file() {
            continue;
        }
        if let Err(error) = move_file(&from, &replaced.join(file)) {
            for back in moved {
                let _ = move_file(&replaced.join(back), &output.join(back));
            }
            return Err(error);
        }
        moved.push(file);
    }
    Ok(())
}

/// After the commit: every file of `.partial/` into `output/`, replacing one of the same name;
/// then `.partial/` goes with `replaced/`.
pub fn move_forward(date_dir: &Path) -> Result<usize, StoreError> {
    let partial = partial_dir(date_dir);
    let output = date_dir.join(OUTPUT_DIR);
    let files = files_in(&partial)?;
    for file in &files {
        move_file(&partial.join(file), &output.join(file))?;
    }
    remove_partial(&partial)?;
    Ok(files.len())
}

/// The files of `.partial/replaced/` back into `output/`; the rest of `.partial/` is deleted.
fn move_back(date_dir: &Path) -> Result<usize, StoreError> {
    let replaced = replaced_dir(date_dir);
    let output = date_dir.join(OUTPUT_DIR);
    let files = files_in(&replaced)?;
    for file in &files {
        move_file(&replaced.join(file), &output.join(file))?;
    }
    remove_partial(&partial_dir(date_dir))?;
    Ok(files.len())
}

/// Delete `.partial/` with what it holds: the outputs of an operation that did not commit.
pub fn discard_partial(date_dir: &Path) -> Result<(), StoreError> {
    remove_partial(&partial_dir(date_dir))
}

fn remove_partial(partial: &Path) -> Result<(), StoreError> {
    match fs::remove_dir_all(partial) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(StoreError::io(partial, error)),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Settled {
    /// No `.partial/` was left.
    Nothing,
    /// The last record wrote what `.partial/` holds: its files moved into `output/`.
    Forward { moved: usize },
    /// `.partial/` held an operation that was not committed: the replaced files went back.
    Back { restored: usize },
}

/// Settle `output/.partial/` by the journal's last record (journal.md, *When a date opens*).
pub fn settle_partial(date_dir: &Path, fold: &Fold) -> Result<Settled, StoreError> {
    let partial = partial_dir(date_dir);
    if !partial.is_dir() {
        return Ok(Settled::Nothing);
    }
    if wrote_what_partial_holds(date_dir, fold)? {
        let moved = move_forward(date_dir)?;
        Ok(Settled::Forward { moved })
    } else {
        let restored = move_back(date_dir)?;
        Ok(Settled::Back { restored })
    }
}

/// Whether `.partial/` holds the last record's operation and nothing else: a later operation that
/// did not commit leaves files the record does not name, or the record's files moved aside.
fn wrote_what_partial_holds(date_dir: &Path, fold: &Fold) -> Result<bool, StoreError> {
    let partial = partial_dir(date_dir);
    let output = date_dir.join(OUTPUT_DIR);
    let held = files_in(&partial)?;
    match &fold.last_writes {
        LastWrites::Nothing => Ok(false),
        LastWrites::Files(files) => {
            if !held
                .iter()
                .all(|name| files.iter().any(|(file, _)| file == name))
            {
                return Ok(false);
            }
            for (file, sha256) in files {
                let in_partial = partial.join(file);
                let in_output = output.join(file);
                let holds = |path: &Path| -> Result<bool, StoreError> {
                    Ok(path.is_file() && sha256_of(path)? == *sha256)
                };
                if !holds(&in_partial)? && !holds(&in_output)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        LastWrites::Reverted(files) => {
            let replaced = files_in(&replaced_dir(date_dir))?;
            Ok(held.is_empty() && files.iter().all(|file| replaced.contains(file)))
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputState {
    Missing,
    /// Its size is not the one its newest record names.
    Changed,
}

/// An output of a live batch that is not as its record says: its card offers **Restore**.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutputProblem {
    pub batch: BatchId,
    pub file: String,
    pub state: OutputState,
}

/// Every output of a live batch, by existence and size.
pub fn check_outputs(date_dir: &Path, fold: &Fold) -> Result<Vec<OutputProblem>, StoreError> {
    let output = date_dir.join(OUTPUT_DIR);
    let mut problems = Vec::new();
    for (batch, live) in &fold.live {
        for file in &live.saved.outputs {
            let path = output.join(&file.file);
            let state = match fs::metadata(&path) {
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    Some(OutputState::Missing)
                }
                Err(error) => return Err(StoreError::io(&path, error)),
                Ok(meta) if meta.len() != live.output_size(file) => Some(OutputState::Changed),
                Ok(_) => None,
            };
            if let Some(state) = state {
                problems.push(OutputProblem {
                    batch: *batch,
                    file: file.file.clone(),
                    state,
                });
            }
        }
    }
    Ok(problems)
}
