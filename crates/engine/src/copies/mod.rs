//! Step 5 of the workflow (02-read.md, duplicates.md): a batch's pages, in reading order, become
//! copies. A page with an Order ID and a label part — a tracking ID or a header image — starts a
//! copy; a page with neither continues the copy right before it, in the same input. Two copies
//! of one Order ID are a repeat, never one longer order.

mod courier;
mod fingerprint;

use std::fmt;

use jiff::civil::Date;

use crate::accounts::OrderAccount;
use crate::ids::{CourierCode, OrderId, TrackingId};
use crate::label::{
    Form, Sender, find_area, find_order_id, find_sender, find_ship_by, find_tracking_id,
    find_weight,
};
use crate::orders::{NoSlip, NoSlipFile, PageItem, UnseenLogo};
use crate::record::{Area, CourierClue, Weight};
use crate::rules::{FilterFields, FilterLine};
use crate::slip::{SlipLine, read_slip};
use crate::text::PageText;
use crate::workflows::Workflows;

pub use self::fingerprint::page_fingerprint;

/// One appearance of an order's label: its first page and the continuation pages right after it.
#[derive(Clone, Debug, PartialEq)]
pub struct Copy {
    pub order_id: OrderId,
    /// The input, by its place in the batch's reading order.
    pub input: usize,
    /// Its pages in the input, counted from 1.
    pub page_start: u16,
    pub page_end: u16,
    pub tracking_id: Option<TrackingId>,
    pub courier: Option<CourierCode>,
    pub courier_clue: CourierClue,
    pub ship_by_date: Option<Date>,
    pub pickup_code: Option<String>,
    /// The receiver's area its first page prints.
    pub area: Area,
    /// The weight its first page prints.
    pub weight: Weight,
    /// The item lines of every slip table of its pages, in page order.
    pub lines: Vec<SlipLine>,
    /// The `Qty Total` of its last slip that prints one.
    pub total_quantity: Option<i64>,
    /// One page fingerprint per page, in page order.
    pub fingerprints: Vec<u64>,
    /// The sender its first page prints with the `Pengirim` caption; none without the caption.
    pub sender: Option<Sender>,
    /// Its account and shop, once its label or the shop prompt names them.
    pub account: Option<OrderAccount>,
}

impl Copy {
    /// Whether the printed `Qty Total` differs from the sum of the copy's lines.
    pub fn total_differs(&self) -> Option<(i64, i64)> {
        let printed = self.total_quantity?;
        let sum: i64 = self.lines.iter().map(|line| line.quantity).sum();
        (printed != sum).then_some((printed, sum))
    }
}

/// The copies of a batch, and the header images the owner must be asked about.
#[derive(Clone, Debug, PartialEq)]
pub struct Copies {
    pub copies: Vec<Copy>,
    pub unseen_logos: Vec<UnseenLogo>,
    pub pages: usize,
}

/// A page that cannot be placed stops the batch, naming the input and the page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CopiesStop {
    /// A label part, but no Order ID by any of the three searches.
    NoOrderId { input: String, page: u16 },
    /// The first page of an input carries no label part, so it continues nothing.
    ContinuesNothing { input: String, page: u16 },
    /// A continuation page names another order than the copy it follows.
    ContinuesAnother {
        input: String,
        page: u16,
        order_id: OrderId,
        copy_of: OrderId,
    },
    /// Pages that start a copy and carry no packing slip.
    NoSlip(NoSlip),
    /// A page headed `Packing List`: the shop's list, not a shipping label.
    NotLabel { input: String, page: u16 },
}

impl fmt::Display for CopiesStop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CopiesStop::NoOrderId { input, page } => write!(
                f,
                "Page {page} of {input} has no Order ID. This does not look like a label \
                 download the app knows. Check the file."
            ),
            CopiesStop::ContinuesNothing { input, page } => write!(
                f,
                "Page {page} of {input} carries no label and continues no label before it. \
                 Download the labels again."
            ),
            CopiesStop::ContinuesAnother {
                input,
                page,
                order_id,
                copy_of,
            } => write!(
                f,
                "Page {page} of {input} is order {order_id} but follows the label of order \
                 {copy_of}. Download the labels again."
            ),
            CopiesStop::NoSlip(no_slip) => no_slip.fmt(f),
            CopiesStop::NotLabel { input, page } => write!(
                f,
                "Page {page} of {input} is a packing list, not a shipping label. Remove it from \
                 the input and read again."
            ),
        }
    }
}

