//! What an Update does to a date folder (versions.md): `update v{app}/update.toml` lists each move before
//! it is made, and the update folder keeps a copy of each file whose content the Update changes, so the
//! owner's `tools/revert-update.py` can undo it. A moved file is not copied.

use std::fs;
use std::path::{Path, PathBuf};

use packing_engine::record::Sha256;
use serde::{Deserialize, Serialize};

use crate::StoreError;
use crate::files::{copy_checked, move_file, sha256_of, write_atomic};
use crate::journal::JOURNAL;
use crate::views::{DAY_TOML, ORDER_CONTENTS_TOML, ORDERS_CSV, SESSION_TOML};

pub const UPDATE_LOG: &str = "update.toml";
/// Where a file in the way of a move is set aside, inside the update folder.
const REPLACED: &str = "replaced";
/// How the revert script renames an update folder it has undone.
const REVERTED: &str = " reverted";
const HEAD: &str = "# What Packing's Update did to this date folder, in order. tools/revert-update.py undoes it.\n";

fn unreadable(path: &Path, error: impl std::fmt::Display) -> StoreError {
    StoreError::io(path, std::io::Error::other(error.to_string()))
}

/// One thing the Update did, in order. Paths are relative to the date folder, `/` between folders.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum UpdateStep {
    Moved {
        from: String,
        to: String,
    },
    /// A copy of `from` written as `to`.
    Copied {
        from: String,
        to: String,
        sha256: Sha256,
    },
    /// A file written where there was none.
    Written {
        file: String,
    },
    /// A file whose content the Update changes; its bytes before are in the update folder by its name.
    Kept {
        file: String,
        sha256: Sha256,
    },
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct LogFile {
    app: String,
    /// The date of v0.7 is flat.
    #[serde(default)]
    flattened: bool,
    /// The journal as the Update last left it: the revert script leaves a date whose journal differs.
    journal_after: Option<Sha256>,
    #[serde(default)]
    step: Vec<UpdateStep>,
}

/// The update folder of one date, written again after every step.
pub struct UpdateLog {
    date_dir: PathBuf,
    folder: PathBuf,
    file: LogFile,
}

impl UpdateLog {
    /// The update folder of `app` in `date_dir`; one a stopped or earlier Update of the same version
    /// started is continued.
    pub fn open(date_dir: &Path, app: &str) -> Result<Self, StoreError> {
        let folder = date_dir.join(format!("update v{app}"));
        let path = folder.join(UPDATE_LOG);
        let file = match fs::read_to_string(&path) {
            Ok(text) => toml::from_str(&text).map_err(|error| unreadable(&path, error))?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => LogFile {
                app: app.to_string(),
                ..LogFile::default()
            },
            Err(error) => return Err(StoreError::io(&path, error)),
        };
        Ok(Self {
            date_dir: date_dir.to_path_buf(),
            folder,
            file,
        })
    }

    pub fn steps(&self) -> &[UpdateStep] {
        &self.file.step
    }

    /// Before the Update changes the date: copy the journal and the views into the update folder, and
    /// note the journal as it is.
    pub fn keep_journal_and_views(&mut self) -> Result<(), StoreError> {
        for name in [
            JOURNAL,
            SESSION_TOML,
            DAY_TOML,
            ORDERS_CSV,
            ORDER_CONTENTS_TOML,
        ] {
            self.keep(name)?;
        }
        self.journal_is_now()
    }

    /// Copy `name` of the date folder into the update folder before its content changes, or list it as
    /// written when it is not there. A file listed already keeps its first copy.
    fn keep(&mut self, name: &str) -> Result<(), StoreError> {
        let listed = self.file.step.iter().any(|step| match step {
            UpdateStep::Kept { file, .. } | UpdateStep::Written { file } => file == name,
            _ => false,
        });
        if listed {
            return Ok(());
        }
        let path = self.date_dir.join(name);
        if !path.is_file() {
            return self.record(UpdateStep::Written { file: name.into() });
        }
        let sha256 = sha256_of(&path)?;
        self.create_folder(&self.folder)?;
        copy_checked(&path, &self.folder.join(name), &sha256)?;
        self.record(UpdateStep::Kept {
            file: name.into(),
            sha256,
        })
    }

    /// Move `from` to `to`, listed first.
    pub fn moving(&mut self, from: &Path, to: &Path) -> Result<(), StoreError> {
        self.record(UpdateStep::Moved {
            from: self.relative(from),
            to: self.relative(to),
        })?;
        move_file(from, to)
    }

    /// Copy `from`, of bytes `sha256`, to `to`, listed first.
    pub fn copying(&mut self, from: &Path, to: &Path, sha256: &Sha256) -> Result<(), StoreError> {
        self.record(UpdateStep::Copied {
            from: self.relative(from),
            to: self.relative(to),
            sha256: sha256.clone(),
        })?;
        copy_checked(from, to, sha256)
    }

    /// Move a file in the way of a move into `replaced/` of the update folder.
    pub fn set_aside(&mut self, path: &Path) -> Result<(), StoreError> {
        let aside = self.folder.join(REPLACED);
        self.create_folder(&aside)?;
        self.moving(path, &aside.join(path.file_name().unwrap_or_default()))
    }

    pub fn flattened(&mut self) -> Result<(), StoreError> {
        self.file.flattened = true;
        self.save()
    }

    /// Note the journal as the Update leaves it now.
    pub fn journal_is_now(&mut self) -> Result<(), StoreError> {
        self.file.journal_after = Some(sha256_of(&self.date_dir.join(JOURNAL))?);
        self.save()
    }

    fn record(&mut self, step: UpdateStep) -> Result<(), StoreError> {
        self.file.step.push(step);
        self.save()
    }

    fn save(&self) -> Result<(), StoreError> {
        self.create_folder(&self.folder)?;
        let path = self.folder.join(UPDATE_LOG);
        let text = toml::to_string(&self.file).map_err(|error| unreadable(&path, error))?;
        write_atomic(&path, format!("{HEAD}{text}").as_bytes())
    }

    fn create_folder(&self, folder: &Path) -> Result<(), StoreError> {
        fs::create_dir_all(folder).map_err(|error| StoreError::io(folder, error))
    }

    fn relative(&self, path: &Path) -> String {
        let inside = path.strip_prefix(&self.date_dir).unwrap_or(path);
        inside
            .components()
            .map(|part| part.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/")
    }
}

/// Whether an Update not reverted has made the date of v0.7 in `date_dir` flat.
pub(crate) fn listed_flat(date_dir: &Path) -> bool {
    #[derive(Deserialize)]
    struct Flat {
        #[serde(default)]
        flattened: bool,
    }
    let Ok(entries) = fs::read_dir(date_dir) else {
        return false;
    };
    entries.filter_map(Result::ok).any(|entry| {
        let name = entry.file_name().to_string_lossy().into_owned();
        name.starts_with("update v")
            && !name.ends_with(REVERTED)
            && fs::read_to_string(entry.path().join(UPDATE_LOG))
                .ok()
                .and_then(|text| toml::from_str::<Flat>(&text).ok())
                .is_some_and(|found| found.flattened)
    })
}
