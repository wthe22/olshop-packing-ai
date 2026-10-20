//! What a read reports besides the orders: the non-fatal warnings, the stops, and the unseen-logo
//! types and the owner's answers (08 › *4. Messages*; D-LABEL-4).

use std::fmt;

use serde::Deserialize;

use crate::ids::{OrderId, TrackingId};
use crate::label::{self, CourierTable};

use super::{Order, ReadOrder, Reading};

/// A non-fatal problem found while reading; the texts are 08 › *4. Messages*.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Warning {
    /// The slip's `Qty Total` differs from the sum of the order's `Qty`.
    QtyTotalDiffers {
        order_id: OrderId,
        tracking_id: Option<TrackingId>,
        qty_total: i64,
        sum: i64,
    },
    /// These orders' couriers could not be deduced.
    UnknownCourier { orders: Vec<OrderLabel> },
    /// These orders' tracking IDs were not found on the label.
    TrackingIdMissing { order_ids: Vec<OrderId> },
}

/// How a read warning names an order: by the tracking ID that is printed big on the label, or by
/// the Order ID when the label printed no tracking ID (08 › *Messages*).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OrderLabel {
    Tracking(TrackingId),
    Order(OrderId),
}

impl fmt::Display for OrderLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrderLabel::Tracking(tracking_id) => tracking_id.fmt(f),
            OrderLabel::Order(order_id) => order_id.fmt(f),
        }
    }
}

impl Warning {
    /// The unknown-courier warning for a set of orders (08 › *Messages*), `None` when every
    /// order's courier is named. Read builds it after the deduction; the logo prompt uses it
    /// again to refresh the warning after an answer named some of those orders (D-LABEL-4).
    pub fn unknown_courier(orders: &[ReadOrder]) -> Option<Warning> {
        let unknown: Vec<&Order> = orders
            .iter()
            .map(|order| &order.order)
            .filter(|order| order.courier.is_none())
            .collect();
        if unknown.is_empty() {
            return None;
        }
        Some(Warning::UnknownCourier {
            orders: unknown
                .iter()
                .map(|order| match &order.tracking_id {
                    Some(tracking_id) => OrderLabel::Tracking(tracking_id.clone()),
                    // 08 names tracking IDs (what is printed big on the label); the Order ID
                    // when none was read.
                    None => OrderLabel::Order(order.order_id.clone()),
                })
                .collect(),
        })
    }
}

impl fmt::Display for Warning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Warning::QtyTotalDiffers {
                order_id,
                tracking_id,
                qty_total,
                sum,
            } => {
                let label = match tracking_id {
                    Some(tracking_id) => tracking_id.to_string(),
                    None => format!("Order {order_id}"),
                };
                write!(
                    f,
                    "Slip total differs for {label}: printed {qty_total}, lines add up to {sum}. \
                     Check that order's slip when packing."
                )
            }
            Warning::UnknownCourier { orders } => {
                let labels: Vec<String> = orders.iter().map(OrderLabel::to_string).collect();
                write!(
                    f,
                    "Courier unknown for {} order{}: {}. They are sorted normally; only a \
                     condition on courier cannot see them.",
                    labels.len(),
                    if labels.len() == 1 { "" } else { "s" },
                    labels.join(", ")
                )
            }
            Warning::TrackingIdMissing { order_ids } if order_ids.len() == 1 => write!(
                f,
                "No tracking ID read for Order {}. Its label is saved as usual; only a \
                 condition on tracking_id cannot see it.",
                order_ids[0]
            ),
            Warning::TrackingIdMissing { order_ids } => {
                let ids: Vec<String> = order_ids.iter().map(OrderId::to_string).collect();
                write!(
                    f,
                    "No tracking ID read for Orders {}. Their labels are saved as usual; only a \
                     condition on tracking_id cannot see them.",
                    ids.join(", ")
                )
            }
        }
    }
}

/// A header-band image the courier data does not know (Q-LABEL-1, D-LABEL-4). The logo prompt
/// asks the owner about each one once; the answer is remembered in `couriers.toml` (D-LABEL-4).
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

    /// Apply the owner's answer about this logo to a set of orders (the logo prompt, D-LABEL-4).
    /// A courier name fills the `courier` of the orders carrying the logo **only when the two
    /// clues agree**: the order's tracking-ID form must have that courier among its candidates
    /// (`table`; D-LABEL-2), else the order stays unknown. *Not a courier* and *Skip* name
    /// nothing. Returns how many orders the answer named.
    pub fn apply(
        &self,
        orders: &mut [ReadOrder],
        answer: &LogoAnswer,
        table: &CourierTable,
    ) -> usize {
        let LogoAnswer::Courier { name } = answer else {
            return 0;
        };
        let mut named = 0;
        for order in orders {
            if !self.order_ids.iter().any(|id| id == &order.order.order_id) {
                continue;
            }
            let Some(tracking_id) = &order.order.tracking_id else {
                continue;
            };
            if order.order.courier.is_none()
                && label::form_names_courier_in(table, tracking_id, name)
            {
                order.order.courier = Some(name.clone());
                named += 1;
            }
        }
        named
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

impl Reading {
    /// Apply one answer to the unseen logo `hash` (the logo prompt, D-LABEL-4). A courier name
    /// fills the `courier` of the orders carrying the logo **only when the two clues agree**
    /// ([`UnseenLogo::apply`]); *Not a courier* and *Skip* name nothing. The answered logo leaves
    /// `unseen_logos`. Returns how many orders the answer named.
    pub fn apply_logo_answer(
        &mut self,
        hash: &str,
        answer: &LogoAnswer,
        table: &CourierTable,
    ) -> usize {
        let Some(index) = self
            .unseen_logos
            .iter()
            .position(|entry| entry.hash == hash)
        else {
            return 0;
        };
        let entry = self.unseen_logos.remove(index);
        entry.apply(&mut self.orders, answer, table)
    }
}

/// One file's pages without a packing slip (0-based page indices; the message prints page + 1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoSlipFile {
    /// The file index in the batch.
    pub file: usize,
    pub name: String,
    pub pages: Vec<usize>,
}

/// The no-slip stop (07 › *Every page must carry a packing slip*).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoSlip {
    pub files: Vec<NoSlipFile>,
    pub plain_pages: usize,
    pub total_pages: usize,
}

impl fmt::Display for NoSlip {
    // Texts of 08 › Messages: the file and page list only when some pages carry a slip.
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

/// A read that stopped the batch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReadError {
    /// A first page with no Order ID and nothing to continue (labels.py raises here).
    NoOrderId { name: String, page: usize },
    /// Pages without a packing slip.
    NoSlip(NoSlip),
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReadError::NoOrderId { name, page } => {
                write!(
                    f,
                    "Page {} of {name} has no Order ID. This does not look like a TikTok Shop \
                     label download. Check the file.",
                    page + 1
                )
            }
            ReadError::NoSlip(err) => err.fmt(f),
        }
    }
}
