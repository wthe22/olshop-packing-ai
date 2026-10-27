//! The layout every part of the summary PDF and the day summary shares: a grey bar with a heading
//! and, at its right, the section's own counts; then ruled rows — a bold key at the left (`A-1`,
//! `2-S1`), the text, and at the right the counts, each number in its column with its word after
//! it, so the numbers of a page stand under each other. A section that breaks to a new page repeats
//! its bar there.

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
pub(crate) const ROW_PT: f32 = 10.5;
/// The key column's width: a keyed row's text starts right of it.
pub(crate) const KEY_W: f32 = 15.0;
/// The rule under a row sits this far below its last line, and the next row as far below the rule.
pub(crate) const RULE_GAP: f32 = 0.6;
pub(crate) const GAP_BELOW_SECTION: f32 = 3.0;
/// Where the numbers of each count column end; a count's word follows its number.
const NUMBER_RIGHT: [f32; 2] = [152.0, 178.0];
const WORD_GAP: f32 = 1.2;
/// The room a count's number may take left of its column's end.
const NUMBER_W: f32 = 12.0;
/// The space kept between a row's text and its tail.
const TAIL_GAP: f32 = 4.0;
const GAP_ABOVE_BAR: f32 = 1.0;
const GAP_BELOW_BAR: f32 = 1.0;

/// A number and the word it counts: `1 order`, `3 orders`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Count {
    pub n: usize,
    pub one: &'static str,
    pub many: &'static str,
}

impl Count {
    fn word(&self) -> &'static str {
        if self.n == 1 { self.one } else { self.many }
    }
}

/// One ruled row: its key, its text (wrapped at spaces), a count per column and a tail at the
/// right edge.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Row {
    pub key: Option<String>,
    pub text: String,
    pub bold: bool,
    pub counts: [Option<Count>; 2],
    /// Text at the right edge, past the count columns (an input's SHA-256).
    pub tail: Option<String>,
}

impl Row {
    pub(crate) fn new(text: impl Into<String>) -> Row {
        Row {
            key: None,
            text: text.into(),
            bold: false,
            counts: [None, None],
            tail: None,
        }
    }

    pub(crate) fn key(mut self, key: impl Into<String>) -> Row {
        self.key = Some(key.into());
        self
    }

    pub(crate) fn bold(mut self) -> Row {
        self.bold = true;
        self
    }

    /// The row with `n` counted as `one` or `many` in column `column` (0 or 1).
    pub(crate) fn count(
        mut self,
        column: usize,
        n: usize,
        one: &'static str,
        many: &'static str,
    ) -> Row {
        self.counts[column] = Some(Count { n, one, many });
        self
    }

    pub(crate) fn tail(mut self, tail: impl Into<String>) -> Row {
        self.tail = Some(tail.into());
        self
    }
}

/// A heading bar, its counts at the right, and its rows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Section {
    pub heading: String,
    pub right: Option<String>,
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

/// The rule under a row whose lines end at `bottom`; the next row starts below it.
pub(crate) fn draw_rule(builder: &mut Builder<'_, '_>, bottom: f32) {
    let y = bottom + RULE_GAP;
    builder.horizontal_line(MARGIN, y, MARGIN + WIDTH);
    builder.y = y + RULE_GAP;
}

/// `text` ending at `right`.
fn text_right_at(builder: &mut Builder<'_, '_>, right: f32, top: f32, font: Font, text: &str) {
    builder.cell_right(
        Cell {
            x: right + CELL_MARGIN - NUMBER_W,
            y: top,
            h: ROW_H,
            size_pt: ROW_PT,
            font,
            text,
        },
        NUMBER_W,
    );
}

pub(crate) fn draw_section(
    builder: &mut Builder<'_, '_>,
    section: &Section,
    measure: &Measure<'_>,
) {
    draw_bar(
        builder,
        &section.heading,
        section.right.as_deref(),
        ROW_H + 2.0 * RULE_GAP,
    );
    for row in &section.rows {
        draw_row(builder, row, measure, &section.heading);
    }
    builder.y += GAP_BELOW_SECTION;
}

fn draw_row(builder: &mut Builder<'_, '_>, row: &Row, measure: &Measure<'_>, heading: &str) {
    let x = if row.key.is_some() {
        TEXT_X + KEY_W
    } else {
        TEXT_X
    };
    let font = if row.bold {
        Font::ArialBold
    } else {
        Font::Arial
    };
    let end = text_end(row, measure);
    let lines = measure.wrap(&row.text, end - x - 2.0 * CELL_MARGIN, font, ROW_PT);
    let height = ROW_H * lines.len().max(1) as f32;
    keep_together(builder, height + 2.0 * RULE_GAP, heading);
    let top = builder.y;
    if let Some(key) = &row.key {
        builder.cell(Cell {
            x: TEXT_X,
            y: top,
            h: ROW_H,
            size_pt: ROW_PT,
            font: Font::ArialBold,
            text: key,
        });
    }
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
    for (right, count) in NUMBER_RIGHT.iter().zip(&row.counts) {
        if let Some(count) = count {
            text_right_at(builder, *right, top, font, &count.n.to_string());
            builder.cell(Cell {
                x: right + WORD_GAP - CELL_MARGIN,
                y: top,
                h: ROW_H,
                size_pt: ROW_PT,
                font,
                text: count.word(),
            });
        }
    }
    if let Some(tail) = &row.tail {
        text_right_at(builder, TEXT_RIGHT, top, font, tail);
    }
    draw_rule(builder, top + height);
}

/// Where a row's text must end: before its first count column, else before its tail.
fn text_end(row: &Row, measure: &Measure<'_>) -> f32 {
    match row.counts.iter().position(Option::is_some) {
        Some(column) => NUMBER_RIGHT[column] - NUMBER_W,
        None => match &row.tail {
            Some(tail) => TEXT_RIGHT - measure.width_mm(tail, Font::Arial, ROW_PT) - TAIL_GAP,
            None => TEXT_RIGHT,
        },
    }
}
