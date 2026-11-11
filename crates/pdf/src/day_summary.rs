//! The day summary (day-summary.md): the open date of every account or of one on A4, from its live
//! batches (an account's day summary: its orders, and the batches that hold one) —
//! each batch as its Day card shows it, its groups with their orders and order contents; the
//! couriers' counts; each batch's inputs with its ledger; then from a new page the pick pages of
//! the whole day, and from another the day's order contents with their orders, most orders first.
//! Drawn in the summary PDF's layout. A view of the journal: nothing reads it back, so its counts
//! read singular where they should.

mod content;

use std::path::Path;

use jiff::civil::Date;
use packing_engine::record::{BatchSaved, of_account};
use packing_engine::workflows::Courier;
use pdfium_render::prelude::Pdfium;

use crate::PdfError;
use crate::facts::{ROW_H, draw_bar, draw_section, new_page};
use crate::page::emit::{emit, header_ops};
use crate::page::fonts::FONT_DIR;
use crate::page::measure::Measure;
use crate::page::{Builder, Op};
use crate::summary_pdf::{draw_content, draw_picks};

/// One live batch of the date.
pub struct DayBatch<'a> {
    pub record: &'a BatchSaved,
    /// `hh:mm` of its newest save.
    pub time: String,
}

pub struct DayFacts<'a> {
    pub date: Date,
    /// Newest first, as on the Day screen.
    pub batches: Vec<DayBatch<'a>>,
    /// `hh:mm`.
    pub printed: &'a str,
    /// The couriers of `packing-workflows.toml`, which name the records' courier codes.
    pub couriers: &'a [Courier],
    /// The account whose day summary this is; none: all accounts.
    pub account: Option<&'a str>,
}

/// Write the day summary and return its page count.
pub fn write_day_summary(
    pdfium: &Pdfium,
    path: &Path,
    facts: &DayFacts<'_>,
) -> Result<usize, PdfError> {
    let mine: Vec<BatchSaved> = match facts.account {
        Some(account) => facts
            .batches
            .iter()
            .map(|batch| of_account(batch.record, account))
            .collect(),
        None => Vec::new(),
    };
    let account_facts;
    let shown = match facts.account {
        None => facts,
        Some(account) => {
            account_facts = DayFacts {
                batches: facts
                    .batches
                    .iter()
                    .zip(&mine)
                    .filter(|(_, mine)| !mine.orders.is_empty())
                    .map(|(batch, mine)| DayBatch {
                        record: mine,
                        time: batch.time.clone(),
                    })
                    .collect(),
                account: Some(account),
                ..*facts
            };
            &account_facts
        }
    };
    let title = format!(
        "Day summary · {} · {}",
        facts.account.unwrap_or("All accounts"),
        facts.date
    );
    let subtitle = format!(
        "{} · printed {}",
        content::header_counts(shown),
        facts.printed
    );

    let font_dir = Path::new(FONT_DIR);
    let measure = Measure::new(pdfium, font_dir)?;
    let mut builder = Builder::new(&measure);
    for section in content::sections(shown, facts) {
        draw_section(&mut builder, &section, &measure);
    }
    draw_picks(&mut builder, &content::picks(shown));
    let contents = content::order_contents(shown);
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
