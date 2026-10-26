//! Measuring text with the same fonts the page uses, so a wrap is where the page breaks the line.

use std::path::Path;

use pdfium_render::prelude::*;

use crate::PdfError;

use super::fonts::{Font, FontSet, segments};

pub(super) const PT_PER_MM: f32 = 72.0 / 25.4;

pub(super) fn to_pt(mm: f32) -> f32 {
    mm * PT_PER_MM
}

/// Measures text widths using a throwaway document with the same fonts (PDFium text objects
/// give the exact Arial/Consolas advance widths).
pub(crate) struct Measure<'a> {
    doc: PdfDocument<'a>,
    fonts: FontSet,
}

impl<'a> Measure<'a> {
    pub(crate) fn new(pdfium: &'a Pdfium, dir: &Path) -> Result<Measure<'a>, PdfError> {
        let mut doc = pdfium.create_new_pdf()?;
        let mut fonts = FontSet::load(&mut doc, dir)?;
        // The measuring document is thrown away, never saved, so it can hold both flavours of all
        // three fonts.
        for font in [Font::Arial, Font::ArialBold, Font::Consola] {
            fonts.load_cid(&mut doc, dir, font)?;
        }
        doc.pages_mut().create_page_at_end(PdfPagePaperSize::a4())?;
        Ok(Measure { doc, fonts })
    }

    pub(super) fn token(&self, font: Font, cid: bool) -> PdfFontToken {
        self.fonts.token(font, cid)
    }

    /// The advance width of `text` at `size_pt`, in millimetres.
    pub(crate) fn width_mm(&self, text: &str, font: Font, size_pt: f32) -> f32 {
        if text.is_empty() {
            return 0.0;
        }
        segments(text)
            .iter()
            .map(|(chunk, cid)| self.chunk_width_mm(chunk, font, size_pt, *cid))
            .sum()
    }

    pub(super) fn chunk_width_mm(&self, text: &str, font: Font, size_pt: f32, cid: bool) -> f32 {
        let mut page = match self.doc.pages().get(0) {
            Ok(page) => page,
            Err(_) => return 0.0,
        };
        let objects = page.objects_mut();
        match objects.create_text_object(
            PdfPoints::new(0.0),
            PdfPoints::new(0.0),
            text,
            self.token(font, cid),
            PdfPoints::new(size_pt),
        ) {
            Ok(object) => object.width().map(|w| w.value / PT_PER_MM).unwrap_or(0.0),
            Err(_) => 0.0,
        }
    }

    /// `text` as the runs to draw, each with its offset from the start of `text` in millimetres:
    /// `(offset, chunk, needs the CID font)`. Plain text is one run at offset 0.
    ///
    /// The offsets come from PDFium's own character origins, measured with the **CID** font (whose
    /// glyphs are all correct). A text object's width is its ink bounds, which drops the trailing
    /// spaces a run in front of the next one ends with — origins do not.
    pub(super) fn segments_with_offsets<'b>(
        &self,
        text: &'b str,
        font: Font,
        size_pt: f32,
    ) -> Vec<(f32, &'b str, bool)> {
        let parts = segments(text);
        if parts.len() == 1 {
            let (chunk, cid) = parts[0];
            return vec![(0.0, chunk, cid)];
        }
        match self.origins(text, font, size_pt) {
            Some(origins) => {
                let base = origins.first().copied().unwrap_or(0.0);
                let mut out = Vec::with_capacity(parts.len());
                let mut index = 0usize;
                for (chunk, cid) in parts {
                    let start = origins.get(index).copied().unwrap_or(base);
                    out.push((start - base, chunk, cid));
                    index += chunk.chars().count();
                }
                out
            }
            // PDFium gave no origins: fall back to ink widths, which sits a fraction of a
            // millimetre off in front of a run that starts with spaces.
            None => {
                let mut x = 0.0;
                parts
                    .iter()
                    .map(|(chunk, cid)| {
                        let offset = x;
                        x += self.chunk_width_mm(chunk, font, size_pt, *cid);
                        (offset, *chunk, *cid)
                    })
                    .collect()
            }
        }
    }

    /// The x origin of every character of `text`, in millimetres from its start, drawn with the
    /// CID font; `None` when PDFium cannot report them.
    pub(super) fn origins(&self, text: &str, font: Font, size_pt: f32) -> Option<Vec<f32>> {
        let mut page = self.doc.pages().get(0).ok()?;
        let object = {
            let objects = page.objects_mut();
            objects
                .create_text_object(
                    PdfPoints::new(0.0),
                    PdfPoints::new(0.0),
                    text,
                    self.token(font, true),
                    PdfPoints::new(size_pt),
                )
                .ok()?
        };
        let textpage = page.text().ok()?;
        let chars = object.as_text_object()?.chars(&textpage).ok()?;
        if chars.len() != text.chars().count() {
            return None;
        }
        Some(
            chars
                .iter()
                .map(|character| match character.origin() {
                    Ok((x, _)) => x.value / PT_PER_MM,
                    Err(_) => 0.0,
                })
                .collect(),
        )
    }

    /// Greedy word wrap to `width_mm`.
    pub(crate) fn wrap(&self, text: &str, width_mm: f32, font: Font, size_pt: f32) -> Vec<String> {
        if text.is_empty() {
            return Vec::new();
        }
        let mut lines: Vec<String> = Vec::new();
        let mut current = String::new();
        for word in text.split(' ') {
            let candidate = if current.is_empty() {
                word.to_string()
            } else {
                format!("{current} {word}")
            };
            if current.is_empty() || self.width_mm(&candidate, font, size_pt) <= width_mm {
                current = candidate;
            } else {
                lines.push(std::mem::take(&mut current));
                current = word.to_string();
            }
        }
        lines.push(current);
        lines
    }
}
