//! Pages become orders: contents key and display names.

use jiff::civil::DateTime;

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
    pub order_id: String,
    pub tracking_id: String,
    /// Deduced from the label; empty when unknown.
    pub courier: String,
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
