use std::fmt;
use std::path::{Path, PathBuf};

use crate::fold::{Damage, DamageReason};

#[derive(Debug)]
pub enum StoreError {
    Io {
        path: PathBuf,
        error: std::io::Error,
    },
    /// The date's journal is damaged: it can be viewed, nothing is written to it.
    Damaged(Damage),
    /// A record the fold would not accept; nothing was written.
    Rejected(DamageReason),
    /// Another copy of the app holds the data folder; the text says which.
    LockHeld { holder: String },
    /// The copy of an input does not hash to what was read: the input is not stored.
    InputChanged { path: PathBuf },
    /// Windows refused to move a file, most often because a viewer has it open.
    FileInUse { path: PathBuf },
}

impl StoreError {
    pub(crate) fn io(path: &Path, error: std::io::Error) -> Self {
        StoreError::Io {
            path: path.to_path_buf(),
            error,
        }
    }
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreError::Io { path, error } => write!(f, "{}: {error}", path.display()),
            StoreError::Damaged(damage) => write!(f, "the journal is damaged: {damage}"),
            StoreError::Rejected(reason) => write!(f, "the record was not saved: {reason}"),
            StoreError::LockHeld { holder } => write!(f, "the data folder is in use by {holder}"),
            StoreError::InputChanged { path } => {
                write!(f, "{} changed while it was copied", path.display())
            }
            StoreError::FileInUse { path } => {
                write!(f, "{} is open in another program", path.display())
            }
        }
    }
}

impl std::error::Error for StoreError {}
