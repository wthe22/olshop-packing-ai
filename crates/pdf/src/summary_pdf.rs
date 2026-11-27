//! The summary PDF of a batch (summary-pdf.md), each part from a new page: the facts pages, which
//! say what the batch is so the paper alone can be traced; each group with its numbered order
//! contents; and the pick pages, what to take from stock. The verifier reads the header's counts,
//! the ledger and each group bar's counts back as text, so their words are fixed and their numbers
//! have no thousands separator. The day summary is drawn the same way from its own parts.

pub(crate) mod content;

use std::path::Path;

use jiff::civil::Date;
use packing_engine::accounts::Account;
use packing_engine::record::{BatchSaved, of_account};
use packing_engine::workflows::Courier;
use pdfium_render::prelude::Pdfium;

use crate::PdfError;
use crate::facts::{
    GAP_BELOW_SECTION, KEY_W, ROW_H, ROW_PT, RULE_GAP, TEXT_RIGHT, TEXT_X, draw_bar, draw_rule,
    draw_section, keep_together, new_page,
};
use crate::page::emit::{emit, header_ops};
use crate::page::fonts::{FONT_DIR, Font};
use crate::page::measure::Measure;
use crate::page::{Builder, CELL_MARGIN, Cell, Op};
use crate::summary_view::{Part, SummaryView};

use self::content::ContentRow;

/// The width of an order content's count of orders, at the right edge.
const COUNT_W: f32 = 20.0;

pub struct SummaryFacts<'a> {
    pub date: Date,
    /// The batch's record, its group PDFs already in `outputs`.
    pub record: &'a BatchSaved,
    /// The account whose summary this is (summary-pdf.md, *One account*); none: all accounts.
    pub account: Option<&'a str>,
    /// The app version the journal's records carry.
    pub app: &'a str,
    /// The couriers of `packing-workflows.toml`, which name the record's courier codes.
    pub couriers: &'a [Courier],
    /// The accounts of `config.toml`, which give each order its account by its shop.
    pub accounts: &'a [Account],
}

/// The batch's summary as its parts: the facts, each group, the picks.
pub fn summary_view(facts: &SummaryFacts<'_>) -> SummaryView {
    let whole = facts.record;
    let mine = facts
        .account
        .map(|account| of_account(whole, account, facts.accounts));
    let record = mine.as_ref().unwrap_or(whole);
    let counts = content::counts(record);
    let groups = content::group_sections(record)
        .into_iter()
        .map(|group| Part::Contents {
            heading: format!("{}  {}", group.code, group.name),
            right: format!("{} orders · {} units", group.orders, group.units),
            rows: group.contents,
        });
    SummaryView {
        title: format!(
            "Summary · {} · {} · batch {}",
            facts.account.unwrap_or("All accounts"),
            facts.date,
            record.batch
        ),
        counts: format!(
            "{} orders · {} units · {} groups",
            counts.orders, counts.units, counts.groups
        ),
        parts: content::facts(record, whole, facts)
            .into_iter()
            .map(Part::Facts)
            .chain(groups)
            .chain(Part::picks(content::picks([record])))
            .collect(),
    }
}

/// Write the summary PDF, printed at `printed` (`hh:mm`), and return its page count.
pub fn write_summary(
    pdfium: &Pdfium,
    path: &Path,
    facts: &SummaryFacts<'_>,
    printed: &str,
) -> Result<usize, PdfError> {
    write_view(pdfium, path, &summary_view(facts), printed)
}

/// Draw `view`'s parts, a part of another kind than the one before from a new page, under the
/// header of its title and counts.
pub(crate) fn write_view(
    pdfium: &Pdfium,
    path: &Path,
    view: &SummaryView,
    printed: &str,
) -> Result<usize, PdfError> {
    let font_dir = Path::new(FONT_DIR);
    let measure = Measure::new(pdfium, font_dir)?;
    let mut builder = Builder::new(&measure);
    let mut before = None;
    for part in &view.parts {
        let kind = std::mem::discriminant(part);
        if before.is_some_and(|before| before != kind) {
            new_page(&mut builder);
        }
        before = Some(kind);
        match part {
            Part::Facts(section) => draw_section(&mut builder, section, &measure),
            Part::Contents {
                heading,
                right,
                rows,
            } => {
                draw_bar(&mut builder, heading, Some(right), ROW_H * 2.0);
                for row in rows {
                    draw_content(&mut builder, row, &measure, heading);
                }
                builder.y += GAP_BELOW_SECTION;
            }
            Part::Picks { heading, rows } => {
                draw_bar(&mut builder, heading, None, ROW_H);
                let rows: Vec<(i64, String)> = rows
                    .iter()
                    .map(|pick| (pick.units, pick.item.clone()))
                    .collect();
                builder.draw_summary_rows(&rows);
            }
        }
    }

    let subtitle = format!("{} · printed {printed}", view.counts);
    let body = builder.pages;
    let count = body.len();
    let pages: Vec<Vec<Op>> = body
        .into_iter()
        .enumerate()
        .map(|(index, ops)| {
            let mut page = header_ops(&view.title, &subtitle, index + 1, count, &measure);
            page.extend(ops);
            page
        })
        .collect();
    emit(pdfium, path, &pages, font_dir, &measure)
}

/// One order content: its number, its items, each wrapped at spaces, and its number of orders at
/// the right; a content that does not fit starts on the next page under its section's bar.
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
