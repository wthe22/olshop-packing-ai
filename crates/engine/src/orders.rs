//! Pages become orders: contents key, display names and the pages → orders step
//! (03 › *Order ID from the label*, *Multi-page orders*; 07 › *What happens in one batch*).

use std::fmt;

use jiff::civil::DateTime;

use crate::label;
use crate::slip::{self, SlipLine};
use crate::text::PageText;

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
    /// Download position, 1..n in first-seen order (07 › *Read*, step 2).
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
    pub warnings: Vec<Warning>,
}

/// A non-fatal problem found while reading; the texts are 08 › *4. Messages*.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Warning {
    /// The slip's `Qty Total` differs from the sum of the order's `Qty`.
    QtyTotalDiffers {
        order_id: String,
        tracking_id: String,
        qty_total: i64,
        sum: i64,
    },
    /// These orders' couriers could not be deduced.
    UnknownCourier {
        order_ids: Vec<String>,
        tracking_ids: Vec<String>,
    },
    /// These orders' tracking IDs were not found on the label.
    TrackingIdMissing { order_ids: Vec<String> },
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
                let label = if tracking_id.is_empty() {
                    format!("Order {order_id}")
                } else {
                    tracking_id.clone()
                };
                write!(
                    f,
                    "Slip total differs for {label}: printed {qty_total}, lines add up to {sum}. \
                     Check that order's slip when packing."
                )
            }
            Warning::UnknownCourier { tracking_ids, .. } => write!(
                f,
                "Courier unknown for {} order{}: {}. They are sorted normally; only a \
                 condition on courier cannot see them.",
                tracking_ids.len(),
                if tracking_ids.len() == 1 { "" } else { "s" },
                tracking_ids.join(", ")
            ),
            Warning::TrackingIdMissing { order_ids } if order_ids.len() == 1 => write!(
                f,
                "No tracking ID read for Order {}. Its label is saved as usual; only a \
                 condition on tracking_id cannot see it.",
                order_ids[0]
            ),
            Warning::TrackingIdMissing { order_ids } => write!(
                f,
                "No tracking ID read for Orders {}. Their labels are saved as usual; only a \
                 condition on tracking_id cannot see them.",
                order_ids.join(", ")
            ),
        }
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

/// One order while its pages are still being merged (labels.py `LabelSet.orders`).
struct Group {
    order_id: String,
    pages: Vec<PageRef>,
    has_slip: bool,
    lines: Vec<SlipLine>,
    qty_total: Option<i64>,
    customer_message: String,
    ship_by: Option<DateTime>,
    tracking_id: String,
    courier: String,
}

fn file_name(file_names: &[&str], file: usize) -> String {
    match file_names.get(file) {
        Some(name) => (*name).to_string(),
        None => format!("file {file}"),
    }
}

fn build_no_slip(no_slip: &[(usize, usize)], total_pages: usize, file_names: &[&str]) -> NoSlip {
    let mut files: Vec<NoSlipFile> = Vec::new();
    for &(file, page) in no_slip {
        match files.iter_mut().find(|entry| entry.file == file) {
            Some(entry) => entry.pages.push(page),
            None => files.push(NoSlipFile {
                file,
                name: file_name(file_names, file),
                pages: vec![page],
            }),
        }
    }
    NoSlip {
        files,
        plain_pages: no_slip.len(),
        total_pages,
    }
}

