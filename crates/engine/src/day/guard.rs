//! The duplicate guard: the orders already saved today, with the batch and time they were first
//! saved (07 › *What happens in one batch* step 4).

use std::fmt;

use crate::ids::{BatchNumber, OrderId, PdfNumber, TrackingId};
use crate::orders::ReadOrder;

use super::state::{DayState, SavedOrder};

impl DayState {
    fn saved_entry(&self, order_id: &OrderId) -> Option<&SavedOrder> {
        self.saved
            .iter()
            .filter(|order| &order.order_id == order_id)
            .min_by_key(|order| order.batch)
    }

    /// The duplicate guard: drop the orders already saved today and list them with the batch,
    /// saved-PDF number and time they were first saved (07 › *What happens in one batch* step 4).
    pub fn drop_saved(&self, orders: &[ReadOrder]) -> (Vec<ReadOrder>, Vec<Duplicate>) {
        let mut kept = Vec::new();
        let mut duplicates = Vec::new();
        for order in orders {
            match self.saved_entry(&order.order.order_id) {
                None => kept.push(order.clone()),
                Some(entry) => duplicates.push(Duplicate {
                    order_id: entry.order_id.clone(),
                    tracking_id: entry.tracking_id.clone(),
                    batch: entry.batch,
                    pdf: entry.pdf,
                    time: self
                        .batches
                        .iter()
                        .find(|batch| batch.batch == entry.batch)
                        .map(|batch| batch.save_time.clone())
                        .unwrap_or_default(),
                }),
            }
        }
        (kept, duplicates)
    }
}

/// An order already saved today (07 › *What happens in one batch* step 4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Duplicate {
    pub order_id: OrderId,
    pub tracking_id: Option<TrackingId>,
    pub batch: BatchNumber,
    pub pdf: PdfNumber,
    pub time: String,
}

impl fmt::Display for Duplicate {
    // 08 › *Messages*: the warning lists `<tracking ID>` (batch `<b>`, PDF `<k>`, `<time>`).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} (batch {}, PDF {}, {})",
            self.tracking_id.as_ref().map_or("", TrackingId::as_str),
            self.batch,
            self.pdf,
            self.time
        )
    }
}

/// The warning for the orders left out by the duplicate guard (08 › *Messages*).
pub fn duplicates_warning(duplicates: &[Duplicate]) -> String {
    let list: Vec<String> = duplicates.iter().map(|dup| dup.to_string()).collect();
    format!(
        "{} orders were already saved today and are left out: {}",
        duplicates.len(),
        list.join(", ")
    )
}
