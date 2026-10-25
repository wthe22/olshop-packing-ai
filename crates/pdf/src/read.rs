//! Open files, read a page's text with positions, the page count and progress.
//!
//! The text runs are rebuilt from PDFium characters in one of two ways; do not "simplify" them
//! into one. The downloaded labels (wkhtmltopdf) and the test fixtures hold one character per
//! text object and need the geometry rebuild; the Shopee-style label holds whole strings per
//! text object, one run each.

use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};

use pdfium_render::prelude::*;

use packing_engine::label::CourierTable;
use packing_engine::text::{LogoImage, PageText, Run};

use crate::PdfError;
use crate::logo;

/// How far down from the top of the page a logo may reach, in points.
pub const BAND_POINTS: f64 = 40.0;

/// A rounding tolerance for the band test: coordinates that meet the band to under half a point
/// still count as inside.
const BAND_TOLERANCE: f64 = 0.5;

/// An opened label PDF. The document borrows the `Pdfium` it was opened with; it never leaves
/// the worker thread that owns that library.
pub struct OpenedFile<'a> {
    pub name: String,
    pub path: PathBuf,
    pub pages: usize,
    doc: PdfDocument<'a>,
    pdfium: &'a Pdfium,
}

impl<'a> OpenedFile<'a> {
    /// The document, for `write.rs` page copying. PDFium objects stay on their thread, so this
    /// is crate-internal only.
    pub(crate) fn document(&self) -> &PdfDocument<'a> {
        &self.doc
    }

    /// The text (whole-page string and rebuilt runs) of one 0-based page, with the logos of the
    /// page's header band. `table` is the courier data the appearance decision uses.
    pub fn page_text(&self, index: usize, table: &CourierTable) -> Result<PageText, PdfError> {
        let page = page_at(&self.doc, &self.path, index)?;
        let text = page.text()?;
        let all = text.all();
        let runs = build_runs(&page, &text)?;
        let logos = self.logos_of(&page, index, table)?;
        Ok(PageText {
            text: all,
            runs,
            width: page.width().value as f64,
            height: page.height().value as f64,
            logos,
        })
    }

    /// The text and runs of one 0-based page with no header-band images: the check before saving
    /// reads its own output back and needs the Order ID and the printed counts only, never a
    /// logo.
    pub fn page_text_plain(&self, index: usize) -> Result<PageText, PdfError> {
        let page = page_at(&self.doc, &self.path, index)?;
        let text = page.text()?;
        let all = text.all();
        let runs = build_runs(&page, &text)?;
        Ok(PageText {
            text: all,
            runs,
            width: page.width().value as f64,
            height: page.height().value as f64,
            logos: Vec::new(),
        })
    }

    /// The header-band image with byte hash `hash` on one 0-based page, as PNG bytes (the logo
    /// prompt shows the logo as printed). `None` when the page carries no such image.
    pub fn band_picture(&self, index: usize, hash: &str) -> Result<Option<Vec<u8>>, PdfError> {
        let scratch = self.scratch()?;
        let page = page_at(&scratch, &self.path, index)?;
        page_picture(&page, &scratch, hash)
    }

    /// A second, throw-away copy of the file for rendering an image. `get_processed_image`
    /// rewrites the image object's matrix inside the page it is called on and the change survives
    /// a page reload (the QR and barcode images grew five times over), so it must never run on
    /// the document whose pages are later copied into the saved PDFs.
    fn scratch(&self) -> Result<PdfDocument<'a>, PdfError> {
        self.pdfium
            .load_pdf_from_file(&self.path, None)
            .map_err(|e| PdfError::Open {
                path: self.path.clone(),
                message: e.to_string(),
            })
    }

    /// The page's header-band logos. The byte hashes come from the real page (read-only); only a
    /// band image the courier data does not know needs its appearance, and that is rendered on
    /// the scratch copy.
    fn logos_of(
        &self,
        page: &PdfPage<'_>,
        index: usize,
        table: &CourierTable,
    ) -> Result<Vec<LogoImage>, PdfError> {
        let logos = page_logos(page, &self.doc, &|_| true)?;
        if logos.iter().all(|logo| table.knows_byte_hash(&logo.hash)) {
            return Ok(logos);
        }
        let scratch = self.scratch()?;
        let scratch_page = page_at(&scratch, &self.path, index)?;
        page_logos(&scratch_page, &scratch, &|hash| table.knows_byte_hash(hash))
    }
}

/// Whether an object's box lies wholly in the page's top band. `height` is
/// the page height in points; the coordinates count from the page bottom.
fn in_band(bounds: &PdfQuadPoints, height: f64) -> bool {
    let band = height - BAND_POINTS;
    let bottom = bounds.bottom().value as f64;
    let top = bounds.top().value as f64;
    bottom >= band - BAND_TOLERANCE && top <= height + BAND_TOLERANCE
}

/// The header-band image whose encoded bytes hash to `hash`, as PNG bytes, for the logo prompt.
/// `None` when the page carries no such image. The image is composited onto white, so a masked
/// logo shows its wordmark (courier.md item 3).
pub fn page_picture(
    page: &PdfPage<'_>,
    document: &PdfDocument<'_>,
    hash: &str,
) -> Result<Option<Vec<u8>>, PdfError> {
    let page_height = page.height().value as f64;
    for object in page.objects().iter() {
        let Some(image) = object.as_image_object() else {
            continue;
        };
        let Ok(bounds) = object.bounds() else {
            continue;
        };
        if !in_band(&bounds, page_height) {
            continue;
        }
        let bytes = image.get_raw_image_data()?;
        if bytes.is_empty() || logo::hash_bytes(&bytes) != hash {
            continue;
        }
        let Some(rendered) = image.get_processed_image(document).ok() else {
            return Ok(None);
        };
        let rgba = rendered.to_rgba8();
        let (width, height) = rgba.dimensions();
        return Ok(logo::picture_png(rgba.as_raw(), width, height));
    }
    Ok(None)
}

