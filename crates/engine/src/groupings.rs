//! The groupings of a data folder: the grouping list, the file
//! of a grouping and the grouping-name rules.
//!
//! The grouping `default` is the data folder's own `categories.toml` — the file the app already
//! writes, never moved (so an existing data folder works as it does now). Every other grouping is
//! `<data folder>/categories/<grouping name>.toml`, the file name being the grouping name, so a
//! grouping can be copied, renamed or written by hand. A grouping with no file reads as the one
//! catch-all entry ([`catch_all_category`]), so a batch packed with it saves every order in one
//! PDF set.

use std::fmt;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::names;
use crate::rules::{Category, catch_all_category, load_rules_named};

/// The data folder's own rules file: the grouping `default`.
pub const CATEGORIES_FILE: &str = "categories.toml";
/// The data folder subfolder holding every other grouping, one file each.
pub const GROUPINGS_DIR: &str = "categories";
/// The grouping every data folder has, kept in [`CATEGORIES_FILE`].
pub const DEFAULT_GROUPING: &str = "default";
/// The extension of a grouping's file; the grouping name is the file name without it.
const GROUPING_EXTENSION: &str = ".toml";

/// A grouping name: what a batch and the settings store, and what the file of a grouping is
/// named from.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Grouping(String);

impl Grouping {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for Grouping {
    fn default() -> Self {
        Self(DEFAULT_GROUPING.to_string())
    }
}

impl fmt::Display for Grouping {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A data folder's groupings could not be read or written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GroupingError {
    /// A grouping name that is (or becomes, after sanitising) empty.
    EmptyName,
    /// The name `default` belongs to the data folder's own [`CATEGORIES_FILE`]: it is neither
    /// created, nor deleted, nor renamed.
    ReservedDefault,
    /// `categories/default.toml` sits beside the data folder's own `categories.toml`; both files
    /// are named.
    DefaultFile { default: String, file: String },
    /// A new grouping's file is already there.
    Exists { file: String },
    /// A grouping's file (or the `categories` folder) cannot be read.
    Read { file: String, message: String },
    /// A grouping's file cannot be written or deleted.
    Write { file: String, message: String },
    /// A grouping's file does not load; the message names the file, line and column.
    Rules { message: String },
}

impl fmt::Display for GroupingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GroupingError::EmptyName => f.write_str("a grouping name cannot be empty"),
            GroupingError::ReservedDefault => write!(
                f,
                "the name \"{}\" is the data folder's own {CATEGORIES_FILE}: it cannot be created, deleted or renamed",
                Grouping::default()
            ),
            GroupingError::DefaultFile { default, file } => write!(
                f,
                "{file} and {default} are one grouping: the name \"{}\" belongs to the data folder's own {CATEGORIES_FILE}",
                Grouping::default()
            ),
            GroupingError::Exists { file } => {
                write!(f, "cannot create {file}: it is already there")
            }
            GroupingError::Read { file, message } => write!(f, "cannot read {file}: {message}"),
            GroupingError::Write { file, message } => {
                write!(f, "Cannot write to {file}: {message}.")
            }
            GroupingError::Rules { message } => f.write_str(message),
        }
    }
}

impl std::error::Error for GroupingError {}

/// One grouping of a data folder: its name and the file it is kept in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupingEntry {
    pub name: Grouping,
    /// The file, relative to the data folder: `categories.toml` for `default`, else
    /// `categories/<grouping name>.toml`.
    pub file: String,
}

/// A grouping name as a file name: characters Windows forbids in a name (`\ / : * ? " < > |`)
/// become `-`.
fn sanitize_name(name: &str) -> String {
    names::sanitize_file_name(name.trim())
}

/// The file of a grouping, relative to the data folder. The grouping `default` is the data
/// folder's own [`CATEGORIES_FILE`]; every other grouping is `categories/<grouping name>.toml`.
pub fn file_of(grouping: &Grouping) -> Result<String, GroupingError> {
    let name = sanitize_name(grouping.as_str());
    if name.is_empty() {
        return Err(GroupingError::EmptyName);
    }
    if name == DEFAULT_GROUPING {
        return Ok(CATEGORIES_FILE.to_string());
    }
    Ok(format!("{GROUPINGS_DIR}/{name}{GROUPING_EXTENSION}"))
}

