//! Data of an older version in the data folder (accounts.md, *Data of older versions*): found and
//! named for the Day screen's note, never read, changed or deleted.

use std::path::Path;

use crate::StoreError;
use crate::dates::{date_named, subfolders};
use crate::journal::JOURNAL;

/// What a v0.1 data folder holds at its top.
const V0_1_NAMES: [&str; 5] = [
    "labels",
    "state.json",
    "couriers.toml",
    "categories.toml",
    "categories",
];

/// What a v0.4 workspace folder holds besides its dates.
const V0_4_SETTINGS: &str = "settings.toml";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OlderVersion {
    V0_1,
    V0_4,
}

/// One older version's data, by the files and folders of the data folder that show it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OlderData {
    pub version: OlderVersion,
    /// Sorted, upper and lower case counting as the same.
    pub names: Vec<String>,
}

/// The older versions whose data the data folder holds, oldest version first.
pub fn older_data(data_dir: &Path) -> Result<Vec<OlderData>, StoreError> {
    let mut found = Vec::new();
    let mut v0_1: Vec<String> = V0_1_NAMES
        .iter()
        .filter(|name| data_dir.join(name).exists())
        .map(|name| name.to_string())
        .collect();
    v0_1.sort();
    if !v0_1.is_empty() {
        found.push(OlderData {
            version: OlderVersion::V0_1,
            names: v0_1,
        });
    }
    let mut v0_4 = Vec::new();
    for name in subfolders(data_dir)? {
        if date_named(&name).is_none()
            && !V0_1_NAMES.contains(&name.as_str())
            && is_workspace(&data_dir.join(&name))?
        {
            v0_4.push(name);
        }
    }
    if !v0_4.is_empty() {
        found.push(OlderData {
            version: OlderVersion::V0_4,
            names: v0_4,
        });
    }
    Ok(found)
}

/// A v0.4 workspace: its shops' `settings.toml`, or a date folder with a journal.
fn is_workspace(dir: &Path) -> Result<bool, StoreError> {
    if dir.join(V0_4_SETTINGS).is_file() {
        return Ok(true);
    }
    Ok(subfolders(dir)?
        .iter()
        .any(|name| date_named(name).is_some() && dir.join(name).join(JOURNAL).is_file()))
}
