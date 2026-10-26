//! The day summary (day-summary.md): the open date of a workspace on A4, from its live batches and
//! reprints — the couriers' counts, each batch with its group PDFs and their Printed ticks, the
//! reprints, then from a new page the pick pages of the whole day. A view of the journal: nothing
//! reads it back, so its counts read singular where they should.

mod content;

use std::collections::BTreeSet;
use std::path::Path;

use jiff::civil::Date;
use packing_engine::record::{BatchSaved, ReprintMade};
use pdfium_render::prelude::Pdfium;

use crate::PdfError;
use crate::packing_list::emit::{emit, header_ops};
use crate::packing_list::fonts::{FONT_DIR, Font};
use crate::packing_list::measure::Measure;
use crate::packing_list::{BAR_H, Builder, CELL_MARGIN, Cell, Layout, MARGIN, Op};
use crate::summary_pdf::{LINE_H, LINE_PT, WIDTH, new_page};

use self::content::{DayLine, DaySection, LineKind};

/// How far a batch's or reprint's parts sit in from its head line, millimetres.
const INDENT: f32 = 5.0;

/// One live batch of the date.
pub struct DayBatch<'a> {
    pub record: &'a BatchSaved,
    /// `hh:mm` of its newest save.
    pub time: String,
    /// The group codes whose Printed tick is set.
    pub printed: &'a BTreeSet<String>,
}

/// One reprint of the date.
pub struct DayReprint<'a> {
    pub made: &'a ReprintMade,
    /// `hh:mm` when it was made.
    pub time: String,
}

pub struct DayFacts<'a> {
    pub workspace: &'a str,
    pub date: Date,
    /// Newest first, as on the Day screen.
    pub batches: Vec<DayBatch<'a>>,
    /// In the order they were made.
    pub reprints: Vec<DayReprint<'a>>,
    /// `hh:mm`.
    pub printed: &'a str,
}

/// Write the day summary and return its page count.
pub fn write_day_summary(
    pdfium: &Pdfium,
    path: &Path,
    facts: &DayFacts<'_>,
) -> Result<usize, PdfError> {
    let title = format!("Day summary · {} · {}", facts.workspace, facts.date);
    let subtitle = format!(
        "{} · printed {}",
        content::header_counts(facts),
        facts.printed
    );

    let font_dir = Path::new(FONT_DIR);
    let measure = Measure::new(pdfium, font_dir)?;
    let mut builder = Builder::new(Layout::Pick, &measure);
    for section in content::sections(facts) {
        draw_section(&mut builder, &section, &measure);
    }
    let picks = content::picks(facts);
    if !picks.is_empty() {
        new_page(&mut builder);
        builder.draw_summary_bar(picks.len());
        builder.draw_summary_rows(&picks);
    }

    let body = builder.pages;
    let count = body.len();
    let pages: Vec<Vec<Op>> = body
        .into_iter()
        .enumerate()
        .map(|(index, ops)| {
            let mut page = header_ops(&title, &subtitle, index + 1, count, &measure);
            page.extend(ops);
            page
        })
        .collect();
    emit(pdfium, path, &pages, font_dir, &measure)
}

/// A heading bar, then each line; a section that breaks to a new page says so there.
fn draw_section(builder: &mut Builder<'_, '_>, section: &DaySection, measure: &Measure<'_>) {
    builder.need(BAR_H + LINE_H, None);
    builder.y += 1.0;
    let top = builder.y;
    builder.fill(MARGIN, top, WIDTH, BAR_H);
    builder.cell(Cell {
        x: MARGIN,
        y: top,
        h: BAR_H,
        size_pt: 11.0,
        font: Font::ArialBold,
        text: &format!(" {}", section.heading),
    });
    builder.y = top + BAR_H;
    let continued = format!("{} (continued)", section.heading);
    for line in &section.lines {
        draw_line(builder, line, measure, &continued);
    }
    builder.y += 1.5;
}

/// One line: its text wrapped at spaces beside its count at the right edge; a head line is bold
/// with a little space above it, a part sits in under its head.
fn draw_line(
    builder: &mut Builder<'_, '_>,
    line: &DayLine,
    measure: &Measure<'_>,
    continued: &str,
) {
    let (x, font) = match line.kind {
        LineKind::Plain => (MARGIN, Font::Arial),
        LineKind::Head => (MARGIN, Font::ArialBold),
        LineKind::Part => (MARGIN + INDENT, Font::Arial),
    };
    let right_w = line.right.as_ref().map_or(0.0, |text| {
        measure.width_mm(text, Font::Arial, LINE_PT) + 2.0 * CELL_MARGIN + 3.0
    });
    let width = MARGIN + WIDTH - x - right_w - 2.0 * CELL_MARGIN;
    let texts = measure.wrap(&line.text, width, font, LINE_PT);
    let gap = if line.kind == LineKind::Head {
        1.0
    } else {
        0.0
    };
    builder.need(gap + LINE_H * texts.len() as f32, Some(continued));
    builder.y += gap;
    let top = builder.y;
    for (index, text) in texts.iter().enumerate() {
        builder.cell(Cell {
            x,
            y: top + LINE_H * index as f32,
            h: LINE_H,
            size_pt: LINE_PT,
            font,
            text,
        });
    }
    if let Some(right) = &line.right {
        builder.cell_right(
            Cell {
                x: MARGIN,
                y: top,
                h: LINE_H,
                size_pt: LINE_PT,
                font: Font::Arial,
                text: right,
            },
            WIDTH,
        );
    }
    builder.y = top + LINE_H * texts.len() as f32;
}

#[cfg(test)]
mod tests;
