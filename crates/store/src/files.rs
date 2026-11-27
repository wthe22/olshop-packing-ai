//! File helpers the store shares: a file's SHA-256, a write that a crash cannot leave half done,
//! a move that names a file Windows refuses to move, and a folder's size.

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
    Ok(digest(hasher))
}

/// The SHA-256 of bytes in memory.
pub(crate) fn sha256_of_bytes(bytes: &[u8]) -> Sha256 {
    digest(Hasher::new_with_prefix(bytes))
}

fn digest(hasher: Hasher) -> Sha256 {
    let hex = format!("{:x}", hasher.finalize());
    Sha256::try_from(hex).expect("a SHA-256 digest is 64 hex characters")
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

/// Copy `source` to `target` through `{name}.partial`, synced and hashed again, then renamed.
pub(crate) fn copy_checked(
    source: &Path,
    target: &Path,
    sha256: &Sha256,
) -> Result<(), StoreError> {
    let name = target.file_name().unwrap_or_default().to_string_lossy();
    let partial = target.with_file_name(format!("{name}.partial"));
    let mut from = fs::File::open(source).map_err(|e| StoreError::io(source, e))?;
    let mut copy = fs::File::create(&partial).map_err(|e| StoreError::io(&partial, e))?;
    std::io::copy(&mut from, &mut copy)
        .and_then(|_| copy.sync_all())
        .map_err(|e| StoreError::io(&partial, e))?;
    drop(copy);
    if sha256_of(&partial)? != *sha256 {
        let _ = fs::remove_file(&partial);
        return Err(StoreError::InputChanged {
            path: source.to_path_buf(),
        });
    }
    move_file(&partial, target)
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

/// The size of every file in `dir` and its subfolders together; 0 when `dir` does not exist.
pub fn folder_size(dir: &Path) -> Result<u64, StoreError> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(error) => return Err(StoreError::io(dir, error)),
    };
    let mut total = 0;
    for entry in entries {
        let entry = entry.map_err(|e| StoreError::io(dir, e))?;
        let meta = entry.metadata().map_err(|e| StoreError::io(dir, e))?;
        total += if meta.is_dir() {
            folder_size(&entry.path())?
        } else {
            meta.len()
        };
    }
    Ok(total)
}
