//! Copy pages into a saved PDF.

use std::path::{Path, PathBuf};

use pdfium_render::prelude::*;

use crate::PdfError;
use crate::read::OpenedFile;

/// Copy the given `(file index, page index)` pages from already-opened source documents, in
/// order, into a new PDF saved at `out`. The pages are copied unchanged (the spike measured
/// them pixel-identical at 150 DPI).
///
/// PDFium shares a page's resources (fonts) only within one import call: copying page by page
/// put a font copy on every page (564 label pages became 51 MB instead of 5 MB). So the source
/// files are first joined into one in-memory document and all pages are imported from it in
/// a single call, in the wanted order.
pub fn copy_pages(
    pdfium: &Pdfium,
    sources: &[OpenedFile<'_>],
    pages: &[(usize, usize)],
    out: &Path,
) -> Result<(), PdfError> {
    if pages.is_empty() {
        return Err(PdfError::Message("no pages to copy".to_string()));
    }
    let mut joined = pdfium.create_new_pdf()?;
    let mut offsets = Vec::with_capacity(sources.len());
    for source in sources {
        offsets.push(joined.pages().len() as usize);
        joined.pages_mut().append(source.document())?;
    }
    let mut range = Vec::with_capacity(pages.len());
    for &(file, page) in pages {
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
        // The range string counts pages from 1; PDFium keeps the order written.
        range.push((offsets[file] + page + 1).to_string());
    }
    let mut destination = pdfium.create_new_pdf()?;
    destination
        .pages_mut()
        .copy_pages_from_document(&joined, &range.join(","), 0)?;
    destination.save_to_file(out).map_err(|e| PdfError::Io {
        path: PathBuf::from(out),
        message: e.to_string(),
    })?;
    Ok(())
}
