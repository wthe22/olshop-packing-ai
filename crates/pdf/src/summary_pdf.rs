//! The summary PDF of a batch (summary-pdf.md), each part from a new page: the facts pages, which
//! say what the batch is so the paper alone can be traced; each group with its numbered order
//! contents; and the pick pages, what to take from stock. The verifier reads the header's counts,
//! the ledger and each group bar's counts back as text, so their words are fixed and their numbers
//! have no thousands separator.

pub(crate) mod content;

use std::path::Path;

use jiff::civil::Date;
use packing_engine::record::BatchSaved;
use packing_engine::workflows::Courier;
use pdfium_render::prelude::Pdfium;

use crate::PdfError;
use crate::facts::{
    GAP_BELOW_SECTION, KEY_W, ROW_H, ROW_PT, RULE_GAP, TEXT_RIGHT, TEXT_X, draw_bar, draw_rule,
    draw_section, keep_together, new_page,
};
use crate::packing_list::emit::{emit, header_ops};
use crate::packing_list::fonts::{FONT_DIR, Font};
use crate::packing_list::measure::Measure;
use crate::packing_list::{Builder, CELL_MARGIN, Cell, Layout, Op};

use self::content::{ContentRow, GroupSection};

/// The width of an order content's count of orders, at the right edge.
const COUNT_W: f32 = 20.0;

pub struct SummaryFacts<'a> {
    /// The workspace's name.
    pub workspace: &'a str,
    pub date: Date,
    /// The batch's record, its group PDFs already in `outputs`.
    pub record: &'a BatchSaved,
    /// `hh:mm`.
    pub printed: &'a str,
    /// The app version the journal's records carry.
    pub app: &'a str,
    /// The couriers of `packing-workflows.toml`, which name the record's courier codes.
    pub couriers: &'a [Courier],
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
        facts.workspace, facts.date, record.batch
    );
    let subtitle = format!(
        "{} orders · {} units · {} groups · printed {}",
        counts.orders, counts.units, counts.groups, facts.printed
    );

    let font_dir = Path::new(FONT_DIR);
    let measure = Measure::new(pdfium, font_dir)?;
    let mut builder = Builder::new(Layout::Pick, &measure);
    for section in content::facts(record, facts.app, facts.couriers) {
        draw_section(&mut builder, &section, &measure);
    }
    let groups = content::group_sections(record);
    if !groups.is_empty() {
        new_page(&mut builder);
        for group in &groups {
            draw_group(&mut builder, group, &measure);
        }
    }
    draw_picks(&mut builder, &content::picks([record]));

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

/// From a new page, the pick bar and the units per item in two columns.
pub(crate) fn draw_picks(builder: &mut Builder<'_, '_>, picks: &[(i64, String)]) {
    if picks.is_empty() {
        return;
    }
    new_page(builder);
    draw_bar(
        builder,
        "Pick summary (units to take from stock)",
        None,
        ROW_H,
    );
    builder.draw_summary_rows(picks);
}

/// A group's bar with its counts, which the verifier reads (always `orders` and `units`), then
/// each of its order contents under a rule.
fn draw_group(builder: &mut Builder<'_, '_>, group: &GroupSection, measure: &Measure<'_>) {
    let heading = format!("{}  {}", group.code, group.name);
    let counts = format!("{} orders · {} units", group.orders, group.units);
    draw_bar(builder, &heading, Some(&counts), ROW_H * 2.0);
    for content in &group.contents {
        draw_content(builder, content, measure, &heading);
    }
    builder.y += GAP_BELOW_SECTION;
}

/// One order content: its number, its items, each wrapped at spaces, and its number of orders at
/// the right; a content that does not fit starts on the next page under its group's bar.
fn draw_content(
    builder: &mut Builder<'_, '_>,
    content: &ContentRow,
    measure: &Measure<'_>,
    heading: &str,
) {
    let x = TEXT_X + KEY_W;
    let width = TEXT_RIGHT - COUNT_W - x;
    let lines: Vec<String> = content
        .items
        .iter()
        .flat_map(|item| measure.wrap(item, width, Font::Arial, ROW_PT))
        .collect();
    let height = ROW_H * lines.len() as f32;
    keep_together(builder, height + 2.0 * RULE_GAP, heading);
    let top = builder.y;
    builder.cell(Cell {
        x: TEXT_X,
        y: top,
        h: ROW_H,
        size_pt: ROW_PT,
        font: Font::ArialBold,
        text: &content.number,
    });
    for (index, text) in lines.iter().enumerate() {
        builder.cell(Cell {
            x,
            y: top + ROW_H * index as f32,
            h: ROW_H,
            size_pt: ROW_PT,
            font: Font::Arial,
            text,
        });
    }
    builder.cell_right(
        Cell {
            x: TEXT_RIGHT + CELL_MARGIN - COUNT_W,
            y: top,
            h: ROW_H,
            size_pt: 11.5,
            font: Font::ArialBold,
            text: &content.orders.to_string(),
        },
        COUNT_W,
    );
    draw_rule(builder, top + height);
}

#[cfg(test)]
mod tests;
