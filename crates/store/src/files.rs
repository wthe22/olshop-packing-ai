//! File helpers the store shares: a file's SHA-256, a write that a crash cannot leave half done,
//! and a move that names a file Windows refuses to move.

use std::fs;
use std::io::{Read, Write};
use std::path::Path;

use packing_engine::record::Sha256;
use sha2::{Digest, Sha256 as Hasher};

use crate::StoreError;

/// The SHA-256 of a file's bytes.
pub fn sha256_of(path: &Path) -> Result<Sha256, StoreError> {
    let mut file = fs::File::open(path).map_err(|e| StoreError::io(path, e))?;
    let mut hasher = Hasher::new();
    let mut buffer = vec![0u8; 1 << 16];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|e| StoreError::io(path, e))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    let hex = format!("{:x}", hasher.finalize());
    Ok(Sha256::try_from(hex).expect("a SHA-256 digest is 64 hex characters"))
}

/// Write `bytes` to a temporary file beside `path`, sync it and rename it into place.
pub(crate) fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), StoreError> {
    let temp = path.with_extension("tmp");
    let mut file = fs::File::create(&temp).map_err(|e| StoreError::io(&temp, e))?;
    file.write_all(bytes)
        .map_err(|e| StoreError::io(&temp, e))?;
    file.sync_all().map_err(|e| StoreError::io(&temp, e))?;
    drop(file);
    fs::rename(&temp, path).map_err(|e| StoreError::io(path, e))
}

/// Move `from` to `to`, replacing a file there. A move Windows refuses because the file is open
/// is `FileInUse`, naming the file the owner must close.
pub(crate) fn move_file(from: &Path, to: &Path) -> Result<(), StoreError> {
    fs::rename(from, to).map_err(|error| {
        if error.kind() == std::io::ErrorKind::PermissionDenied || error.raw_os_error() == Some(32)
        {
            StoreError::FileInUse {
                path: from.to_path_buf(),
            }
        } else {
            StoreError::io(from, error)
        }
    })
}

/// The files directly inside `dir`, sorted by name; none when `dir` does not exist.
pub(crate) fn files_in(dir: &Path) -> Result<Vec<String>, StoreError> {
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
            .is_file()
        {
            names.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    names.sort();
    Ok(names)
}
