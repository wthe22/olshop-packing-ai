//! The search index of the open workspace (reprint.md): built on its own thread when the workspace
//! opens, so the window never waits for it; each commit's date is taken in again from its fold. A
//! fold that arrives while the build runs is applied when the build ends, so no commit is lost.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use packing_store::{Fold, SearchIndex};

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
}

#[derive(Clone, Default)]
pub(crate) struct Index(Arc<Mutex<Holder>>);

impl Index {
    /// Fold every journal of `workspace` on a thread of its own; the index of another workspace
    /// is dropped at once.
    pub(crate) fn rebuild(&self, data_dir: PathBuf, workspace: String) {
        *self.0.lock().unwrap() = Holder {
            workspace: workspace.clone(),
            built: Built::Loading,
            pending: Vec::new(),
        };
        let holder = self.0.clone();
        std::thread::spawn(move || {
            let built = SearchIndex::build(&data_dir, &workspace);
            let mut holder = holder.lock().unwrap();
            if holder.workspace != workspace {
                return;
            }
            holder.built = match built {
                Ok(mut index) => {
                    for fold in holder.pending.drain(..) {
                        index.refresh(&fold);
                    }
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
