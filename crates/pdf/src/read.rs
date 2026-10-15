//! Open files, read a page's text with positions, the page count and progress.
//!
//! The text runs are rebuilt from PDFium characters exactly as the spike did (07 › *Text runs
//! from PDFium*, validated against pypdf on 600/600 sample and 23/23 testdata orders). Do not
//! "simplify" the two strategies: the downloaded labels (wkhtmltopdf) hold one character per
//! text object and need the geometry rebuild, while fpdf2 files (the testdata) hold one whole
//! run per text object and must not be sorted by position.

use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};

use pdfium_render::prelude::*;

use packing_engine::text::{PageText, Run};

use crate::PdfError;

/// An opened label PDF. The document borrows the `Pdfium` it was opened with; it never leaves
/// the worker thread that owns that library.
pub struct OpenedFile<'a> {
    pub name: String,
    pub path: PathBuf,
    pub pages: usize,
    doc: PdfDocument<'a>,
}

impl<'a> OpenedFile<'a> {
    /// The document, for `write.rs` page copying. PDFium objects stay on their thread, so this
    /// is crate-internal only.
    pub(crate) fn document(&self) -> &PdfDocument<'a> {
        &self.doc
    }

    /// The text (whole-page string and rebuilt runs) of one 0-based page.
    pub fn page_text(&self, index: usize) -> Result<PageText, PdfError> {
        let page = self
            .doc
            .pages()
            .get(index as PdfPageIndex)
            .map_err(|e| PdfError::Open {
                path: self.path.clone(),
                message: format!("page {}: {e}", index + 1),
            })?;
        let text = page.text()?;
        let all = text.all();
        let runs = build_runs(&page, &text)?;
        Ok(PageText {
            text: all,
            runs,
            width: page.width().value as f64,
        })
    }
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned())
}

/// Open one label file; reads only its page count.
pub fn open<'a>(pdfium: &'a Pdfium, path: &Path) -> Result<OpenedFile<'a>, PdfError> {
    let doc = pdfium
        .load_pdf_from_file(path, None)
        .map_err(|e| PdfError::Open {
            path: path.to_path_buf(),
            message: e.to_string(),
        })?;
    let pages = doc.pages().len() as usize;
    Ok(OpenedFile {
        name: file_name(path),
        path: path.to_path_buf(),
        pages,
        doc,
    })
}

/// Open several files in the given order.
pub fn open_all<'a>(
    pdfium: &'a Pdfium,
    paths: &[PathBuf],
) -> Result<Vec<OpenedFile<'a>>, PdfError> {
    paths.iter().map(|path| open(pdfium, path)).collect()
}

