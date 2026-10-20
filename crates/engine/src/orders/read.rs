//! The pages → orders step: group each page's Order ID into one order, count the unseen logos and
//! build the read warnings and the no-slip stop (07 › *What happens in one batch* steps 2-3).

use jiff::civil::DateTime;

use crate::ids::{OrderId, TrackingId};
use crate::label::{self, CourierTable};
use crate::slip::{self, SlipLine};
use crate::text::LogoImage;

use super::{
    Line, NoSlip, NoSlipFile, Order, PageItem, PageRef, ReadError, ReadOrder, Reading, UnseenLogo,
    Warning, display_name,
};

/// One order while its pages are still being merged (labels.py `LabelSet.orders`).
struct Group {
    order_id: OrderId,
    pages: Vec<PageRef>,
    has_slip: bool,
    lines: Vec<SlipLine>,
    qty_total: Option<i64>,
    customer_message: String,
    ship_by: Option<DateTime>,
    /// The courier's parcel number; `None` when the label printed none.
    tracking_id: Option<TrackingId>,
    /// The first page's whole text, for the label's courier clues.
    text: String,
    /// The header-band images of the order's pages, deduplicated by byte hash (Q-LABEL-5).
    logos: Vec<LogoImage>,
}

/// The `file_name` of the batch reader: the page's file by its index in the batch, or `file <n>`.
pub(super) fn file_name(file_names: &[&str], file: usize) -> String {
    match file_names.get(file) {
        Some(name) => (*name).to_string(),
        None => format!("file {file}"),
    }
}

/// Count one page of an unseen logo and the order it belongs to (first-seen order kept).
fn record_unseen(unseen: &mut Vec<UnseenLogo>, image: &LogoImage, order_id: &OrderId) {
    match unseen.iter_mut().find(|entry| entry.hash == image.hash) {
        Some(entry) => {
            entry.pages += 1;
            if !entry.order_ids.iter().any(|id| id == order_id) {
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
/// names the files for the messages; `table` is the data folder's courier data. Pages are
/// consumed one at a time, so a 200-page file can be read with progress.
pub fn read_batch<P>(
    pages: P,
    file_names: &[&str],
    table: &CourierTable,
) -> Result<Reading, ReadError>
where
    P: IntoIterator<Item = PageItem>,
{
    let mut groups: Vec<Group> = Vec::new();
    let mut continuation_pages: Vec<PageRef> = Vec::new();
    let mut no_slip: Vec<(usize, usize)> = Vec::new();
    let mut unseen: Vec<UnseenLogo> = Vec::new();
    let mut total_pages = 0usize;
    let mut current_file: Option<usize> = None;
    let mut previous: Option<OrderId> = None;

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
                groups.push(Group {
                    order_id: order_id.clone(),
                    pages: Vec::new(),
                    has_slip: false,
                    lines: Vec::new(),
                    qty_total: None,
                    customer_message: String::new(),
                    ship_by: None,
                    tracking_id,
                    text: page_text.text.clone(),
                    logos: Vec::new(),
                });
                groups.len() - 1
            }
        };

        // The page's header-band images (Q-LABEL-5): the order's logo clue, and the logos the
        // prompt must ask the owner about (D-LABEL-4).
        let mut seen_on_page: Vec<&str> = Vec::new();
        for image in &page_text.logos {
            if !groups[index]
                .logos
                .iter()
                .any(|seen| seen.hash == image.hash)
            {
                groups[index].logos.push(image.clone());
            }
            if seen_on_page.contains(&image.hash.as_str()) {
                continue;
            }
            seen_on_page.push(&image.hash);
            if label::is_unseen_logo(table, image) {
                record_unseen(&mut unseen, image, &order_id);
            }
        }

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
        let courier =
            label::deduce_courier_in(table, &group.text, group.tracking_id.as_ref(), &group.logos);
        let order = Order {
            order_id: group.order_id,
            tracking_id: group.tracking_id,
            courier,
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

    if let Some(unknown) = Warning::unknown_courier(&orders) {
        warnings.push(unknown);
    }
    let missing: Vec<OrderId> = orders
        .iter()
        .filter(|order| order.order.tracking_id.is_none())
        .map(|order| order.order.order_id.clone())
        .collect();
    if !missing.is_empty() {
        warnings.push(Warning::TrackingIdMissing { order_ids: missing });
    }

    Ok(Reading {
        orders,
        continuation_pages,
        unseen_logos: unseen,
        warnings,
    })
}
