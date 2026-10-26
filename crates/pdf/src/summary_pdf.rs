//! The summary PDF of a batch (summary-pdf.md): the pick pages, what to take from stock, then the
//! facts page, which says what the batch is so the paper alone can be traced. The verifier reads
//! the header's counts, the ledger and each group's line back as text, so their words are fixed
//! and their numbers have no thousands separator.

mod content;

use std::path::Path;

use jiff::civil::Date;
use packing_engine::record::BatchSaved;
use pdfium_render::prelude::Pdfium;

use crate::PdfError;
use crate::packing_list::emit::{emit, header_ops};
use crate::packing_list::fonts::{FONT_DIR, Font};
use crate::packing_list::measure::Measure;
use crate::packing_list::{BAR_H, BODY_TOP, Builder, CELL_MARGIN, Cell, Layout, MARGIN, Op};

use self::content::Section;

/// A facts line's height, millimetres.
const LINE_H: f32 = 5.0;
const LINE_PT: f32 = 9.5;
const WIDTH: f32 = 190.0;

pub struct SummaryFacts<'a> {
    /// The namespace's name.
    pub owner: &'a str,
    pub date: Date,
    /// The batch's record, its group PDFs already in `outputs`.
    pub record: &'a BatchSaved,
    /// `hh:mm`.
    pub printed: &'a str,
    /// The app version the journal's records carry.
    pub app: &'a str,
}

/// Write the summary PDF and return its page count.
pub fn write_summary(
    pdfium: &Pdfium,
    path: &Path,
    facts: &SummaryFacts<'_>,
) -> Result<usize, PdfError> {
    let record = facts.record;
    let counts = content::counts(record);
    let title = format!(
        "Summary · {} · {} · batch {}",
        facts.owner, facts.date, record.batch
    );
    let subtitle = format!(
        "{} orders · {} units · {} groups · printed {}",
        counts.orders, counts.units, counts.groups, facts.printed
    );

    let font_dir = Path::new(FONT_DIR);
    let measure = Measure::new(pdfium, font_dir)?;
    let mut builder = Builder::new(Layout::Pick, &measure);
    let picks = content::picks(record);
    if !picks.is_empty() {
        builder.draw_summary_bar(picks.len());
        builder.draw_summary_rows(&picks);
        builder.pages.push(Vec::new());
        builder.y = BODY_TOP;
    }
    for section in content::facts(record, facts.app) {
        draw_section(&mut builder, &section, &measure);
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

/// A heading bar, then each line wrapped at spaces; a section that breaks to a new page repeats its
/// heading there.
fn draw_section(builder: &mut Builder<'_, '_>, section: &Section, measure: &Measure<'_>) {
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
        for text in measure.wrap(line, WIDTH - 2.0 * CELL_MARGIN, Font::Arial, LINE_PT) {
            builder.need(LINE_H, Some(&continued));
            builder.cell(Cell {
                x: MARGIN,
                y: builder.y,
                h: LINE_H,
                size_pt: LINE_PT,
                font: Font::Arial,
                text: &text,
            });
            builder.y += LINE_H;
        }
    }
    builder.y += 1.5;
}

#[cfg(test)]
mod tests;
