//! The error a batch step fails with.

use std::fmt;

use packing_engine::day::DayError;
use packing_engine::ids::{BatchNumber, OrderId};
use packing_engine::orders::ReadError;

use crate::PdfError;

/// A batch step failed.
#[derive(Debug)]
pub enum BatchError {
    /// A PDF job failed (a file cannot be opened, a write failed).
    Pdf(PdfError),
    /// Saving into the day folder failed.
    Day(DayError),
    /// Reading stopped the batch (no Order ID on a first page, a page with no packing slip).
    Read(ReadError),
    /// Every order was already saved today.
    AllSaved { orders: usize, batch: BatchNumber },
    /// A plain message.
    Message(String),
}

impl fmt::Display for BatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BatchError::Pdf(error) => error.fmt(f),
            BatchError::Day(error) => error.fmt(f),
            BatchError::Read(error) => error.fmt(f),
            BatchError::AllSaved { orders, batch } => write!(
                f,
                "All {orders} orders were already saved today (batch {batch}). \
                 These labels were prepared before; nothing to do."
            ),
            BatchError::Message(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for BatchError {}

impl From<PdfError> for BatchError {
    fn from(error: PdfError) -> Self {
        BatchError::Pdf(error)
    }
}

impl From<DayError> for BatchError {
    fn from(error: DayError) -> Self {
        BatchError::Day(error)
    }
}

impl From<ReadError> for BatchError {
    fn from(error: ReadError) -> Self {
        BatchError::Read(error)
    }
}

pub(super) fn missing(id: &OrderId) -> BatchError {
    BatchError::Message(format!("order {id} is not in the draft"))
}