/// Walk the pages of already-opened files in order, calling `on_page(file, page, text)` and
/// `progress(pages_done, pages_total)` after each page. Between pages the cancel flag is read;
/// when it is set the walk stops with [`PdfError::Cancelled`].
pub fn for_each_page<F>(
    files: &[OpenedFile<'_>],
    cancel: &AtomicBool,
    progress: &mut dyn FnMut(usize, usize),
    on_page: &mut F,
) -> Result<(), PdfError>
where
    F: FnMut(usize, usize, &PageText) -> Result<(), PdfError>,
{
    let total: usize = files.iter().map(|file| file.pages).sum();
    let mut done = 0usize;
    for (file_index, file) in files.iter().enumerate() {
        for page_index in 0..file.pages {
            if cancel.load(AtomicOrdering::Relaxed) {
                return Err(PdfError::Cancelled);
            }
            let page = file.page_text(page_index)?;
            on_page(file_index, page_index, &page)?;
            done += 1;
            progress(done, total);
        }
    }
    Ok(())
}

/// Open the files and read every page (the plain reader the tests and CLI use).
pub fn read_all(
    pdfium: &Pdfium,
    paths: &[PathBuf],
    cancel: &AtomicBool,
    progress: &mut dyn FnMut(usize, usize),
) -> Result<Vec<packing_engine::orders::PageItem>, PdfError> {
    let files = open_all(pdfium, paths)?;
    let mut pages = Vec::new();
    for_each_page(&files, cancel, progress, &mut |file, page, text| {
        pages.push((file, page, text.clone()));
        Ok(())
    })?;
    Ok(pages)
}

/// Rebuild a page's runs; see the module comment for why there are two strategies.
fn build_runs(page: &PdfPage, text: &PdfPageText) -> Result<Vec<Run>, PdfError> {
    // Detect the granularity from the first few text objects only: the fpdf2 pages lead with a
    // multi-char object, the wkhtmltopdf pages have no multi-char object at all. This keeps the
    // (quadratic) `chars` scan off the per-glyph pages.
    let mut per_glyph = true;
    let mut seen = 0usize;
    for object in page.objects().iter() {
        if let Some(text_object) = object.as_text_object() {
            if text_object
                .chars(text)
                .map(|chars| chars.len() > 1)
                .unwrap_or(false)
            {
                per_glyph = false;
                break;
            }
            seen += 1;
            if seen >= 16 {
                break;
            }
        }
    }
    if per_glyph {
        build_runs_geometry(text)
    } else {
        build_runs_objects(page, text)
    }
}

/// Operator-level objects (fpdf2 testdata): one run per text object, characters in PDFium index
/// order, origin from the first glyph — pypdf's run, exactly.
fn build_runs_objects(page: &PdfPage, text: &PdfPageText) -> Result<Vec<Run>, PdfError> {
    let mut runs: Vec<Run> = Vec::new();
    for object in page.objects().iter() {
        let Some(text_object) = object.as_text_object() else {
            continue;
        };
        let chars = text_object.chars(text)?;
        let mut string = String::new();
        let mut origin: Option<(f64, f64)> = None;
        for character in chars.iter() {
            let raw = character.unicode_string().unwrap_or_default();
            for ch in raw.chars() {
                // PDFium reports a hyphen at a line end as U+0002; the PDF text is a real '-'.
                string.push(if ch == '\u{0002}' { '-' } else { ch });
            }
            if origin.is_none()
                && !raw.trim().is_empty()
                && let Ok((x, y)) = character.origin()
            {
                origin = Some((x.value as f64, y.value as f64));
            }
        }
        let trimmed = string.trim();
        if trimmed.is_empty() {
            continue;
        }
        let (x, y) = origin.unwrap_or((0.0, 0.0));
        runs.push(Run {
            x,
            y,
            text: trimmed.to_string(),
        });
    }
    Ok(runs)
}

/// Per-glyph objects (wkhtmltopdf samples): PDFium's character order is not the visual order, so
/// characters are placed by their box first, then grouped: a run ends at a line change, a
/// horizontal gap wider than `SPLIT_POINTS`, or a font name/size change; a gap wider than
/// `SPACE_POINTS` inside a run becomes a space. A run's (x, y) is its first character's origin.
fn build_runs_geometry(text: &PdfPageText) -> Result<Vec<Run>, PdfError> {
    const SPLIT_POINTS: f64 = 5.0;
    const SPACE_POINTS: f64 = 1.0;
    const LINE_TOLERANCE: f64 = 0.7;

    let chars = text.chars();
    let count = chars.len();
    // (baseline y, left, right, origin x, font, size, text); whitespace dropped.
    let mut items: Vec<(f64, f64, f64, f64, String, f32, String)> = Vec::new();
    let mut last_y = 0.0f64;
    for i in 0..count {
        let character = chars.get(i)?;
        let raw = character.unicode_string().unwrap_or_default();
        let string: String = raw
            .chars()
            .map(|ch| if ch == '\u{0002}' { '-' } else { ch })
            .collect();
        if string.trim().is_empty() {
            continue;
        }
        let (left, right) = match character.loose_bounds() {
            Ok(rect) => (rect.left().value as f64, rect.right().value as f64),
            Err(_) => match character.tight_bounds() {
                Ok(rect) => (rect.left().value as f64, rect.right().value as f64),
                Err(_) => continue,
            },
        };
        let (origin_x, origin_y) = match character.origin() {
            Ok((x, y)) => (x.value as f64, y.value as f64),
            Err(_) => (left, last_y),
        };
        if origin_y.is_finite() {
            last_y = origin_y;
        }
        items.push((
            origin_y,
            left,
            right,
            origin_x,
            character.font_name(),
            character.unscaled_font_size().value,
            string,
        ));
    }
    items.sort_by(|a, b| {
        b.0.partial_cmp(&a.0)
            .unwrap_or(Ordering::Equal)
            .then(a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal))
    });

    let mut runs: Vec<Run> = Vec::new();
    let mut buffer = String::new();
    let mut run_x = 0.0f64;
    let mut run_y = 0.0f64;
    let mut run_font = String::new();
    let mut run_size = 0.0f32;
    let mut last_right = 0.0f64;
    let mut have = false;
    for (origin_y, left, right, origin_x, font, size, string) in items {
        let gap = left - last_right;
        let new_run = !have
            || (origin_y - run_y).abs() > LINE_TOLERANCE
            || gap > SPLIT_POINTS
            || font != run_font
            || (size - run_size).abs() > 0.01;
        if new_run {
            if have {
                push_run(&mut runs, &mut buffer, run_x, run_y);
                buffer.clear();
            }
            run_x = origin_x;
            run_y = origin_y;
            run_font = font;
            run_size = size;
        } else if gap > SPACE_POINTS {
            buffer.push(' ');
        }
        buffer.push_str(&string);
        last_right = right;
        have = true;
    }
    if have {
        push_run(&mut runs, &mut buffer, run_x, run_y);
    }
    Ok(runs)
}

fn push_run(runs: &mut Vec<Run>, buffer: &mut str, x: f64, y: f64) {
    let trimmed = buffer.trim();
    if !trimmed.is_empty() {
        runs.push(Run {
            x,
            y,
            text: trimmed.to_string(),
        });
    }
}
