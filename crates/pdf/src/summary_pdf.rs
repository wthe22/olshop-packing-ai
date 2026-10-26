//! The summary PDF of a batch (summary-pdf.md), each part from a new page: the facts pages, which
//! say what the batch is so the paper alone can be traced; each group with its order contents; and
//! the pick pages, what to take from stock. The verifier reads the header's counts, the ledger and
//! each group's line back as text, so their words are fixed and their numbers have no thousands
//! separator.

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

use self::content::{ContentRow, GroupSection, Section};

/// A facts line's height, millimetres.
const LINE_H: f32 = 5.0;
const LINE_PT: f32 = 9.5;
const WIDTH: f32 = 190.0;
/// An order content's item line, millimetres.
const ITEM_H: f32 = 5.2;
/// Where an order content's count of orders ends, millimetres from the left edge.
const COUNT_RIGHT: f32 = MARGIN + WIDTH;
const COUNT_W: f32 = 20.0;

pub struct SummaryFacts<'a> {
    /// The namespace's name.
    pub namespace: &'a str,
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
        facts.namespace, facts.date, record.batch
    );
    let subtitle = format!(
        "{} orders · {} units · {} groups · printed {}",
        counts.orders, counts.units, counts.groups, facts.printed
    );

    let font_dir = Path::new(FONT_DIR);
    let measure = Measure::new(pdfium, font_dir)?;
    let mut builder = Builder::new(Layout::Pick, &measure);
    for section in content::facts(record, facts.app) {
        draw_section(&mut builder, &section, &measure);
    }
    let groups = content::group_sections(record);
    if !groups.is_empty() {
        new_page(&mut builder);
        for group in &groups {
            draw_group(&mut builder, group, &measure);
        }
    }
    let picks = content::picks(record);
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

fn new_page(builder: &mut Builder<'_, '_>) {
    builder.pages.push(Vec::new());
    builder.y = BODY_TOP;
}

/// A group's bar with its counts, then each of its order contents under a rule.
fn draw_group(builder: &mut Builder<'_, '_>, group: &GroupSection, measure: &Measure<'_>) {
    builder.need(BAR_H + ITEM_H * 2.0, None);
    let top = builder.y;
    builder.fill(MARGIN, top, WIDTH, BAR_H);
    builder.cell(Cell {
        x: MARGIN,
        y: top,
        h: BAR_H,
        size_pt: 11.0,
        font: Font::ArialBold,
        text: &format!(" {}", group.heading),
    });
    builder.cell_right(
        Cell {
            x: MARGIN,
            y: top,
            h: BAR_H,
            size_pt: 9.5,
            font: Font::Arial,
            text: &format!("{} orders · {} units", group.orders, group.units),
        },
        WIDTH,
    );
    builder.y = top + BAR_H;
    for content in &group.contents {
        draw_content(builder, content, measure);
    }
    builder.y += 1.5;
}

/// One order content: its items, each wrapped at spaces, and its number of orders at the right; a
/// content that does not fit starts on the next page.
fn draw_content(builder: &mut Builder<'_, '_>, content: &ContentRow, measure: &Measure<'_>) {
    let width = COUNT_RIGHT - COUNT_W - (MARGIN + 2.0) - 2.0 * CELL_MARGIN;
    let lines: Vec<String> = content
        .items
        .iter()
        .flat_map(|item| measure.wrap(item, width, Font::Arial, 10.5))
        .collect();
    builder.need(ITEM_H * lines.len() as f32 + 1.2, None);
    let top = builder.y;
    for (index, text) in lines.iter().enumerate() {
        builder.cell(Cell {
            x: MARGIN + 2.0,
            y: top + ITEM_H * index as f32,
            h: ITEM_H,
            size_pt: 10.5,
            font: Font::Arial,
            text,
        });
    }
    builder.cell_right(
        Cell {
            x: COUNT_RIGHT - COUNT_W,
            y: top,
            h: ITEM_H,
            size_pt: 11.5,
            font: Font::ArialBold,
            text: &content.orders.to_string(),
        },
        COUNT_W,
    );
    let line_y = top + ITEM_H * lines.len() as f32 + 0.6;
    builder.horizontal_line(MARGIN, line_y, COUNT_RIGHT);
    builder.y = line_y + 0.6;
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
