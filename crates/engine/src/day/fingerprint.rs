//! A label file's identity: its size and XXH3-64, streamed so a big file is not held in memory.

use std::fs::File;
use std::io::Read;
use std::path::Path;

use xxhash_rust::xxh3::Xxh3;

use super::error::DayError;

/// A label file's identity: size and XXH3-64 of its bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fingerprint {
    pub size: u64,
    pub xxh3: String,
}

/// The size and XXH3-64 of a file, streamed in chunks so a big file is not held in memory.
pub fn fingerprint(path: &Path) -> Result<Fingerprint, DayError> {
    let mut file = File::open(path).map_err(|error| DayError::Read {
        path: path.to_path_buf(),
        message: error.to_string(),
    })?;
    let mut hasher = Xxh3::new();
    let mut size = 0u64;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer).map_err(|error| DayError::Read {
            path: path.to_path_buf(),
            message: error.to_string(),
        })?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        size += read as u64;
    }
    Ok(Fingerprint {
        size,
        xxh3: format!("{:016x}", hasher.digest()),
    })
}
