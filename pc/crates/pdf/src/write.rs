//! Copy pages into a saved PDF.

use std::path::{Path, PathBuf};

use pdfium_render::prelude::*;

use crate::PdfError;
use crate::read::OpenedFile;

/// Copy the given `(file index, page index)` pages from already-opened source documents, in
/// order, into a new PDF saved at `out`. The pages are copied unchanged
/// (`copy_page_range_from_document`; the spike measured them pixel-identical at 150 DPI).
pub fn copy_pages(
    pdfium: &Pdfium,
    sources: &[OpenedFile<'_>],
    pages: &[(usize, usize)],
    out: &Path,
) -> Result<(), PdfError> {
    if pages.is_empty() {
        return Err(PdfError::Message("no pages to copy".to_string()));
    }
    let mut destination = pdfium.create_new_pdf()?;
    for (destination_index, &(file, page)) in pages.iter().enumerate() {
        let source = sources.get(file).ok_or_else(|| {
            PdfError::Message(format!(
                "page refers to file {file}, but only {} files are open",
                sources.len()
            ))
        })?;
        if page >= source.pages {
            return Err(PdfError::Message(format!(
                "{}: page {} does not exist ({} pages)",
                source.name,
                page + 1,
                source.pages
            )));
        }
        destination.pages_mut().copy_page_range_from_document(
            source.document(),
            (page as PdfPageIndex)..=(page as PdfPageIndex),
            destination_index as PdfPageIndex,
        )?;
    }
    destination.save_to_file(out).map_err(|e| PdfError::Io {
        path: PathBuf::from(out),
        message: e.to_string(),
    })?;
    Ok(())
}
