//! The sets of categories of a data folder (04 › *Sets*; 07 › *Rules file*): the set list, the
//! file of a set and the set-name rules.
//!
//! The set `default` is the data folder's own `categories.toml` — the file the app already
//! writes, never moved (so an existing data folder works as it does now). Every other set is
//! `<data folder>/categories/<set name>.toml`, the file name being the set name, so a set can be
//! copied, renamed or written by hand. A set with no file reads as the one catch-all entry
//! ([`catch_all_category`]), so a batch packed with it saves every order in one PDF set.

use std::fmt;
use std::path::{Path, PathBuf};

use crate::names;
use crate::rules::{Category, catch_all_category, load_rules_named};

/// The data folder's own rules file: the set `default` (07 › *Rules file*).
pub const CATEGORIES_FILE: &str = "categories.toml";
/// The data folder subfolder holding every other set, one file each.
pub const SETS_DIR: &str = "categories";
/// The set every data folder has, kept in [`CATEGORIES_FILE`].
pub const DEFAULT_SET: &str = "default";
/// The extension of a set's file; the set name is the file name without it.
pub const SET_EXTENSION: &str = ".toml";

/// A data folder's sets could not be read or written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SetError {
    /// A set name that is (or becomes, after sanitising) empty.
    EmptyName,
    /// The name `default` belongs to the data folder's own [`CATEGORIES_FILE`]: it is neither
    /// created, nor deleted, nor renamed.
    ReservedDefault,
    /// `categories/default.toml` sits beside the data folder's own `categories.toml`; both files
    /// are named.
    DefaultFile { default: String, file: String },
    /// A new set's file is already there.
    Exists { file: String },
    /// A set's file (or the `categories` folder) cannot be read.
    Read { file: String, message: String },
    /// A set's file cannot be written or deleted.
    Write { file: String, message: String },
    /// A set's file does not load; the message names the file, line and column.
    Rules { message: String },
}

impl fmt::Display for SetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SetError::EmptyName => f.write_str("a grouping name cannot be empty"),
            SetError::ReservedDefault => write!(
                f,
                "the name \"{DEFAULT_SET}\" is the data folder's own {CATEGORIES_FILE}: it cannot be created, deleted or renamed"
            ),
            SetError::DefaultFile { default, file } => write!(
                f,
                "{file} and {default} are one grouping: the name \"{DEFAULT_SET}\" belongs to the data folder's own {CATEGORIES_FILE}"
            ),
            SetError::Exists { file } => write!(f, "cannot create {file}: it is already there"),
            SetError::Read { file, message } => write!(f, "cannot read {file}: {message}"),
            SetError::Write { file, message } => write!(f, "Cannot write to {file}: {message}."),
            SetError::Rules { message } => f.write_str(message),
        }
    }
}

impl std::error::Error for SetError {}

/// One set of a data folder: its name and the file it is kept in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Set {
    pub name: String,
    /// The file, relative to the data folder: `categories.toml` for `default`, else
    /// `categories/<set name>.toml`.
    pub file: String,
}

/// A set name as a file name: characters Windows forbids in a name (`\ / : * ? " < > |`) become
/// `-` (04 › *Sets*).
pub fn sanitize_name(name: &str) -> String {
    names::sanitize_file_name(name.trim())
}

/// The file of a set, relative to the data folder. The set `default` is the data folder's own
/// [`CATEGORIES_FILE`]; every other set is `categories/<set name>.toml`.
pub fn file_of(name: &str) -> Result<String, SetError> {
    let name = sanitize_name(name);
    if name.is_empty() {
        return Err(SetError::EmptyName);
    }
    if name == DEFAULT_SET {
        return Ok(CATEGORIES_FILE.to_string());
    }
    Ok(format!("{SETS_DIR}/{name}{SET_EXTENSION}"))
}

/// The path of a set's file inside `folder`.
pub fn path_of(folder: &Path, name: &str) -> Result<PathBuf, SetError> {
    Ok(folder.join(file_of(name)?))
}

