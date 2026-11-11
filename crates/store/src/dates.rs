//! The dates of the data folder (accounts.md, *Listing and opening dates*): one folder per date for
//! every account, and what the panel and New batch read from their journals.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use jiff::civil::Date;

use crate::StoreError;
use crate::date::DateFolder;
use crate::journal::{JOURNAL, Journal};

/// The dates that have a folder, oldest first.
pub fn dates(data_dir: &Path) -> Result<Vec<Date>, StoreError> {
    let mut dates: Vec<Date> = subfolders(data_dir)?
        .iter()
        .filter_map(|name| date_named(name))
        .collect();
    dates.sort();
    Ok(dates)
}

/// The date a folder's name is, written as the app writes it (`2026-10-07`).
pub(crate) fn date_named(name: &str) -> Option<Date> {
    name.parse::<Date>()
        .ok()
        .filter(|date| date.to_string() == name)
}

/// Each date with its journal's length (0 before its first commit): a date another copy of the app
/// committed to has a new one, as the journal only grows.
pub fn journal_lengths(data_dir: &Path) -> Result<BTreeMap<Date, u64>, StoreError> {
    let mut lengths = BTreeMap::new();
    for date in dates(data_dir)? {
        let journal = DateFolder::new(data_dir, date).dir().join(JOURNAL);
        let length = match fs::metadata(&journal) {
            Ok(meta) => meta.len(),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => 0,
            Err(error) => return Err(StoreError::io(&journal, error)),
        };
        lengths.insert(date, length);
    }
    Ok(lengths)
}

/// The sort rule of the newest batch: the newest live batch of the newest date that has one. A
/// damaged journal is passed over, as Search passes it over.
pub fn newest_batch_rule(data_dir: &Path) -> Result<Option<String>, StoreError> {
    for date in dates(data_dir)?.into_iter().rev() {
        let journal = Journal::open(&DateFolder::new(data_dir, date))?;
        if journal.damage().is_some() {
            continue;
        }
        if let Some(live) = journal.fold().live.values().next_back() {
            return Ok(Some(live.saved.sort_rule.name.clone()));
        }
    }
    Ok(None)
}

/// Whether a date has a group PDF of a live batch that is not marked Printed yet: the panel's mark
/// under today. A damaged journal, or a date with no journal, has none.
pub fn has_unprinted_groups(data_dir: &Path, date: Date) -> Result<bool, StoreError> {
    let journal = Journal::open(&DateFolder::new(data_dir, date))?;
    if journal.damage().is_some() {
        return Ok(false);
    }
    Ok(journal.fold().live.values().any(|live| {
        live.saved
            .outputs
            .iter()
            .filter_map(|output| output.group.as_deref())
            .any(|code| !live.printed.contains(code))
    }))
}

/// The names of the folders in `dir`; none when it is not there.
pub(crate) fn subfolders(dir: &Path) -> Result<Vec<String>, StoreError> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(StoreError::io(dir, error)),
    };
    let mut names = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| StoreError::io(dir, e))?;
        if entry
            .file_type()
            .map_err(|e| StoreError::io(dir, e))?
            .is_dir()
        {
            names.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    names.sort_by_key(|name| name.to_lowercase());
    Ok(names)
}
