//! The search index of the open workspace (reprint.md): built on its own thread when the workspace
//! opens, so the window never waits for it; each commit's date is taken in again from its fold. A
//! fold that arrives while the build runs is applied when the build ends, so no commit is lost.
//! A date another copy of the app committed to is found by its journal's length and taken in again.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use jiff::civil::Date;
use packing_store::{DateFolder, Fold, Journal, SearchIndex, StoreError, journal_lengths};

#[derive(Default)]
enum Built {
    #[default]
    Loading,
    Ready(SearchIndex),
    Failed(String),
}

#[derive(Default)]
struct Holder {
    workspace: String,
    built: Built,
    /// The folds committed while the build ran.
    pending: Vec<Fold>,
    /// Each journal's length when the index last read it.
    lengths: BTreeMap<Date, u64>,
}

#[derive(Clone, Default)]
pub(crate) struct Index(Arc<Mutex<Holder>>);

impl Index {
    /// Fold every journal of `workspace` on a thread of its own; the index of another workspace
    /// is dropped at once.
    pub(crate) fn rebuild(&self, data_dir: PathBuf, workspace: String) {
        *self.0.lock().unwrap() = Holder {
            workspace: workspace.clone(),
            ..Holder::default()
        };
        let holder = self.0.clone();
        std::thread::spawn(move || {
            let built = journal_lengths(&data_dir, &workspace).and_then(|lengths| {
                SearchIndex::build(&data_dir, &workspace).map(|index| (index, lengths))
            });
            let mut holder = holder.lock().unwrap();
            if holder.workspace != workspace {
                return;
            }
            holder.built = match built {
                Ok((mut index, lengths)) => {
                    for fold in holder.pending.drain(..) {
                        index.refresh(&fold);
                    }
                    holder.lengths = lengths;
                    Built::Ready(index)
                }
                Err(error) => Built::Failed(error.to_string()),
            };
        });
    }

    /// Take a committed date in again.
    pub(crate) fn refresh(&self, fold: &Fold) {
        let mut holder = self.0.lock().unwrap();
        if holder.workspace != fold.workspace {
            return;
        }
        match &mut holder.built {
            Built::Ready(index) => index.refresh(fold),
            Built::Loading => holder.pending.push(fold.clone()),
            Built::Failed(_) => {}
        }
    }

    /// Take in again each date whose journal changed since the index last read it, this copy's
    /// commits included; whether a date came, went or changed. Nothing while the index is built.
    pub(crate) fn take_in_changes(&self, data_dir: &Path) -> Result<bool, StoreError> {
        let mut holder = self.0.lock().unwrap();
        let Holder {
            workspace,
            built: Built::Ready(index),
            lengths,
            ..
        } = &mut *holder
        else {
            return Ok(false);
        };
        let now = journal_lengths(data_dir, workspace)?;
        if now == *lengths {
            return Ok(false);
        }
        for (date, length) in &now {
            if lengths.get(date) != Some(length) {
                index.take_in(&Journal::open(&DateFolder::new(
                    data_dir, workspace, *date,
                ))?);
            }
        }
        *lengths = now;
        Ok(true)
    }

    /// Run `f` on the index; `None` while it is being built.
    pub(crate) fn with<R>(&self, f: impl FnOnce(&SearchIndex) -> R) -> Result<Option<R>, String> {
        match &self.0.lock().unwrap().built {
            Built::Loading => Ok(None),
            Built::Ready(index) => Ok(Some(f(index))),
            Built::Failed(error) => Err(format!("The search index could not be built: {error}")),
        }
    }

    /// Wait until the build has ended; for the tests, which search right after a start.
    #[cfg(test)]
    pub(crate) fn wait(&self) {
        while matches!(self.0.lock().unwrap().built, Built::Loading) {
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }
}
