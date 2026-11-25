//! What a read reports besides the copies: the no-slip stop, and the unseen logos with the
//! owner's answers.

use std::fmt;

use serde::Deserialize;

use crate::ids::OrderId;

/// A header-band image the courier data does not know. The logo prompt asks the owner about
/// each one once; the answer is remembered in `parser.toml`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnseenLogo {
    /// XXH3-64 (16 hex) of the image's encoded bytes.
    pub hash: String,
    /// XXH3-64 (16 hex) of the image's appearance, when `packing-pdf` computed it.
    pub appearance: Option<String>,
    /// The pages carrying the image.
    pub pages: usize,
    /// The orders carrying it, in download order.
    pub order_ids: Vec<OrderId>,
}

impl UnseenLogo {
    /// The orders carrying the image.
    pub fn orders(&self) -> usize {
        self.order_ids.len()
    }
}

/// The owner's answer about one unseen logo (the logo prompt): the shape the window sends, tagged
/// by `kind`.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LogoAnswer {
    /// The logo is this courier's.
    Courier { name: String },
    /// The image is no courier's logo (a buying channel's strip).
    NotCourier,
    /// Say nothing about it; the orders stay unknown.
    Skip,
}

/// One file's pages without a packing slip (0-based page indices; the message prints page + 1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoSlipFile {
    /// The file index in the batch.
    pub file: usize,
    pub name: String,
    pub pages: Vec<usize>,
}

/// The no-slip stop.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoSlip {
    pub files: Vec<NoSlipFile>,
    pub plain_pages: usize,
    pub total_pages: usize,
}

impl fmt::Display for NoSlip {
    // The warning texts: the file and page list only when some pages carry a slip.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "These labels have no packing slip. {} of {} pages are plain shipping labels",
            self.plain_pages, self.total_pages
        )?;
        if self.plain_pages != self.total_pages {
            let files: Vec<String> = self
                .files
                .iter()
                .map(|file| {
                    let pages: Vec<String> =
                        file.pages.iter().map(|p| (p + 1).to_string()).collect();
                    format!("{}, pages {}", file.name, pages.join(", "))
                })
                .collect();
            write!(f, " ({})", files.join("; "))?;
        }
        write!(
            f,
            ". In the seller centre, download the labels again with Shipping label + Packing slip."
        )
    }
}
