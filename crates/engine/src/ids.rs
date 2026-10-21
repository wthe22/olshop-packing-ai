//! The values that identify things in a day: an order's platform Order ID, the courier's tracking
//! ID, and the numbers a day gives its batches and its saved PDFs.
//!
//! Each one is a newtype over the text or number `state.json` stores and serialises
//! transparently, so a stored file and a payload between the app and the window keep their shape.

use std::fmt;

use serde::{Deserialize, Serialize};

/// A platform order's number as its label prints it: 18 digits on the TikTok and Tokopedia
/// labels, 14 characters on the Shopee-style one.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct OrderId(String);

impl OrderId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for OrderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// The courier's parcel number, as printed on the label (`JY0000001234`).
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TrackingId(String);

impl TrackingId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TrackingId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A batch's number within its day: `1, 2, 3, …`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BatchNumber(i64);

impl BatchNumber {
    pub fn new(number: i64) -> Self {
        Self(number)
    }

    /// The number after this one; the day's numbering counts up from 1.
    pub fn next(self) -> Self {
        Self(self.0 + 1)
    }

    /// The number as written in `state.json` and in a batch's folder name.
    pub fn get(self) -> i64 {
        self.0
    }
}

impl fmt::Display for BatchNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A saved PDF's number through the day, counted up from 1 across the day's batches.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PdfNumber(i64);

impl PdfNumber {
    pub fn new(number: i64) -> Self {
        Self(number)
    }

    /// The number after this one.
    pub fn next(self) -> Self {
        Self(self.0 + 1)
    }
}

impl fmt::Display for PdfNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
