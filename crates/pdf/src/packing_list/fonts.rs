//! The packed fonts: which file each one is, the characters PDFium's non-CID font path writes
//! wrongly, and the plain/CID pair of each font that a page draws with.

use std::path::Path;

use pdfium_render::prelude::*;

use crate::PdfError;

/// The font files, from `C:\Windows\Fonts` (03 › *Packing list (A4)*).
pub(super) const FONT_DIR: &str = r"C:\Windows\Fonts";

/// Which of the three packed fonts a piece of text uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Font {
    Arial,
    ArialBold,
    Consola,
}

pub(super) fn font_file(font: Font) -> &'static str {
    match font {
        Font::Arial => "arial.ttf",
        Font::ArialBold => "arialbd.ttf",
        Font::Consola => "consola.ttf",
    }
}

/// The characters PDFium's non-CID font path writes with the wrong glyph **and** the wrong code:
/// with `cid = false` every one of `× ÷ é ® ™ ° ± ñ Ä ½ €` came out as `ÿ` (U+00FF), while
/// `· Ø § — – … ’ “ ”` came out right (probe, review round 1). A wrong `×` is what the owner sees
/// in the item lines of `full`/`summary`. Such characters are drawn with a CID copy of the same
/// font, which writes them — and reads them back — correctly.
pub(super) fn cid_required(ch: char) -> bool {
    !(ch.is_ascii() || matches!(ch, '·' | 'Ø' | '§' | '—' | '–' | '…' | '’' | '“' | '”'))
}

/// `text` split into runs that the non-CID font writes correctly and runs that need the CID font
/// ([`cid_required`]), in order. Most text is one run.
pub(super) fn segments(text: &str) -> Vec<(&str, bool)> {
    let mut segments: Vec<(&str, bool)> = Vec::new();
    let mut start = 0usize;
    let mut current: Option<bool> = None;
    for (index, ch) in text.char_indices() {
        let cid = cid_required(ch);
        match current {
            Some(flag) if flag == cid => {}
            Some(flag) => {
                segments.push((&text[start..index], flag));
                start = index;
                current = Some(cid);
            }
            None => current = Some(cid),
        }
    }
    if let Some(flag) = current {
        segments.push((&text[start..], flag));
    }
    segments
}

pub(super) fn font_index(font: Font) -> usize {
    match font {
        Font::Arial => 0,
        Font::ArialBold => 1,
        Font::Consola => 2,
    }
}

/// Loads one of the packed fonts into a document (PDFium embeds it whole).
pub(super) fn load_font(
    doc: &mut PdfDocument<'_>,
    dir: &Path,
    font: Font,
    cid: bool,
) -> Result<PdfFontToken, PdfError> {
    let path = dir.join(font_file(font));
    doc.fonts_mut()
        .load_true_type_from_file(&path, cid)
        .map_err(|e| PdfError::Font {
            path,
            message: e.to_string(),
        })
}

/// The three packed fonts in both flavours PDFium offers: `plain` (non-CID), which writes ASCII
/// and a handful of typographic characters correctly, and `cid`, which writes every character
/// correctly but encodes plain spaces as U+00A0 and `-` as U+00AD in the text layer — so it is
/// used for the characters [`cid_required`] names, and only those.
pub(super) struct FontSet {
    plain: [PdfFontToken; 3],
    cid: [Option<PdfFontToken>; 3],
}

impl FontSet {
    /// Load the three fonts as non-CID fonts (every page uses them).
    pub(super) fn load(doc: &mut PdfDocument<'_>, dir: &Path) -> Result<FontSet, PdfError> {
        Ok(FontSet {
            plain: [
                load_font(doc, dir, Font::Arial, false)?,
                load_font(doc, dir, Font::ArialBold, false)?,
                load_font(doc, dir, Font::Consola, false)?,
            ],
            cid: [None; 3],
        })
    }

    /// Load the CID flavour of `font` on first use, so a page that needs none does not carry the
    /// second copy of the font in the file (D2).
    pub(super) fn load_cid(
        &mut self,
        doc: &mut PdfDocument<'_>,
        dir: &Path,
        font: Font,
    ) -> Result<(), PdfError> {
        if self.cid[font_index(font)].is_none() {
            self.cid[font_index(font)] = Some(load_font(doc, dir, font, true)?);
        }
        Ok(())
    }

    pub(super) fn token(&self, font: Font, cid: bool) -> PdfFontToken {
        if cid {
            self.cid[font_index(font)].expect("the CID font is loaded before drawing")
        } else {
            self.plain[font_index(font)]
        }
    }
}
