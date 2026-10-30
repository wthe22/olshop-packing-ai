//! Workspaces (app/workspaces.md): one folder of the data folder per workspace, one folder per
//! date inside it. The app lists, opens, creates, renames and deletes them.

use std::fmt;
use std::fs;
use std::path::Path;
use std::time::SystemTime;

use jiff::civil::Date;
use packing_engine::sorting::default_file_name;

use crate::StoreError;
use crate::date::DateFolder;
use crate::files::move_file;
use crate::journal::{JOURNAL, Journal};

/// A workspace name is at most this many characters.
pub const WORKSPACE_LIMIT: usize = 40;

/// The folders of a version 0.1 data folder, which are no names.
const RESERVED: [&str; 2] = ["labels", "categories"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorkspaceNameError {
    /// Empty, longer than [`WORKSPACE_LIMIT`], or not what the default file-name rule makes of it.
    NotAName,
    /// A folder of a version 0.1 data folder.
    Reserved,
    /// Another workspace has it, upper and lower case counting as the same.
    Taken,
}

impl fmt::Display for WorkspaceNameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            WorkspaceNameError::NotAName => {
                "1 to 40 letters, digits, spaces, - or _, with no space at either end"
            }
            WorkspaceNameError::Reserved => "that is a folder of Packing v0.1",
            WorkspaceNameError::Taken => "another workspace has that name",
        })
    }
}

/// Whether `name` can be a new workspace beside `existing`.
pub fn check_workspace_name(name: &str, existing: &[String]) -> Result<(), WorkspaceNameError> {
    let length = name.chars().count();
    if length == 0 || length > WORKSPACE_LIMIT || default_file_name(name) != name {
        return Err(WorkspaceNameError::NotAName);
    }
    if RESERVED
        .iter()
        .any(|reserved| name.eq_ignore_ascii_case(reserved))
    {
        return Err(WorkspaceNameError::Reserved);
    }
    if existing
        .iter()
        .any(|other| other.to_lowercase() == name.to_lowercase())
    {
        return Err(WorkspaceNameError::Taken);
    }
    Ok(())
}

/// The folders of the data folder whose names are workspace names, sorted.
pub fn workspaces(data_dir: &Path) -> Result<Vec<String>, StoreError> {
    let mut names: Vec<String> = subfolders(data_dir)?
        .into_iter()
        .filter(|name| check_workspace_name(name, &[]).is_ok())
        .collect();
    names.sort_by_key(|name| name.to_lowercase());
    Ok(names)
}

/// The dates of a workspace that have a folder, oldest first.
pub fn dates(data_dir: &Path, workspace: &str) -> Result<Vec<Date>, StoreError> {
    let mut dates: Vec<Date> = subfolders(&data_dir.join(workspace))?
        .into_iter()
        .filter_map(|name| name.parse::<Date>().ok())
        .collect();
    dates.sort();
    Ok(dates)
}

/// The workspace whose journal was written last, the one the app opens on at start.
pub fn last_written(data_dir: &Path) -> Result<Option<String>, StoreError> {
    let mut newest: Option<(SystemTime, String)> = None;
    for workspace in workspaces(data_dir)? {
        for date in dates(data_dir, &workspace)? {
            let journal = data_dir
                .join(&workspace)
                .join(date.to_string())
                .join(JOURNAL);
            let Ok(modified) = fs::metadata(&journal).and_then(|meta| meta.modified()) else {
                continue;
            };
            if newest.as_ref().is_none_or(|(time, _)| modified > *time) {
                newest = Some((modified, workspace.clone()));
            }
        }
    }
    Ok(newest.map(|(_, workspace)| workspace))
}

/// The sort rule of the workspace's newest batch: the newest live batch of the newest date that
/// has one. A damaged journal is passed over, as Search passes it over.
pub fn newest_batch_rule(data_dir: &Path, workspace: &str) -> Result<Option<String>, StoreError> {
    for date in dates(data_dir, workspace)?.into_iter().rev() {
        let journal = Journal::open(&DateFolder::new(data_dir, workspace, date))?;
        if journal.damage().is_some() {
            continue;
        }
        if let Some(live) = journal.fold().live.values().next_back() {
            return Ok(Some(live.saved.sort_rule.name.clone()));
        }
    }
    Ok(None)
}

/// Make a new workspace's folder, empty.
pub fn create_workspace(data_dir: &Path, name: &str) -> Result<(), StoreError> {
    let existing = workspaces(data_dir)?;
    check_workspace_name(name, &existing).map_err(|error| StoreError::BadName {
        name: name.to_string(),
        reason: error.to_string(),
    })?;
    let dir = data_dir.join(name);
    fs::create_dir_all(&dir).map_err(|e| StoreError::io(&dir, e))
}

/// Give a workspace another name: its folder is renamed, everything in it stays. Its journals keep
/// the name they were started under. The name may differ from the old one in case only.
pub fn rename_workspace(data_dir: &Path, name: &str, new_name: &str) -> Result<(), StoreError> {
    let others: Vec<String> = workspaces(data_dir)?
        .into_iter()
        .filter(|other| other != name)
        .collect();
    check_workspace_name(new_name, &others).map_err(|error| StoreError::BadName {
        name: new_name.to_string(),
        reason: error.to_string(),
    })?;
    move_file(&data_dir.join(name), &data_dir.join(new_name))
}

/// Move a workspace's folder, with all its dates, to the Recycle Bin.
pub fn delete_workspace(data_dir: &Path, name: &str) -> Result<(), StoreError> {
    let dir = data_dir.join(name);
    if !workspaces(data_dir)?.iter().any(|known| known == name) {
        return Err(StoreError::io(
            &dir,
            std::io::Error::from(std::io::ErrorKind::NotFound),
        ));
    }
    // The Recycle Bin is a COM call; a thread of its own starts COM in the mode it asks for,
    // whatever the calling thread started.
    let target = dir.clone();
    std::thread::spawn(move || trash::delete(&target))
        .join()
        .map_err(|_| StoreError::io(&dir, std::io::Error::other("the Recycle Bin call failed")))?
        .map_err(|error| StoreError::io(&dir, std::io::Error::other(error.to_string())))
}

fn subfolders(dir: &Path) -> Result<Vec<String>, StoreError> {
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
    Ok(names)
}
