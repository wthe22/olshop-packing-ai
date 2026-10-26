//! Workspaces (app/workspaces.md): one folder of the data folder per workspace, one folder per
//! date inside it. The app lists, opens and creates them; it never renames or deletes one.

use std::fmt;
use std::fs;
use std::path::Path;
use std::time::SystemTime;

use jiff::civil::Date;
use packing_engine::sorting::default_file_name;

use crate::StoreError;
use crate::journal::JOURNAL;

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
