//! A summary as its parts, in the order the PDF prints them: what the summary PDF and the day
//! summary draw, and what the app's Summary screen shows (ui/summary.md). Built without PDFium.

use serde::Serialize;

pub use crate::facts::{Count, Row, Section};
pub use crate::summary_pdf::content::ContentRow;

/// The pick part's heading.
pub(crate) const PICKS_HEADING: &str = "Pick summary (units to take from stock)";

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SummaryView {
    /// `Summary · All accounts · 2026-10-07 · batch 2`, the PDF's title.
    pub title: String,
    /// The header's counts; the PDF adds the time it was printed.
    pub counts: String,
    pub parts: Vec<Part>,
}

/// One part under its heading bar. The PDF starts a part of another kind than the one before on a
/// new page.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Part {
    /// Ruled rows: the accounts, couriers, inputs, a batch as its Day card.
    Facts(Section),
    /// Numbered order contents with their orders: a group of a batch, or the day's contents.
    Contents {
        heading: String,
        right: String,
        rows: Vec<ContentRow>,
    },
    /// The units to take from stock per item.
    Picks { heading: String, rows: Vec<Pick> },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Pick {
    pub units: i64,
    pub item: String,
}

impl Part {
    /// The pick part of `picks`; none when there is nothing to pick.
    pub(crate) fn picks(picks: Vec<(i64, String)>) -> Option<Part> {
        (!picks.is_empty()).then(|| Part::Picks {
            heading: PICKS_HEADING.to_string(),
            rows: picks
                .into_iter()
                .map(|(units, item)| Pick { units, item })
                .collect(),
        })
    }
}