/// Read a batch into orders (03 › *Order ID from the label*, *Multi-page orders*; 07 ›
/// *What happens in one batch* steps 2-3). `pages` are the pages in download order; `file_names`
/// names the files for the messages. Pages are consumed one at a time, so a 200-page file can be
/// read with progress.
pub fn read_batch<P>(pages: P, file_names: &[&str]) -> Result<Reading, ReadError>
where
    P: IntoIterator<Item = PageItem>,
{
    let mut groups: Vec<Group> = Vec::new();
    let mut continuation_pages: Vec<PageRef> = Vec::new();
    let mut no_slip: Vec<(usize, usize)> = Vec::new();
    let mut total_pages = 0usize;
    let mut current_file: Option<usize> = None;
    let mut previous: Option<String> = None;

    for (file, page, page_text) in pages {
        total_pages += 1;
        if current_file != Some(file) {
            // Python resets `previous` per file: a page with no Order ID cannot continue an
            // order that started in another file.
            current_file = Some(file);
            previous = None;
        }

        let found = label::find_order_id(&page_text.text);
        let order_id = match &found {
            Some(id) => id.clone(),
            None => match &previous {
                // No Order ID = the previous page's order continues (a long label); a first
                // page has nothing to continue.
                Some(id) => {
                    continuation_pages.push(PageRef { file, page });
                    id.clone()
                }
                None => {
                    return Err(ReadError::NoOrderId {
                        name: file_name(file_names, file),
                        page,
                    });
                }
            },
        };
        previous = Some(order_id.clone());

        let slip = slip::read_slip(&page_text);
        if found.is_some() && slip.is_none() {
            no_slip.push((file, page));
        }

        let index = match groups.iter().position(|group| group.order_id == order_id) {
            Some(index) => index,
            None => {
                let tracking_id = label::find_tracking_id(&page_text.text, &order_id);
                let courier = label::deduce_courier(&page_text.text, &tracking_id);
                groups.push(Group {
                    order_id: order_id.clone(),
                    pages: Vec::new(),
                    has_slip: false,
                    lines: Vec::new(),
                    qty_total: None,
                    customer_message: String::new(),
                    ship_by: None,
                    tracking_id,
                    courier,
                });
                groups.len() - 1
            }
        };

        let group = &mut groups[index];
        group.pages.push(PageRef { file, page });
        if group.ship_by.is_none() {
            group.ship_by = label::find_ship_by(&page_text.text);
        }
        if let Some(slip) = slip {
            group.has_slip = true;
            group.lines.extend(slip.lines);
            if slip.qty_total.is_some() {
                group.qty_total = slip.qty_total;
            }
            if !slip.customer_message.is_empty() {
                group.customer_message = slip.customer_message;
            }
        }
    }

    if !no_slip.is_empty() {
        return Err(ReadError::NoSlip(build_no_slip(
            &no_slip,
            total_pages,
            file_names,
        )));
    }

    let mut warnings = Vec::new();
    let mut orders = Vec::with_capacity(groups.len());
    for (index, group) in groups.into_iter().enumerate() {
        let lines: Vec<Line> = group
            .lines
            .iter()
            .map(|line| Line {
                name: line.name.clone(),
                variation: line.variation.clone(),
                seller_sku: line.seller_sku.clone(),
                quantity: line.quantity,
                display_name: display_name(&line.name, &line.variation),
            })
            .collect();
        let sum: i64 = lines.iter().map(|line| line.quantity).sum();
        if let Some(qty_total) = group.qty_total
            && qty_total != sum
        {
            warnings.push(Warning::QtyTotalDiffers {
                order_id: group.order_id.clone(),
                tracking_id: group.tracking_id.clone(),
                qty_total,
                sum,
            });
        }
        let order = Order {
            order_id: group.order_id,
            tracking_id: group.tracking_id,
            courier: group.courier,
            ship_by: group.ship_by,
            lines,
            customer_message: group.customer_message,
        };
        orders.push(ReadOrder {
            order,
            pages: group.pages,
            position: index + 1,
            has_slip: group.has_slip,
            qty_total: group.qty_total,
        });
    }

    let unknown: Vec<&Order> = orders
        .iter()
        .map(|order| &order.order)
        .filter(|order| order.courier.is_empty())
        .collect();
    if !unknown.is_empty() {
        warnings.push(Warning::UnknownCourier {
            order_ids: unknown.iter().map(|o| o.order_id.clone()).collect(),
            // 08 names tracking IDs (what is printed big on the label); the Order ID when none.
            tracking_ids: unknown
                .iter()
                .map(|o| {
                    if o.tracking_id.is_empty() {
                        o.order_id.clone()
                    } else {
                        o.tracking_id.clone()
                    }
                })
                .collect(),
        });
    }
    let missing: Vec<String> = orders
        .iter()
        .filter(|order| order.order.tracking_id.is_empty())
        .map(|order| order.order.order_id.clone())
        .collect();
    if !missing.is_empty() {
        warnings.push(Warning::TrackingIdMissing { order_ids: missing });
    }

    Ok(Reading {
        orders,
        continuation_pages,
        warnings,
    })
}
