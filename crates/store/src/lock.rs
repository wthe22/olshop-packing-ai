//! One copy of the app per data folder: `.lock` held with `File::try_lock` while the app runs,
//! and `.lock-holder` beside it saying which copy holds it (Windows refuses to read a locked file).

use std::fs::{self, File, OpenOptions, TryLockError};
use std::path::Path;

use packing_engine::record::Datetime;

use crate::StoreError;

pub const LOCK: &str = ".lock";
pub const LOCK_HOLDER: &str = ".lock-holder";

/// The data folder's lock; released when dropped.
pub struct DataLock {
    _file: File,
}

impl DataLock {
    /// Take the data folder, or say which copy of the app has it.
    pub fn acquire(data_dir: &Path) -> Result<Self, StoreError> {
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
            Err(TryLockError::WouldBlock) => {
                let holder = fs::read_to_string(data_dir.join(LOCK_HOLDER))
                    .map(|text| text.trim().to_string())
                    .unwrap_or_else(|_| "another copy of the app".to_string());
                return Err(StoreError::LockHeld { holder });
            }
            Err(TryLockError::Error(error)) => return Err(StoreError::io(&path, error)),
        }
        let holder_path = data_dir.join(LOCK_HOLDER);
        fs::write(&holder_path, holder_text()).map_err(|e| StoreError::io(&holder_path, e))?;
        Ok(Self { _file: file })
    }
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
