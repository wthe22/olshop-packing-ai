//! The A4 page the summary PDFs are drawn on: the page geometry in millimetres, the drawing
//! operations a page is made of, and the two-column pick rows.
//!
//! The geometry, in millimetres: a cell's text baseline sits at `top + 0.5*h + 0.3*font_size`
//! and a left-aligned cell indents its text by the 1 mm cell margin. Wrapping width is the
//! column width minus twice that margin.

pub(crate) mod emit;
pub(crate) mod fonts;
pub(crate) mod measure;

use self::fonts::Font;
use self::measure::{Measure, PT_PER_MM};

// Layout constants, millimetres.
pub(crate) const PAGE_H: f32 = 297.0;
pub(crate) const MARGIN: f32 = 10.0;
pub(crate) const BOTTOM: f32 = PAGE_H - MARGIN; // 287
pub(crate) const CELL_MARGIN: f32 = 1.0;
/// The page header's height: title 7 + counts line 5 + the 1 mm gap.
const HEADER_H: f32 = 13.0;
/// Where the body starts on every page — below the header, never at the top margin, or the first
/// row of the body is printed over the title and the counts line.
pub(crate) const BODY_TOP: f32 = MARGIN + HEADER_H; // 23
const COL_X: [f32; 2] = [10.0, 106.0]; // pick-summary column left edges
const COL_W: f32 = 80.0; // pick-summary name wrap width
const ROW_H: f32 = 5.0; // pick-summary row height
pub(crate) const BAR_H: f32 = 6.5; // heading bar height

/// One drawing operation, in millimetres from the page's top-left.
#[derive(Clone, Debug)]
pub(crate) enum Op {
    Text {
        x: f32,
        /// Baseline distance from the page top.
        baseline: f32,
        size_pt: f32,
        font: Font,
        text: String,
    },
    Line {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
    },
    Fill {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    },
}

/// One piece of text a page draws: where it sits (millimetres from the page's top-left, `y` is
/// the cell's top), the cell's height and the font.
pub(crate) struct Cell<'t> {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) h: f32,
    pub(crate) size_pt: f32,
    pub(crate) font: Font,
    pub(crate) text: &'t str,
}

pub(crate) struct Builder<'m, 'a> {
    pub(crate) pages: Vec<Vec<Op>>,
    pub(crate) y: f32,
    measure: &'m Measure<'a>,
}

impl<'m, 'a> Builder<'m, 'a> {
    pub(crate) fn new(measure: &'m Measure<'a>) -> Builder<'m, 'a> {
        Builder {
            pages: vec![Vec::new()],
            y: BODY_TOP,
            measure,
        }
    }

    fn ops(&mut self) -> &mut Vec<Op> {
        self.pages.last_mut().expect("at least one page")
    }

    /// A baseline for a cell of height `h` at top `y_top`.
    fn baseline(y_top: f32, h: f32, size_pt: f32) -> f32 {
        y_top + 0.5 * h + 0.3 * (size_pt / PT_PER_MM)
    }

    /// Left-aligned cell text, indented by the cell margin.
    pub(crate) fn cell(&mut self, cell: Cell<'_>) {
        let baseline = Self::baseline(cell.y, cell.h, cell.size_pt);
        self.ops().push(Op::Text {
            x: cell.x + CELL_MARGIN,
            baseline,
            size_pt: cell.size_pt,
            font: cell.font,
            text: cell.text.to_string(),
        });
    }

    /// Right-aligned text inside a cell of width `w`: its advance, not its ink, ends at the edge,
    /// so the digits of numbers right-aligned on one edge stand under each other.
    pub(crate) fn cell_right(&mut self, cell: Cell<'_>, w: f32) {
        let width = self.measure.advance_mm(cell.text, cell.font, cell.size_pt);
        let baseline = Self::baseline(cell.y, cell.h, cell.size_pt);
        self.ops().push(Op::Text {
            x: cell.x + w - CELL_MARGIN - width,
            baseline,
            size_pt: cell.size_pt,
            font: cell.font,
            text: cell.text.to_string(),
        });
    }

    pub(crate) fn fill(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.ops().push(Op::Fill { x, y, w, h });
    }

    pub(crate) fn horizontal_line(&mut self, x1: f32, y: f32, x2: f32) {
        self.ops().push(Op::Line {
            x1,
            y1: y,
            x2,
            y2: y,
        });
    }

