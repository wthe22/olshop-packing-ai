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
    /// Another copy of the app kept writing to the data folder longer than a write waits; the
    /// text says which copy.
    LockHeld { holder: String },
    /// Another copy of the app wrote to the journal after it was folded: nothing was appended.
    JournalChanged { path: PathBuf },
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
            StoreError::LockHeld { holder } => write!(
                f,
                "another copy of the app is still writing to the data folder ({holder}); try again in a moment"
            ),
            StoreError::JournalChanged { path } => write!(
                f,
                "another copy of the app wrote to {} meanwhile; nothing was saved, try again",
                path.display()
            ),
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