/// The tracking ID forms of the rules' classifiers.
fn forms(workflows: &Workflows) -> Vec<Form<'_>> {
    workflows
        .classifiers
        .iter()
        .filter_map(|c| c.tracking_id.as_ref().map(|form| form.form()))
        .collect()
}

/// The tracking ID, the courier and its clue of one label page, read as a batch reads them: for an
/// order saved before they could be read.
pub fn label_courier(
    workflows: &Workflows,
    page: &PageText,
    order_id: &OrderId,
) -> (Option<TrackingId>, Option<CourierCode>, CourierClue) {
    let tracking_id = find_tracking_id(&page.text, order_id, &forms(workflows));
    let (courier, clue) = courier::name_courier(workflows, page, tracking_id.as_ref());
    let tracking_id =
        tracking_id.or_else(|| courier::pickup_code(page).as_deref().map(TrackingId::new));
    (tracking_id, courier, clue)
}

/// Read a batch's pages into copies. `pages` come in reading order; `inputs` names each input.
pub fn read_copies<P>(
    pages: P,
    inputs: &[&str],
    workflows: &Workflows,
) -> Result<Copies, CopiesStop>
where
    P: IntoIterator<Item = PageItem>,
{
    let forms = forms(workflows);
    let name = |input: usize| {
        inputs
            .get(input)
            .map_or_else(|| format!("file {input}"), |n| n.to_string())
    };
    let mut copies: Vec<Copy> = Vec::new();
    let mut unseen: Vec<UnseenLogo> = Vec::new();
    let mut no_slip: Vec<(usize, usize)> = Vec::new();
    let mut current: Option<usize> = None;
    let mut pages_read = 0usize;
    for (input, index, page) in pages {
        pages_read += 1;
        let number = index as u16 + 1;
        if is_packing_list(&page) {
            return Err(CopiesStop::NotLabel {
                input: name(input),
                page: number,
            });
        }
        let order_id = find_order_id(&page.text);
        let no_id = OrderId::new("");
        let tracking_id = find_tracking_id(&page.text, order_id.as_ref().unwrap_or(&no_id), &forms);
        let slip = read_slip(&page);
        let split_slip = current.map(|at| &copies[at]).is_some_and(|copy| {
            is_split_slip(copy, (input, &page), (&order_id, &tracking_id), &no_slip)
                && slip.is_some()
        });
        let has_label_part = (tracking_id.is_some() || !page.logos.is_empty()) && !split_slip;
        if has_label_part {
            let Some(order_id) = order_id else {
                return Err(CopiesStop::NoOrderId {
                    input: name(input),
                    page: number,
                });
            };
            if slip.is_none() {
                no_slip.push((input, index));
            }
            let (courier, courier_clue) =
                courier::name_courier(workflows, &page, tracking_id.as_ref());
            // An instant label's header images are the buying channel's strips: its tag names
            // the courier, and a strip named as a courier would unname every such label.
            if tracking_id.is_some() || !crate::label::is_instant(&page.text) {
                note_unseen(&mut unseen, workflows, &page, &order_id);
            }
            let pickup_code = courier::pickup_code(&page);
            // An instant label prints no tracking ID: its pickup code is the one the courier uses.
            let tracking_id = tracking_id.or_else(|| pickup_code.as_deref().map(TrackingId::new));
            copies.push(Copy {
                order_id,
                input,
                page_start: number,
                page_end: number,
                tracking_id,
                courier,
                courier_clue,
                ship_by_date: find_ship_by(&page.text).map(|time| time.date()),
                pickup_code,
                area: find_area(&page),
                weight: find_weight(&page),
                lines: slip.as_ref().map(|s| s.lines.clone()).unwrap_or_default(),
                total_quantity: slip.as_ref().and_then(|s| s.qty_total),
                fingerprints: vec![page_fingerprint(&page)],
                sender: find_sender(&page),
                account: None,
            });
            current = Some(copies.len() - 1);
            continue;
        }
        let open = current.filter(|&at| copies[at].input == input);
        let Some(at) = open else {
            return Err(CopiesStop::ContinuesNothing {
                input: name(input),
                page: number,
            });
        };
        let copy = &mut copies[at];
        if let Some(order_id) = order_id
            && order_id != copy.order_id
        {
            return Err(CopiesStop::ContinuesAnother {
                input: name(input),
                page: number,
                order_id,
                copy_of: copy.order_id.clone(),
            });
        }
        copy.page_end = number;
        copy.fingerprints.push(page_fingerprint(&page));
        if copy.ship_by_date.is_none() {
            copy.ship_by_date = find_ship_by(&page.text).map(|time| time.date());
        }
        if let Some(slip) = slip {
            let label_page = (copy.input, usize::from(copy.page_start) - 1);
            no_slip.retain(|&page| page != label_page);
            copy.lines.extend(slip.lines);
            if slip.qty_total.is_some() {
                copy.total_quantity = slip.qty_total;
            }
        }
    }
    if !no_slip.is_empty() {
        return Err(CopiesStop::NoSlip(no_slip_stop(
            &no_slip, pages_read, &name,
        )));
    }
    Ok(Copies {
        copies,
        unseen_logos: unseen,
        pages: pages_read,
    })
}