    /// The pick rows, breaking to a new page when the next chunk does not fit.
    pub(crate) fn draw_summary_rows(&mut self, entries: &[(i64, String)]) {
        let mut top = self.y;
        let mut index = 0;
        while index < entries.len() {
            let mut take = category_fit(&entries[index..], BOTTOM - top, self.measure);
            if take == 0 {
                self.pages.push(Vec::new());
                self.y = BODY_TOP;
                top = self.y;
                take = category_fit(&entries[index..], BOTTOM - top, self.measure).max(1);
            }
            self.draw_summary_chunk(&entries[index..index + take], top);
            index += take;
            if index < entries.len() {
                self.pages.push(Vec::new());
                self.y = BODY_TOP;
                top = self.y;
            } else {
                self.y = top;
            }
        }
    }

    /// One chunk of the pick rows, in two columns.
    fn draw_summary_chunk(&mut self, chunk: &[(i64, String)], top: f32) {
        let half = chunk.len().div_ceil(2);
        for (column, part) in [&chunk[..half], &chunk[half..]].into_iter().enumerate() {
            let mut y = top;
            let x = COL_X[column];
            for (units, name) in part {
                let lines = self
                    .measure
                    .wrap(name, COL_W - 2.0 * CELL_MARGIN, Font::Arial, 10.0);
                self.cell_right(
                    Cell {
                        x,
                        y,
                        h: ROW_H,
                        size_pt: 10.5,
                        font: Font::ArialBold,
                        text: &grouped(*units),
                    },
                    12.0,
                );
                for (i, text) in lines.iter().enumerate() {
                    self.cell(Cell {
                        x: x + 14.0,
                        y: y + ROW_H * i as f32,
                        h: ROW_H,
                        size_pt: 10.0,
                        font: Font::Arial,
                        text,
                    });
                }
                y += ROW_H * lines.len() as f32 + 0.4;
            }
        }
    }
}

/// Thousands separated by commas: `1,234`.
fn grouped(n: i64) -> String {
    let digits = n.unsigned_abs().to_string();
    let mut out = String::new();
    for (index, ch) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    if n < 0 { format!("-{out}") } else { out }
}

/// Largest count of the leading entries whose balanced two-column split fits `avail`.
fn category_fit(entries: &[(i64, String)], avail: f32, measure: &Measure<'_>) -> usize {
    let mut heights: Vec<f32> = Vec::with_capacity(entries.len());
    for (_units, name) in entries {
        let lines = measure.wrap(name, COL_W - 2.0 * CELL_MARGIN, Font::Arial, 10.0);
        heights.push(ROW_H * lines.len() as f32 + 0.4);
    }
    let mut prefix = vec![0.0f32];
    for height in &heights {
        prefix.push(prefix.last().copied().unwrap_or(0.0) + height);
    }
    for count in (1..=entries.len()).rev() {
        let left = count.div_ceil(2);
        let right_total = prefix[count] - prefix[left];
        if prefix[left].max(right_total) <= avail {
            return count;
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::fonts::{cid_required, segments};

    /// Which characters are drawn with the CID font, and — the part the text layer depends on —
    /// that no CID run ever holds a character the CID font encodes wrongly (a plain space becomes
    /// U+00A0, `-` U+00AD, `·` U+A78F, so a run with one of those in it would spoil a viewer's
    /// text layer).
    #[test]
    fn only_the_characters_pdfium_encodes_wrongly_use_the_cid_font() {
        // The header, the counts line and plain names: one run, no CID font.
        for text in [
            "Summary · 2026-10-11 · batch 1",
            "54 orders · 77 units · printed 13:15",
            "Sepatu Standar Samping Motor",
            "Pick summary (units to take from stock)",
        ] {
            let parts = segments(text);
            assert_eq!(parts.len(), 1, "{text:?} is one run: {parts:?}");
            assert!(!parts[0].1, "{text:?} needs no CID font");
        }

        // An item line: only the `×` needs the CID font; the `—` is a plain run of its own, so the
        // text after it starts at its true origin.
        assert_eq!(
            segments("Sepatu Standar Samping Motor — honda  ×1"),
            vec![
                ("Sepatu Standar Samping Motor ", false),
                ("—", false),
                (" honda  ", false),
                ("×", true),
                ("1", false),
            ]
        );
        assert_eq!(segments("——"), vec![("—", false), ("—", false)]);

        // Characters PDFium writes as `ÿ` each get their own run — the spaces between them stay
        // plain, and no run mixes the two.
        let parts = segments("Kabel Ñ ² ® ñ ½° ±");
        assert!(
            parts.len() > 3,
            "each such character is its own run: {parts:?}"
        );
        for (chunk, cid) in &parts {
            for ch in chunk.chars() {
                assert_eq!(
                    *cid,
                    cid_required(ch),
                    "{ch:?} in run {chunk:?} needs the CID font: {cid}"
                );
            }
        }
    }
}
