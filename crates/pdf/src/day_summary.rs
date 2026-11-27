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
use packing_engine::accounts::Account;
use packing_engine::record::{BatchSaved, of_account};
use packing_engine::workflows::Courier;
use pdfium_render::prelude::Pdfium;

use crate::PdfError;
use crate::summary_pdf::write_view;
use crate::summary_view::{Part, SummaryView};

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
    /// The couriers of `packing-workflows.toml`, which name the records' courier codes.
    pub couriers: &'a [Courier],
    /// The account whose day summary this is; none: all accounts.
    pub account: Option<&'a str>,
    /// The accounts of `config.toml`, which give each order its account by its shop.
    pub accounts: &'a [Account],
}

/// The day summary as its parts: each batch, the accounts, couriers and inputs; the day's picks;
/// the day's order contents.
pub fn day_summary_view(facts: &DayFacts<'_>) -> SummaryView {
    let mine: Vec<BatchSaved> = match facts.account {
        Some(account) => facts
            .batches
            .iter()
            .map(|batch| of_account(batch.record, account, facts.accounts))
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
    let contents = content::order_contents(shown);
    let contents = (!contents.is_empty()).then(|| Part::Contents {
        heading: "Order contents (orders per content)".to_string(),
        right: content::contents_counts(&contents),
        rows: contents,
    });
    SummaryView {
        title: format!(
            "Day summary · {} · {}",
            facts.account.unwrap_or("All accounts"),
            facts.date
        ),
        counts: content::header_counts(shown),
        parts: content::sections(shown, facts)
            .into_iter()
            .map(Part::Facts)
            .chain(Part::picks(content::picks(shown)))
            .chain(contents)
            .collect(),
    }
}

/// Write the day summary, printed at `printed` (`hh:mm`), and return its page count.
pub fn write_day_summary(
    pdfium: &Pdfium,
    path: &Path,
    facts: &DayFacts<'_>,
    printed: &str,
) -> Result<usize, PdfError> {
    write_view(pdfium, path, &day_summary_view(facts), printed)
}

#[cfg(test)]
mod tests;
