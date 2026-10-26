//! The layout the summary PDF and the day summary share: a grey bar with a heading, then rows of
//! text with their counts in three columns at the right, so the numbers of a page line up. A
//! section that breaks to a new page repeats its bar there.

use crate::packing_list::fonts::Font;
use crate::packing_list::measure::Measure;
use crate::packing_list::{BAR_H, BODY_TOP, BOTTOM, Builder, CELL_MARGIN, Cell, MARGIN};

/// The bars' width, millimetres: the page's text area.
pub(crate) const WIDTH: f32 = 190.0;
/// Where text starts, in line with a bar's heading: a cell's own margin puts it 2 mm in.
pub(crate) const TEXT_X: f32 = MARGIN + 1.0;
/// The right end of the text area, as far in as [`TEXT_X`].
pub(crate) const TEXT_RIGHT: f32 = MARGIN + WIDTH - 2.0;
pub(crate) const ROW_H: f32 = 5.5;
const ROW_PT: f32 = 10.0;
/// How far a part sits in from its head row.
const INDENT: f32 = 5.0;
/// Where each count column's text ends.
const COLUMN_RIGHT: [f32; 3] = [143.0, 166.0, TEXT_RIGHT];
/// Where a row's text ends when the row has counts.
const TEXT_END: f32 = 122.0;
const GAP_ABOVE_BAR: f32 = 1.0;
const GAP_BELOW_BAR: f32 = 1.0;
pub(crate) const GAP_BELOW_SECTION: f32 = 3.0;
const GAP_ABOVE_HEAD: f32 = 1.5;

/// How a row is drawn: a section's own row, the bold head of a block, or one of its parts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RowKind {
    Plain,
    Head,
    Part,
}

/// One row: its text, wrapped at spaces, and up to three counts, one per column.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Row {
    pub kind: RowKind,
    pub text: String,
    pub counts: [Option<String>; 3],
}

impl Row {
    pub(crate) fn new(kind: RowKind, text: impl Into<String>) -> Row {
        Row {
            kind,
            text: text.into(),
            counts: [None, None, None],
        }
    }

    pub(crate) fn plain(text: impl Into<String>) -> Row {
        Row::new(RowKind::Plain, text)
    }

    /// The row with `count` in column `column` (0 to 2).
    pub(crate) fn count(mut self, column: usize, count: impl Into<String>) -> Row {
        self.counts[column] = Some(count.into());
        self
    }
}

/// A heading bar and its rows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Section {
    pub heading: &'static str,
    pub rows: Vec<Row>,
}

/// `1 order`, `3 orders`.
pub(crate) fn count(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

pub(crate) fn new_page(builder: &mut Builder<'_, '_>) {
    builder.pages.push(Vec::new());
    builder.y = BODY_TOP;
}

/// A bar with `heading` and, at its right, `right`; `keep` millimetres of what follows stay on the
/// same page as the bar.
pub(crate) fn draw_bar(
    builder: &mut Builder<'_, '_>,
    heading: &str,
    right: Option<&str>,
    keep: f32,
) {
    if builder.y + GAP_ABOVE_BAR + BAR_H + GAP_BELOW_BAR + keep > BOTTOM {
        new_page(builder);
    }
    let top = builder.y + GAP_ABOVE_BAR;
    builder.fill(MARGIN, top, WIDTH, BAR_H);
    builder.cell(Cell {
        x: TEXT_X,
        y: top,
        h: BAR_H,
        size_pt: 11.0,
        font: Font::ArialBold,
        text: heading,
    });
    if let Some(right) = right {
        builder.cell_right(
            Cell {
                x: MARGIN,
                y: top,
                h: BAR_H,
                size_pt: 9.5,
                font: Font::Arial,
                text: right,
            },
            TEXT_RIGHT + CELL_MARGIN - MARGIN,
        );
    }
    builder.y = top + BAR_H + GAP_BELOW_BAR;
}

/// When `height` does not fit, a new page with the bar again, marked *(continued)*.
pub(crate) fn keep_together(builder: &mut Builder<'_, '_>, height: f32, heading: &str) {
    if builder.y + height > BOTTOM {
        new_page(builder);
        draw_bar(builder, &format!("{heading} (continued)"), None, height);
    }
}

pub(crate) fn draw_section(
    builder: &mut Builder<'_, '_>,
    section: &Section,
    measure: &Measure<'_>,
) {
    draw_bar(builder, section.heading, None, ROW_H);
    for (index, row) in section.rows.iter().enumerate() {
        draw_row(builder, row, index == 0, measure, section.heading);
    }
    builder.y += GAP_BELOW_SECTION;
}

fn draw_row(
    builder: &mut Builder<'_, '_>,
    row: &Row,
    first: bool,
    measure: &Measure<'_>,
    heading: &str,
) {
    let (x, font) = match row.kind {
        RowKind::Plain => (TEXT_X, Font::Arial),
        RowKind::Head => (TEXT_X, Font::ArialBold),
        RowKind::Part => (TEXT_X + INDENT, Font::Arial),
    };
    let end = if row.counts.iter().any(Option::is_some) {
        TEXT_END
    } else {
        TEXT_RIGHT
    };
    let lines = measure.wrap(&row.text, end - x - 1.0, font, ROW_PT);
    let gap = if row.kind == RowKind::Head && !first {
        GAP_ABOVE_HEAD
    } else {
        0.0
    };
    keep_together(builder, gap + ROW_H * lines.len() as f32, heading);
    builder.y += gap;
    let top = builder.y;
    for (index, text) in lines.iter().enumerate() {
        builder.cell(Cell {
            x,
            y: top + ROW_H * index as f32,
            h: ROW_H,
            size_pt: ROW_PT,
            font,
            text,
        });
    }
    let count_font = if row.kind == RowKind::Head {
        Font::ArialBold
    } else {
        Font::Arial
    };
    for (right, count) in COLUMN_RIGHT.iter().zip(&row.counts) {
        if let Some(count) = count {
            builder.cell_right(
                Cell {
                    x: MARGIN,
                    y: top,
                    h: ROW_H,
                    size_pt: ROW_PT,
                    font: count_font,
                    text: count,
                },
                right + CELL_MARGIN - MARGIN,
            );
        }
    }
    builder.y = top + ROW_H * lines.len() as f32;
}
