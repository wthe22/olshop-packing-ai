//! What the save, a reprint and a restore share: pages copied unchanged from stored inputs into a
//! file of `output/.partial/`, and a written file described as its record names it.

use std::fs;
use std::path::{Path, PathBuf};

use packing_engine::record::OutputFile;
use packing_pdf::worker::PdfWorker;
use packing_pdf::{read, write};
use packing_store::{StoreError, sha256_of};

use crate::BatchError;

/// Copy `pages` — `(source, page index from 0)` — in their order from `sources` into `target`.
pub(crate) fn copy_pages(
    worker: &PdfWorker,
    sources: Vec<PathBuf>,
    pages: Vec<(usize, usize)>,
    target: PathBuf,
) -> Result<(), BatchError> {
    worker.run(move |pdfium| {
        let opened = read::open_all(pdfium, &sources)?;
        write::copy_pages(pdfium, &opened, &pages, &target)
    })??;
    Ok(())
}

/// `output/.partial/` of a date, made if missing.
pub(crate) fn partial(date_dir: &Path) -> Result<PathBuf, BatchError> {
    let partial = packing_store::partial_dir(date_dir);
    fs::create_dir_all(&partial).map_err(|error| StoreError::Io {
        path: partial.clone(),
        error,
    })?;
    Ok(partial)
}

pub(crate) fn described(
    path: &Path,
    group: Option<&str>,
    pages: usize,
) -> Result<OutputFile, BatchError> {
    let size = fs::metadata(path)
        .map_err(|error| StoreError::Io {
            path: path.to_path_buf(),
            error,
        })?
        .len();
    Ok(OutputFile {
        file: path
            .file_name()
            .map_or_else(String::new, |name| name.to_string_lossy().into_owned()),
        group: group.map(str::to_string),
        pages: u16::try_from(pages).unwrap_or(u16::MAX),
        size,
        sha256: sha256_of(path)?,
    })
}
