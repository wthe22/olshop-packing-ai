//! The data folder's write lock. Copies of the app, on this PC or another, share one data folder;
//! `.lock` is held with `File::try_lock` only while one of them writes (a commit and the file moves
//! around it, a summary made on request, a write of the accounts), and `.lock-holder` beside it says which copy
//! holds it (Windows refuses to read a locked file).

use std::fs::{self, File, OpenOptions, TryLockError};
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

use packing_engine::record::Datetime;

use crate::StoreError;

pub const LOCK: &str = ".lock";
pub const LOCK_HOLDER: &str = ".lock-holder";

/// How long a write waits for another copy's write to end: longer than the save of a big batch.
const WAIT: Duration = Duration::from_secs(60);
const RETRY: Duration = Duration::from_millis(100);

/// The data folder's lock; released when dropped.
pub struct DataLock {
    _file: File,
}

impl DataLock {
    /// Take the lock, waiting while another copy writes; after a minute the error names it.
    pub fn acquire(data_dir: &Path) -> Result<Self, StoreError> {
        let start = Instant::now();
        loop {
            if let Some(lock) = Self::try_acquire(data_dir)? {
                return Ok(lock);
            }
            if start.elapsed() >= WAIT {
                return Err(StoreError::LockHeld {
                    holder: holder(data_dir),
                });
            }
            thread::sleep(RETRY);
        }
    }

    /// Take the lock if no other copy is writing; `None` while one is.
    pub fn try_acquire(data_dir: &Path) -> Result<Option<Self>, StoreError> {
        fs::create_dir_all(data_dir).map_err(|e| StoreError::io(data_dir, e))?;
        let path = data_dir.join(LOCK);
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(&path)
            .map_err(|e| StoreError::io(&path, e))?;
        match file.try_lock() {
            Ok(()) => {}
            Err(TryLockError::WouldBlock) => return Ok(None),
            Err(TryLockError::Error(error)) => return Err(StoreError::io(&path, error)),
        }
        let holder_path = data_dir.join(LOCK_HOLDER);
        fs::write(&holder_path, holder_text()).map_err(|e| StoreError::io(&holder_path, e))?;
        Ok(Some(Self { _file: file }))
    }
}

/// What `.lock-holder` says, or a plain name while it is being written.
fn holder(data_dir: &Path) -> String {
    fs::read_to_string(data_dir.join(LOCK_HOLDER))
        .map(|text| text.trim().to_string())
        .ok()
        .filter(|text| !text.is_empty())
        .unwrap_or_else(|| "another copy of the app".to_string())
}

/// `<computer>, process <pid>, since <time>`.
fn holder_text() -> String {
    let computer = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "this PC".to_string());
    format!(
        "{computer}, process {}, since {}\n",
        std::process::id(),
        Datetime::now()
    )
}
