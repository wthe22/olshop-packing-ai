//! A namespace's date folder, and what happens when it opens (journal.md, *When a date opens*):
//! fold, inputs, `output/.partial/`, outputs, views — in that order, before the date is shown.

use std::path::{Path, PathBuf};

use jiff::civil::Date;

use crate::StoreError;
use crate::inputs::{InputProblem, check_inputs};
use crate::journal::Journal;
use crate::outputs::{OutputProblem, Settled, check_outputs, settle_partial};
use crate::views::{views_current, write_views};

/// `{data_dir}/{namespace}/{date}/`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DateFolder {
    namespace: String,
    date: Date,
    dir: PathBuf,
}

impl DateFolder {
    pub fn new(data_dir: &Path, namespace: &str, date: Date) -> Self {
        Self {
            namespace: namespace.to_string(),
            date,
            dir: data_dir.join(namespace).join(date.to_string()),
        }
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn date(&self) -> Date {
        self.date
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }
}

/// An opened date: its journal and what the open-time checks found and did.
pub struct OpenedDate {
    pub journal: Journal,
    pub inputs: Vec<InputProblem>,
    pub settled: Settled,
    pub outputs: Vec<OutputProblem>,
    /// The views were missing or stale and were written again.
    pub views_written: bool,
}

/// Open a date. A damaged journal opens for viewing: the checks that only read still run, and
/// nothing is written — no `.partial/` settled, no view written.
pub fn open_date(folder: &DateFolder) -> Result<OpenedDate, StoreError> {
    let journal = Journal::open(folder)?;
    let dir = folder.dir();
    let fold = journal.fold();
    let writable = journal.damage().is_none();
    if writable && fold.last_seq == 0 {
        return Ok(OpenedDate {
            journal,
            inputs: Vec::new(),
            settled: Settled::Nothing,
            outputs: Vec::new(),
            views_written: false,
        });
    }
    let inputs = check_inputs(dir, fold)?;
    let settled = if writable {
        settle_partial(dir, fold)?
    } else {
        Settled::Nothing
    };
    let outputs = check_outputs(dir, fold)?;
    let views_written = writable && !views_current(dir, fold);
    if views_written {
        write_views(dir, fold)?;
    }
    Ok(OpenedDate {
        journal,
        inputs,
        settled,
        outputs,
        views_written,
    })
}
