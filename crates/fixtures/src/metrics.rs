//! The two fonts of the fixtures and their character widths: the standard Helvetica and
//! Helvetica-Bold metrics, in thousandths of the font size, for the characters 32 to 126. The
//! same table goes into each font's `Widths`, so PDFium measures the characters where the writer
//! put them.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Font {
    Regular,
    Bold,
}

pub const FIRST_CHAR: u8 = 32;
pub const LAST_CHAR: u8 = 126;

#[rustfmt::skip]
const REGULAR: [u16; 95] = [
    278, 278, 355, 556, 556, 889, 667, 191, 333, 333, 389, 584, 278, 333, 278, 278,
    556, 556, 556, 556, 556, 556, 556, 556, 556, 556, 278, 278, 584, 584, 584, 556,
    1015, 667, 667, 722, 722, 667, 611, 778, 722, 278, 500, 667, 556, 833, 722, 778,
    667, 778, 722, 667, 611, 722, 667, 944, 667, 667, 611, 278, 278, 278, 469, 556,
    333, 556, 556, 500, 556, 556, 278, 556, 556, 222, 222, 500, 222, 833, 556, 556,
    556, 556, 333, 500, 278, 556, 500, 722, 500, 500, 500, 334, 260, 334, 584,
];

#[rustfmt::skip]
const BOLD: [u16; 95] = [
    278, 333, 474, 556, 556, 889, 722, 238, 333, 333, 389, 584, 278, 333, 278, 278,
    556, 556, 556, 556, 556, 556, 556, 556, 556, 556, 333, 333, 584, 584, 584, 611,
    975, 722, 722, 722, 722, 667, 611, 778, 722, 278, 556, 722, 611, 833, 722, 778,
    667, 778, 722, 667, 611, 722, 667, 944, 667, 667, 611, 333, 278, 333, 584, 556,
    333, 556, 611, 556, 611, 556, 333, 611, 611, 278, 278, 556, 278, 889, 611, 611,
    611, 611, 389, 556, 333, 611, 556, 778, 556, 556, 500, 389, 280, 389, 584,
];

impl Font {
    pub fn base_name(self) -> &'static str {
        match self {
            Font::Regular => "Helvetica",
            Font::Bold => "Helvetica-Bold",
        }
    }

    pub fn resource_name(self) -> &'static str {
        match self {
            Font::Regular => "F1",
            Font::Bold => "F2",
        }
    }

    pub fn widths(self) -> &'static [u16; 95] {
        match self {
            Font::Regular => &REGULAR,
            Font::Bold => &BOLD,
        }
    }

    /// The advance of one character at `size` points. The fixtures print ASCII only.
    pub fn advance(self, ch: char, size: f64) -> f64 {
        let code = ch as u32;
        assert!(
            (FIRST_CHAR as u32..=LAST_CHAR as u32).contains(&code),
            "the fixtures print ASCII only, not {ch:?}"
        );
        self.widths()[(code - FIRST_CHAR as u32) as usize] as f64 * size / 1000.0
    }
}