/// The set list of a data folder (04 › *Sets*): `default` always, then every `<name>.toml` in its
/// `categories/` folder, in file-name order. A file `categories/default.toml` — the name
/// `default` again — is refused, naming both files.
pub fn list(folder: &Path) -> Result<Vec<Set>, SetError> {
    let mut sets = vec![Set {
        name: DEFAULT_SET.to_string(),
        file: CATEGORIES_FILE.to_string(),
    }];
    let dir = folder.join(SETS_DIR);
    let mut names: Vec<String> = Vec::new();
    match std::fs::read_dir(&dir) {
        Ok(entries) => {
            for entry in entries {
                let entry = entry.map_err(|error| SetError::Read {
                    file: SETS_DIR.to_string(),
                    message: error.to_string(),
                })?;
                if !entry
                    .file_type()
                    .map(|kind| kind.is_file())
                    .unwrap_or(false)
                {
                    continue;
                }
                let file_name = entry.file_name();
                let Some(file_name) = file_name.to_str() else {
                    continue;
                };
                let Some(name) = file_name.strip_suffix(SET_EXTENSION) else {
                    continue;
                };
                if name == DEFAULT_SET {
                    return Err(SetError::DefaultFile {
                        default: CATEGORIES_FILE.to_string(),
                        file: format!("{SETS_DIR}/{file_name}"),
                    });
                }
                names.push(name.to_string());
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(SetError::Read {
                file: SETS_DIR.to_string(),
                message: error.to_string(),
            });
        }
    }
    names.sort();
    for name in names {
        sets.push(Set {
            file: format!("{SETS_DIR}/{name}{SET_EXTENSION}"),
            name,
        });
    }
    Ok(sets)
}

/// A set's categories: its file's entries, or the one catch-all entry when the set has no file
/// (04 › *Sets*). A file that does not load gives [`SetError::Rules`] naming the file.
pub fn load(folder: &Path, name: &str) -> Result<Vec<Category>, SetError> {
    let file = file_of(name)?;
    let path = folder.join(&file);
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(vec![catch_all_category()]);
        }
        Err(error) => {
            return Err(SetError::Read {
                file,
                message: error.to_string(),
            });
        }
    };
    load_rules_named(&text, &file).map_err(|error| SetError::Rules {
        message: error.to_string(),
    })
}

/// Write a new set's file: a copy of `copy_from`'s file when one is given, else the one catch-all
/// entry, so the new set reads like a set whose picks are not set yet (04 › *Sets*). The name
/// `default` is the data folder's own `categories.toml` and cannot be created, and a file already
/// there is refused, so nothing the owner wrote is overwritten.
pub fn create(folder: &Path, name: &str, copy_from: Option<&str>) -> Result<(), SetError> {
    let file = file_of(name)?;
    if file == CATEGORIES_FILE {
        return Err(SetError::ReservedDefault);
    }
    let path = folder.join(&file);
    if path.exists() {
        return Err(SetError::Exists { file });
    }
    let text = match copy_from {
        Some(from) => copied_text(folder, from)?,
        None => catch_all_text(),
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| SetError::Write {
            file: file.clone(),
            message: error.to_string(),
        })?;
    }
    std::fs::write(&path, text).map_err(|error| SetError::Write {
        file,
        message: error.to_string(),
    })
}

/// Delete a set's file (04 › *Sets*), so its name leaves the set list; a set with no file is
/// already gone. The set `default` is the data folder's own `categories.toml` and cannot be
/// deleted.
pub fn delete(folder: &Path, name: &str) -> Result<(), SetError> {
    let file = file_of(name)?;
    if file == CATEGORIES_FILE {
        return Err(SetError::ReservedDefault);
    }
    let path = folder.join(&file);
    match std::fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(SetError::Write {
            file,
            message: error.to_string(),
        }),
    }
}

/// The bytes a copy of `from` starts from: its file, or its one catch-all entry when that set has
/// no file of its own.
fn copied_text(folder: &Path, from: &str) -> Result<String, SetError> {
    let file = file_of(from)?;
    match std::fs::read_to_string(folder.join(&file)) {
        Ok(text) => Ok(text),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(catch_all_text()),
        Err(error) => Err(SetError::Read {
            file,
            message: error.to_string(),
        }),
    }
}

/// The one catch-all entry as a rules file's text (04 › *Sets*): code `Z`, name `Lainnya`, no
/// condition, so it takes every order. A new set — and the copy of a set with no file — starts
/// from it.
fn catch_all_text() -> String {
    let category = catch_all_category();
    format!(
        "[[category]]\ncode = \"{}\"\nname = \"{}\"\n",
        category.code, category.name
    )
}