/// The grouping list of a data folder: `default` always, then every `<name>.toml`
/// in its `categories/` folder, in file-name order. A file `categories/default.toml` — the name
/// `default` again — is refused, naming both files.
pub fn list(folder: &Path) -> Result<Vec<GroupingEntry>, GroupingError> {
    let mut groupings = vec![GroupingEntry {
        name: Grouping::default(),
        file: CATEGORIES_FILE.to_string(),
    }];
    let dir = folder.join(GROUPINGS_DIR);
    let mut names: Vec<String> = Vec::new();
    match std::fs::read_dir(&dir) {
        Ok(entries) => {
            for entry in entries {
                let entry = entry.map_err(|error| GroupingError::Read {
                    file: GROUPINGS_DIR.to_string(),
                    message: error.to_string(),
                })?;
                if let Some(name) = grouping_file(&entry)? {
                    names.push(name);
                }
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(GroupingError::Read {
                file: GROUPINGS_DIR.to_string(),
                message: error.to_string(),
            });
        }
    }
    names.sort();
    for name in names {
        groupings.push(GroupingEntry {
            file: format!("{GROUPINGS_DIR}/{name}{GROUPING_EXTENSION}"),
            name: Grouping::new(name),
        });
    }
    Ok(groupings)
}

/// A grouping's categories: its file's entries, or the one catch-all entry when the grouping has
/// no file. A file that does not load gives [`GroupingError::Rules`] naming the
/// file.
pub fn load(folder: &Path, grouping: &Grouping) -> Result<Vec<Category>, GroupingError> {
    let file = file_of(grouping)?;
    let path = folder.join(&file);
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(vec![catch_all_category()]);
        }
        Err(error) => {
            return Err(GroupingError::Read {
                file,
                message: error.to_string(),
            });
        }
    };
    load_rules_named(&text, &file).map_err(|error| GroupingError::Rules {
        message: error.to_string(),
    })
}

/// Write a new grouping's file: a copy of `copy_from`'s file when one is given, else the one
/// catch-all entry, so the new grouping reads like a grouping whose categories are not set yet. The
/// name `default` is the data folder's own `categories.toml` and cannot be
/// created, and a file already there is refused, so nothing the owner wrote is overwritten.
pub fn create(
    folder: &Path,
    grouping: &Grouping,
    copy_from: Option<&Grouping>,
) -> Result<(), GroupingError> {
    let file = file_of(grouping)?;
    if file == CATEGORIES_FILE {
        return Err(GroupingError::ReservedDefault);
    }
    let path = folder.join(&file);
    if path.exists() {
        return Err(GroupingError::Exists { file });
    }
    let text = match copy_from {
        Some(from) => copied_text(folder, from)?,
        None => catch_all_text(),
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| GroupingError::Write {
            file: file.clone(),
            message: error.to_string(),
        })?;
    }
    std::fs::write(&path, text).map_err(|error| GroupingError::Write {
        file,
        message: error.to_string(),
    })
}

/// Delete a grouping's file, so its name leaves the grouping list; a grouping with
/// no file is already gone. The grouping `default` is the data folder's own `categories.toml`
/// and cannot be deleted.
pub fn delete(folder: &Path, grouping: &Grouping) -> Result<(), GroupingError> {
    let file = file_of(grouping)?;
    if file == CATEGORIES_FILE {
        return Err(GroupingError::ReservedDefault);
    }
    let path = folder.join(&file);
    match std::fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(GroupingError::Write {
            file,
            message: error.to_string(),
        }),
    }
}

/// The bytes a copy of `from` starts from: its file, or its one catch-all entry when that
/// grouping has no file of its own.
fn copied_text(folder: &Path, from: &Grouping) -> Result<String, GroupingError> {
    let file = file_of(from)?;
    match std::fs::read_to_string(folder.join(&file)) {
        Ok(text) => Ok(text),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(catch_all_text()),
        Err(error) => Err(GroupingError::Read {
            file,
            message: error.to_string(),
        }),
    }
}

/// The grouping name `entry` holds, or `None` when it is not a `<name>.toml` file; a file named
/// `default.toml` is refused.
fn grouping_file(entry: &std::fs::DirEntry) -> Result<Option<String>, GroupingError> {
    if !entry
        .file_type()
        .map(|kind| kind.is_file())
        .unwrap_or(false)
    {
        return Ok(None);
    }
    let file_name = entry.file_name();
    let Some(file_name) = file_name.to_str() else {
        return Ok(None);
    };
    let Some(name) = file_name.strip_suffix(GROUPING_EXTENSION) else {
        return Ok(None);
    };
    if name == DEFAULT_GROUPING {
        return Err(GroupingError::DefaultFile {
            default: CATEGORIES_FILE.to_string(),
            file: format!("{GROUPINGS_DIR}/{file_name}"),
        });
    }
    Ok(Some(name.to_string()))
}

/// The one catch-all entry as a rules file's text: code `Z`, name `Lainnya`, no
/// condition, so it takes every order. A new grouping — and the copy of a grouping with no file
/// — starts from it.
fn catch_all_text() -> String {
    let category = catch_all_category();
    format!(
        "[[category]]\ncode = \"{}\"\nname = \"{}\"\n",
        category.code, category.name
    )
}