/// The images whose **whole box** lies in the page's top band (the top
/// [`BAND_POINTS`] points, `y` counted from the page bottom) — on every sample style exactly the
/// courier's logo and the buying channel's strip.
///
/// Each image is hashed by its **encoded bytes**. Its **appearance** is computed only where
/// `known` does not already know that byte hash (the courier data's `logo` and `not_courier`
/// lists), so a page whose logo the table knows pays no image decode.
pub fn page_logos(
    page: &PdfPage<'_>,
    document: &PdfDocument<'_>,
    known: &dyn Fn(&str) -> bool,
) -> Result<Vec<LogoImage>, PdfError> {
    let height = page.height().value as f64;
    let mut logos = Vec::new();
    for object in page.objects().iter() {
        let Some(image) = object.as_image_object() else {
            continue;
        };
        let Ok(bounds) = object.bounds() else {
            continue;
        };
        if !in_band(&bounds, height) {
            continue;
        }
        let bytes = image.get_raw_image_data()?;
        if bytes.is_empty() {
            continue;
        }
        let hash = logo::hash_bytes(&bytes);
        let appearance = if known(&hash) {
            None
        } else {
            appearance_of(image, document)
        };
        logos.push(LogoImage { hash, appearance });
    }
    Ok(logos)
}

/// The image's appearance hash, from PDFium's rendered bitmap (the `/SMask` is its alpha
/// channel). `None` when the image cannot be rendered.
fn appearance_of(image: &PdfPageImageObject<'_>, document: &PdfDocument<'_>) -> Option<String> {
    let rendered = image.get_processed_image(document).ok()?;
    let rgba = rendered.to_rgba8();
    let (width, height) = rgba.dimensions();
    if width == 0 || height == 0 {
        return None;
    }
    Some(logo::appearance_hash(
        rgba.as_raw(),
        width as usize,
        height as usize,
    ))
}

/// One 0-based page of a document; the error names the file and the page (1-based).
fn page_at<'a>(doc: &PdfDocument<'a>, path: &Path, index: usize) -> Result<PdfPage<'a>, PdfError> {
    doc.pages()
        .get(index as PdfPageIndex)
        .map_err(|e| PdfError::Open {
            path: path.to_path_buf(),
            message: format!("page {}: {e}", index + 1),
        })
}

pub(crate) fn file_name(path: &Path) -> String {
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
        pdfium,
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
/// when it is set the walk stops with [`PdfError::Cancelled`]. `table` is the data folder's
/// courier data, used for the band images' appearance decision.
pub fn for_each_page<F>(
    files: &[OpenedFile<'_>],
    cancel: &AtomicBool,
    progress: &mut dyn FnMut(usize, usize),
    table: &CourierTable,
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
            let page = file.page_text(page_index, table)?;
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
    table: &CourierTable,
    progress: &mut dyn FnMut(usize, usize),
) -> Result<Vec<packing_engine::orders::PageItem>, PdfError> {
    let files = open_all(pdfium, paths)?;
    let mut pages = Vec::new();
    for_each_page(&files, cancel, progress, table, &mut |file, page, text| {
        pages.push((file, page, text.clone()));
        Ok(())
    })?;
    Ok(pages)
}

/// Rebuild a page's runs; see the module comment for why there are two strategies.
fn build_runs(page: &PdfPage, text: &PdfPageText) -> Result<Vec<Run>, PdfError> {
    // Detect the granularity from the first few text objects only: the Shopee-style pages lead
    // with multi-char objects, the downloaded labels have no multi-char object at all. This keeps
    // the (quadratic) `chars` scan off the per-glyph pages.
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

/// Whole strings per text object (the Shopee-style label): one run per text object, characters
/// in PDFium index order, origin from the first glyph.
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
    let mut items = collect_glyphs(text)?;
    // Top to bottom, then left to right.
    items.sort_by(|a, b| {
        b.0.partial_cmp(&a.0)
            .unwrap_or(Ordering::Equal)
            .then(a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal))
    });
    Ok(glyph_runs(items))
}

/// One placed character: (baseline y, left, right, origin x, font, size, text).
type PlacedChar = (f64, f64, f64, f64, String, f32, String);

/// The page's characters with their boxes and origins, whitespace dropped.
fn collect_glyphs(text: &PdfPageText) -> Result<Vec<PlacedChar>, PdfError> {
    let chars = text.chars();
    let count = chars.len();
    let mut items: Vec<PlacedChar> = Vec::new();
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
    Ok(items)
}

/// Group the placed characters into runs: a run ends at a line change, a horizontal gap wider
/// than `SPLIT_POINTS`, or a font name/size change; a gap wider than `SPACE_POINTS` inside a run
/// becomes a space.
fn glyph_runs(items: Vec<PlacedChar>) -> Vec<Run> {
    const SPLIT_POINTS: f64 = 5.0;
    const SPACE_POINTS: f64 = 1.0;
    const LINE_TOLERANCE: f64 = 0.7;

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
    runs
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
