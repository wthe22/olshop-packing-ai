//! The day summary (day-summary.md): the open date of a workspace on A4, from its live batches —
//! each batch as its Day card shows it, its groups with their orders and order contents; the
//! couriers' counts; then from a new page the pick pages of the whole day, and from another the
//! day's order contents with their orders. Drawn in the summary PDF's layout. A view of the
//! journal: nothing reads it back, so its counts read singular where they should.

mod content;

use std::path::Path;

use jiff::civil::Date;
use packing_engine::record::BatchSaved;
use packing_engine::workflows::Courier;
use pdfium_render::prelude::Pdfium;

use crate::PdfError;
use crate::facts::{ROW_H, draw_bar, draw_section, new_page};
use crate::packing_list::emit::{emit, header_ops};
use crate::packing_list::fonts::FONT_DIR;
use crate::packing_list::measure::Measure;
use crate::packing_list::{Builder, Layout, Op};
use crate::summary_pdf::{draw_content, draw_picks};

/// One live batch of the date.
pub struct DayBatch<'a> {
    pub record: &'a BatchSaved,
    /// `hh:mm` of its newest save.
    pub time: String,
}

pub struct DayFacts<'a> {
    pub workspace: &'a str,
    pub date: Date,
    /// Newest first, as on the Day screen.
    pub batches: Vec<DayBatch<'a>>,
    /// `hh:mm`.
    pub printed: &'a str,
    /// The couriers of `packing-workflows.toml`, which name the records' courier codes.
    pub couriers: &'a [Courier],
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
    draw_picks(&mut builder, &content::picks(facts));
    let contents = content::order_contents(facts);
    if !contents.is_empty() {
        new_page(&mut builder);
        let heading = "Order contents (orders per content)";
        let right = content::contents_counts(&contents);
        draw_bar(&mut builder, heading, Some(&right), ROW_H * 2.0);
        for row in &contents {
            draw_content(&mut builder, row, &measure, heading);
        }
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

#[cfg(test)]
mod tests;
