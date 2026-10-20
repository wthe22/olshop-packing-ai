//! The day's failures, and whether a refused write means the file is open in another program.

use std::fmt;
use std::path::{Path, PathBuf};

/// A `state.json` could not be read or written, or a batch folder deleted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DayError {
    /// `state.json` cannot be read: `<message>`.
    Read { path: PathBuf, message: String },
    /// A folder cannot be written ("Cannot write to `<folder>`: `<reason>`").
    Write { folder: PathBuf, message: String },
    /// A file is open in another program (Windows refuses the delete/rename).
    FileInUse { path: PathBuf },
}

impl fmt::Display for DayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DayError::Read { path, message } => {
                write!(f, "cannot read {}: {message}", path.display())
            }
            DayError::Write { folder, message } => {
                write!(f, "Cannot write to {}: {message}", folder.display())
            }
            DayError::FileInUse { path } => write!(
                f,
                "{} is open in another program. Close it (e.g. the PDF viewer) and press Save again.",
                path.display()
            ),
        }
    }
}

impl std::error::Error for DayError {}

/// Windows `ERROR_SHARING_VIOLATION` / `ERROR_ACCESS_DENIED`: a file is open elsewhere.
fn in_use(error: &std::io::Error) -> bool {
    matches!(error.raw_os_error(), Some(32) | Some(5))
}

/// The error for a failed write at `path`: a sharing violation is the file-in-use wording.
pub(super) fn io_error(path: &Path, error: std::io::Error) -> DayError {
    if in_use(&error) {
        DayError::FileInUse {
            path: path.to_path_buf(),
        }
    } else {
        DayError::Write {
            folder: path.to_path_buf(),
            message: error.to_string(),
        }
    }
}
