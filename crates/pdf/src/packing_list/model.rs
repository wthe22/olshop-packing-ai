//! The packing list's data model: the three layouts, one slip line, one order of a run, one
//! run, one saved PDF's section, and the whole list with the counts its header prints.

use std::fmt;

use packing_engine::ids::{BatchNumber, PdfNumber, TrackingId};

/// The three layouts (03 › *Packing list (A4)*).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layout {
    /// Item lines and the run's tracking IDs.
    Full,
    /// Item lines, no tracking IDs.
    Summary,
    /// Only the pick summary.
    Pick,
}

impl Layout {
    /// Parse a layout name; `None` for an unknown one.
    pub fn parse(name: &str) -> Option<Layout> {
        match name {
            "full" => Some(Layout::Full),
            "summary" => Some(Layout::Summary),
            "pick" => Some(Layout::Pick),
            _ => None,
        }
    }

    pub(super) fn tracking(self) -> bool {
        self == Layout::Full
    }
}

impl fmt::Display for Layout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Layout::Full => "full",
            Layout::Summary => "summary",
            Layout::Pick => "pick",
        })
    }
}

/// One slip line of an order as the packing list needs it (Python `orders.Line`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackingLine {
    pub name: String,
    /// Empty for the slip's `Default` variation.
    pub variation: String,
    pub quantity: i64,
    /// `orders.display_name(name, variation)`.
    pub display_name: String,
}

/// One order of a run (Python `orders.Order`): its tracking ID and slip lines.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunOrder {
    /// The courier's parcel number; `None` prints an empty cell.
    pub tracking_id: Option<TrackingId>,
    pub lines: Vec<PackingLine>,
}

/// One run of a saved PDF (Python `picks.Run`): a group of orders with identical contents.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunRows {
    /// The run number; the label is printed as `{section.number}-{number:02}`.
    pub number: u32,
    pub orders: Vec<RunOrder>,
}

/// One saved PDF (Python `picks.SavedPdf`): a pick's number, code, name and its runs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SavedPdfSection {
    pub number: PdfNumber,
    pub code: String,
    pub name: String,
    pub runs: Vec<RunRows>,
}

/// The whole packing list: the day, the batch number (07 › *Packing list*) and the sections
/// (one for scope `per-pdf`, all for scope `whole`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackingList {
    /// `YYYY-MM-DD`.
    pub day: String,
    /// The batch number shown in the header, when the list belongs to one batch.
    pub batch: Option<BatchNumber>,
    /// Already-formatted `HH:MM` print time.
    pub printed: String,
    pub sections: Vec<SavedPdfSection>,
}

impl PackingList {
    pub(super) fn orders(&self) -> usize {
        self.sections
            .iter()
            .flat_map(|section| &section.runs)
            .map(|run| run.orders.len())
            .sum()
    }

    pub(super) fn units(&self) -> i64 {
        self.sections
            .iter()
            .flat_map(|section| &section.runs)
            .flat_map(|run| &run.orders)
            .flat_map(|order| &order.lines)
            .map(|line| line.quantity)
            .sum()
    }

    pub(super) fn run_count(&self) -> usize {
        self.sections.iter().map(|section| section.runs.len()).sum()
    }

    /// The counts a packing list prints in its header and section bars: `(orders, units, runs)`.
    pub fn counts(&self) -> (usize, i64, usize) {
        (self.orders(), self.units(), self.run_count())
    }
}