/// A packing list is the shop's own page: it prints no tracking ID and no courier, and it must
/// never join a copy as a slip.
fn is_packing_list(page: &PageText) -> bool {
    page.text.contains("Packing List")
}

/// The slip half of a split label: a page right after a label page that had no slip, in the same
/// input, with the same Order ID, no header image and no tracking ID but the label's. It prints the
/// tracking ID beside the slip, so it would otherwise start a second copy of the order.
fn is_split_slip(
    copy: &Copy,
    (input, page): (usize, &crate::text::PageText),
    (order_id, tracking_id): (&Option<OrderId>, &Option<TrackingId>),
    no_slip: &[(usize, usize)],
) -> bool {
    let label_page = (copy.input, usize::from(copy.page_start) - 1);
    copy.input == input
        && copy.page_end == copy.page_start
        && no_slip.last() == Some(&label_page)
        && page.logos.is_empty()
        && order_id.as_ref() == Some(&copy.order_id)
        && (tracking_id.is_none() || *tracking_id == copy.tracking_id)
}

/// Count a header image no list knows, for the logo prompt.
fn note_unseen(
    unseen: &mut Vec<UnseenLogo>,
    workflows: &Workflows,
    page: &crate::text::PageText,
    order_id: &OrderId,
) {
    for image in &page.logos {
        let known = |hash: &String| {
            workflows.not_courier_logos.contains(hash)
                || workflows
                    .classifiers
                    .iter()
                    .any(|c| c.logos.contains(hash) || c.shapes.contains(hash))
        };
        if known(&image.hash) || image.appearance.as_ref().is_some_and(known) {
            continue;
        }
        match unseen.iter_mut().find(|entry| entry.hash == image.hash) {
            Some(entry) => {
                entry.pages += 1;
                if !entry.order_ids.contains(order_id) {
                    entry.order_ids.push(order_id.clone());
                }
            }
            None => unseen.push(UnseenLogo {
                hash: image.hash.clone(),
                appearance: image.appearance.clone(),
                pages: 1,
                order_ids: vec![order_id.clone()],
            }),
        }
    }
}

fn no_slip_stop(pages: &[(usize, usize)], total: usize, name: &dyn Fn(usize) -> String) -> NoSlip {
    let mut files: Vec<NoSlipFile> = Vec::new();
    for &(input, page) in pages {
        match files.iter_mut().find(|file| file.file == input) {
            Some(file) => file.pages.push(page),
            None => files.push(NoSlipFile {
                file: input,
                name: name(input),
                pages: vec![page],
            }),
        }
    }
    NoSlip {
        files,
        plain_pages: pages.len(),
        total_pages: total,
    }
}

impl FilterFields for Copy {
    fn lines(&self) -> Vec<FilterLine<'_>> {
        self.lines
            .iter()
            .map(|line| FilterLine {
                name: &line.name,
                variation: &line.variation,
                seller_sku: &line.seller_sku,
                quantity: line.quantity,
            })
            .collect()
    }

    fn courier(&self) -> &str {
        self.courier.as_ref().map_or("", CourierCode::as_str)
    }

    fn order_id(&self) -> &str {
        self.order_id.as_str()
    }

    fn tracking_id(&self) -> &str {
        self.tracking_id.as_ref().map_or("", TrackingId::as_str)
    }

    fn ship_by(&self) -> Option<Date> {
        self.ship_by_date
    }

    fn account(&self) -> &str {
        self.account.as_ref().map_or("", |found| &found.account)
    }

    fn shop(&self) -> &str {
        self.account
            .as_ref()
            .and_then(|found| found.shop.as_deref())
            .unwrap_or_default()
    }
}
