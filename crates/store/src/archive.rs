//! Archive (archive.md): a date kept without its PDFs. Archiving deletes every PDF directly in the
//! date folder, the live batches' inputs last, so a date that stops half way is not archived and
//! can be archived again. Nothing is recorded: a date is archived when it has a live batch and
//! none of their inputs is in its folder.

use std::fmt;
use std::fs;
use std::path::Path;

use jiff::civil::Date;

use crate::StoreError;
use crate::date::DateFolder;
use crate::files::files_in;
use crate::inputs::live_inputs;
use crate::journal::Journal;
use crate::outputs::partial_dir;

/// Whether a date folder holding `live_inputs` (its live batches' input file names) is archived.
pub fn is_archived(date_dir: &Path, live_inputs: &[String]) -> bool {
    !live_inputs.is_empty()
        && live_inputs
            .iter()
            .all(|name| !date_dir.join(name).is_file())
}

/// Why a date is not archived.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotArchivable {
    Today,
    /// `.partial/` holds an operation that is settled when the date opens.
    Partial,
    NoBatch,
    Archived,
    Damaged,
}

impl fmt::Display for NotArchivable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            NotArchivable::Today => "today is never archived",
            NotArchivable::Partial => {
                "an operation that did not finish waits in .partial; open the date first"
            }
            NotArchivable::NoBatch => "it has no saved batch",
            NotArchivable::Archived => "it is archived already",
            NotArchivable::Damaged => "its journal is damaged",
        })
    }
}

/// A PDF directly in a date folder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PdfFile {
    pub name: String,
    pub bytes: u64,
}

/// What archiving a date deletes, in order: every other PDF, then the live batches' inputs.
#[derive(Debug)]
pub struct ArchivePlan {
    pub folder: DateFolder,
    pub pdfs: Vec<PdfFile>,
    live_inputs: Vec<String>,
}

impl ArchivePlan {
    pub fn bytes(&self) -> u64 {
        self.pdfs.iter().map(|pdf| pdf.bytes).sum()
    }
}

/// What archiving a date did.
#[derive(Debug, PartialEq, Eq)]
pub struct Archived {
    pub deleted: Vec<String>,
    /// The PDFs Windows refused to delete (most often open in a viewer); they stay.
    pub left: Vec<String>,
    /// None of its live inputs is left: the date is archived now.
    pub archived: bool,
}

/// What archiving `folder`'s date on `today` would delete; a date that is not archived is refused
/// with the reason.
pub fn plan_archive(folder: &DateFolder, today: Date) -> Result<ArchivePlan, StoreError> {
    let refuse = |reason| Err(StoreError::NotArchivable(reason));
    if folder.date() == today {
        return refuse(NotArchivable::Today);
    }
    let dir = folder.dir();
    if partial_dir(dir).is_dir() {
        return refuse(NotArchivable::Partial);
    }
    let journal = Journal::open(folder)?;
    if journal.damage().is_some() {
        return refuse(NotArchivable::Damaged);
    }
    let live_inputs = live_inputs(journal.fold());
    if live_inputs.is_empty() {
        return refuse(NotArchivable::NoBatch);
    }
    if is_archived(dir, &live_inputs) {
        return refuse(NotArchivable::Archived);
    }
    let mut pdfs = Vec::new();
    for name in files_in(dir)? {
        if !name.to_lowercase().ends_with(".pdf") {
            continue;
        }
        let path = dir.join(&name);
        let bytes = fs::metadata(&path)
            .map_err(|e| StoreError::io(&path, e))?
            .len();
        pdfs.push(PdfFile { name, bytes });
    }
    pdfs.sort_by_key(|pdf| live_inputs.contains(&pdf.name));
    Ok(ArchivePlan {
        folder: folder.clone(),
        pdfs,
        live_inputs,
    })
}

/// Delete the plan's PDFs in its order; the caller holds the data folder's lock. A PDF that cannot
/// be deleted is left and named, and the rest go on.
pub fn archive(plan: ArchivePlan) -> Archived {
    let dir = plan.folder.dir();
    let (mut deleted, mut left) = (Vec::new(), Vec::new());
    for pdf in plan.pdfs {
        match fs::remove_file(dir.join(&pdf.name)) {
            Ok(()) => deleted.push(pdf.name),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => left.push(pdf.name),
        }
    }
    Archived {
        archived: is_archived(dir, &plan.live_inputs),
        deleted,
        left,
    }
}
