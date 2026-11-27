//! A batch as the fold reads it, whichever version of the app saved it. Journal 2.0 gave an
//! order its pickup code as its tracking ID when its label printed none, and kept the Seller SKUs
//! beside the order rather than in its content; both are made as 2.1 has them here. J&T Express was
//! `jnt` until v0.6 and is `jnt-express` since.

use packing_engine::ids::{CourierCode, TrackingId};
use packing_engine::record::{BatchSaved, ItemPick, OrderContent};

const OLD_JNT: &str = "jnt";
const JNT_EXPRESS: &str = "jnt-express";

/// The batch as the fold reads it.
pub(crate) fn as_read(saved: &BatchSaved) -> BatchSaved {
    let mut saved = saved.clone();
    for order in &mut saved.orders {
        if order
            .courier
            .as_ref()
            .is_some_and(|code| code.as_str() == OLD_JNT)
        {
            order.courier = Some(CourierCode::new(JNT_EXPRESS));
        }
        if order.tracking_id.is_none() {
            order.tracking_id = order.pickup_code.as_deref().map(TrackingId::new);
        }
    }
    with_skus_in_contents(&mut saved);
    saved
}

/// Each order's SKUs go into the items of its content, and the contents are made again: two orders
/// whose items are the same but whose SKUs differ are two contents. A batch of 2.1 has no SKUs beside
/// its orders, so it is left as it is.
fn with_skus_in_contents(saved: &mut BatchSaved) {
    let damaged = saved
        .orders
        .iter()
        .any(|order| usize::from(order.content) >= saved.contents.len());
    if damaged || saved.orders.iter().all(|order| order.skus.is_empty()) {
        return;
    }
    let mut contents: Vec<OrderContent> = Vec::new();
    for order in &mut saved.orders {
        let old = &saved.contents[usize::from(order.content)];
        let items = old
            .items
            .iter()
            .enumerate()
            .map(|(at, item)| ItemPick {
                sku: order.skus.get(at).cloned().flatten(),
                ..item.clone()
            })
            .collect();
        let content = OrderContent { items };
        order.content = match contents.iter().position(|known| *known == content) {
            Some(index) => index,
            None => {
                contents.push(content);
                contents.len() - 1
            }
        } as u16;
        order.skus.clear();
    }
    saved.contents = contents;
}
