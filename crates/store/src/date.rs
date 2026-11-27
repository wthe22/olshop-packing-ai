//! A date folder of the data folder, and what happens when it opens (journal.md, *When a date opens*):
//! fold, inputs, `.partial/`, outputs, views — in that order, before the date is shown. An archived
//! date has no inputs and outputs to check.

use std::path::{Path, PathBuf};

use jiff::civil::Date;

use crate::StoreError;
use crate::archive::is_archived;
use crate::inputs::{InputProblem, check_inputs, live_inputs};
use crate::journal::Journal;
use crate::outputs::{OutputProblem, Settled, check_outputs, settle_partial};
use crate::views::{views_current, write_views};

/// `{data_dir}/{date}/`: the work of one date for every account.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DateFolder {
    date: Date,
    dir: PathBuf,
}

impl DateFolder {
    pub fn new(data_dir: &Path, date: Date) -> Self {
        Self {
            date,
            dir: data_dir.join(date.to_string()),
        }
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
    /// Its PDFs were removed (archive.md): its inputs and outputs are not checked.
    pub archived: bool,
    pub inputs: Vec<InputProblem>,
    pub settled: Settled,
    pub outputs: Vec<OutputProblem>,
    /// The views were missing or stale and were written again.
    pub views_written: bool,
}

/// Open a date while no other copy of the app writes to the data folder (the caller holds its
/// lock). A damaged journal opens for viewing: the checks that only read still run, and nothing is
/// written — no `.partial/` settled, no view written.
pub fn open_date(folder: &DateFolder) -> Result<OpenedDate, StoreError> {
    open(folder, true)
}

/// Open a date for viewing while another copy of the app may be writing to it: the checks that
/// only read run, and `.partial/` and the views are left to the copy that writes.
pub fn view_date(folder: &DateFolder) -> Result<OpenedDate, StoreError> {
    open(folder, false)
}

fn open(folder: &DateFolder, may_write: bool) -> Result<OpenedDate, StoreError> {
    let journal = Journal::open(folder)?;
    let dir = folder.dir();
    let fold = journal.fold();
    let writable = may_write && journal.damage().is_none();
    if journal.damage().is_none() && fold.last_seq == 0 {
        return Ok(OpenedDate {
            journal,
            archived: false,
            inputs: Vec::new(),
            settled: Settled::Nothing,
            outputs: Vec::new(),
            views_written: false,
        });
    }
    let archived = is_archived(dir, &live_inputs(fold));
    let inputs = match archived {
        true => Vec::new(),
        false => check_inputs(folder, fold)?,
    };
    let settled = if writable {
        settle_partial(dir, fold)?
    } else {
        Settled::Nothing
    };
    let outputs = match archived {
        true => Vec::new(),
        false => check_outputs(dir, fold)?,
    };
    let views_written = writable && !views_current(dir, fold);
    if views_written {
        write_views(dir, fold)?;
    }
    Ok(OpenedDate {
        journal,
        archived,
        inputs,
        settled,
        outputs,
        views_written,
    })
}
