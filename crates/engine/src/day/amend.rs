//! Undo and redo the day's last batch, reusing its numbers (07 › *Revert and amend*).

use std::path::{Path, PathBuf};

use crate::groupings::Grouping;
use crate::ids::BatchNumber;
use crate::names;

use super::state::{BatchEntry, DayState};

impl DayState {
    /// Undo the last batch: remove it and its orders from the state. Returns the removed batch
    /// (the caller deletes its folder with [`delete_batch_folder`](super::delete_batch_folder));
    /// the numbers are reused (07 › *Revert and amend*).
    pub fn revert_last(&mut self) -> Option<BatchEntry> {
        let last = self.batches.iter().map(|batch| batch.batch).max()?;
        let index = self.batches.iter().position(|batch| batch.batch == last)?;
        let removed = self.batches.remove(index);
        self.saved.retain(|order| order.batch != last);
        Some(removed)
    }

    /// Start an amend of the last batch (07 › *Revert and amend*): the `download/` file paths in
    /// their stored order, and the state with the last batch removed (the duplicate guard then
    /// ignores the batch's own orders, and the numbers are reused).
    pub fn amend(&self, day_dir: &Path) -> Option<Amend> {
        let last = self.batches.iter().map(|batch| batch.batch).max()?;
        let entry = self
            .batches
            .iter()
            .find(|batch| batch.batch == last)?
            .clone();
        let download = day_dir
            .join(names::batch_folder(last))
            .join(names::DOWNLOAD_DIR);
        let files = entry
            .files
            .iter()
            .map(|file| download.join(&file.name))
            .collect();
        let mut state = self.clone();
        state.revert_last();
        Some(Amend {
            batch: last,
            grouping: entry.grouping,
            files,
            state,
        })
    }
}

/// An amend in progress: the last batch's `download/` files and the state without it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Amend {
    pub batch: BatchNumber,
    /// The grouping the batch was packed with (04 › *Sets*): an amend reads it again, whatever
    /// grouping was used last.
    pub grouping: Grouping,
    pub files: Vec<PathBuf>,
    pub state: DayState,
}
