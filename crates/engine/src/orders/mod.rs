//! The order and line model and what a read produces: the contents key, display names and the
//! types the pages → orders step fills. The step itself is in the modules beside this file.

use jiff::civil::DateTime;

use crate::ids::{OrderId, TrackingId};
use crate::text::PageText;

mod read;
mod warnings;

pub use self::read::read_batch;
pub use self::warnings::{
    LogoAnswer, NoSlip, NoSlipFile, OrderLabel, ReadError, UnseenLogo, Warning,
};

/// The slip's placeholder for a product without variants (Python `DEFAULT_VARIATION`).
pub const DEFAULT_VARIATION: &str = "Default";

/// One item line of an order, from the packing slip.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    /// Full Product Name.
    pub name: String,
    /// Empty when the slip says `Default`.
    pub variation: String,
    pub seller_sku: String,
    pub quantity: i64,
    pub display_name: String,
}

/// One order as the PC app knows it: label fields and slip lines only (no CSV).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Order {
    pub order_id: OrderId,
    /// The courier's parcel number; `None` when the label printed none.
    pub tracking_id: Option<TrackingId>,
    /// Deduced from the label; `None` when unknown.
    pub courier: Option<String>,
    /// The label's `In transit by`.
    pub ship_by: Option<DateTime>,
    /// Slip row order.
    pub lines: Vec<Line>,
    pub customer_message: String,
}

impl Order {
    pub fn total_quantity(&self) -> i64 {
        self.lines.iter().map(|line| line.quantity).sum()
    }

    pub fn distinct_items(&self) -> i64 {
        self.lines.len() as i64
    }

    /// Identical-contents key (01-requirements): per line (name, variation, quantity), sorted.
    pub fn contents(&self) -> Vec<(String, String, i64)> {
        let mut key: Vec<_> = self
            .lines
            .iter()
            .map(|line| (line.name.clone(), line.variation.clone(), line.quantity))
            .collect();
        key.sort();
        key
    }
}

/// `orders.display_name`: the name up to the first `|` plus the variation after an em dash;
/// `Default` is no variation.
pub fn display_name(name: &str, variation: &str) -> String {
    let short = name.split('|').next().unwrap_or("").trim();
    let variant = variation.trim();
    if !variant.is_empty() && variant != DEFAULT_VARIATION {
        format!("{short} — {variant}")
    } else {
        short.to_string()
    }
}

/// One page of a batch: the file index (in download order), the 0-based page index inside that
/// file, and the page text.
pub type PageItem = (usize, usize, PageText);

/// A page reference: the file index and the 0-based page index.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PageRef {
    pub file: usize,
    pub page: usize,
}

/// One order read from the labels, ready to use.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadOrder {
    pub order: Order,
    /// The order's pages, in download order.
    pub pages: Vec<PageRef>,
    /// Download position, 1..n in first-seen order.
    pub position: usize,
    pub has_slip: bool,
    pub qty_total: Option<i64>,
}

/// The result of reading a batch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reading {
    /// The orders, in download order.
    pub orders: Vec<ReadOrder>,
    /// Pages without an Order ID that continue the previous page's order.
    pub continuation_pages: Vec<PageRef>,
    /// The header-band images the courier data does not know, in first-seen order.
    pub unseen_logos: Vec<UnseenLogo>,
    pub warnings: Vec<Warning>,
}
