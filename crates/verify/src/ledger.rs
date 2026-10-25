//! V3, the ledger — every page of every input in exactly one written or left-out copy — and V4,
//! the orders: unique in the record and in the date's other live batches, each left-out copy
//! naming what it was left out for.

use std::collections::{BTreeMap, BTreeSet};

use crate::Findings;
use crate::record::{Batch, Kept};

/// The input's file name in a message, by its `PdfID`.
pub(crate) fn input_name(batch: &Batch, pdf: u8) -> String {
    batch
        .inputs
        .iter()
        .find(|input| input.pdf == pdf)
        .map_or_else(|| format!("input {pdf}"), |input| input.filename.clone())
}

pub(crate) fn check_ledger(batch: &Batch, findings: &mut Findings) {
    let mut covered: BTreeMap<(u8, u16), Vec<&str>> = BTreeMap::new();
    let copies = batch
        .orders
        .iter()
        .map(|o| (o.pdf, o.page_start, o.page_end, o.order_id.as_str()))
        .chain(
            batch
                .left_out
                .iter()
                .map(|l| (l.pdf, l.page_start, l.page_end, l.order_id.as_str())),
        );
    for (pdf, start, end, order) in copies {
        if !batch.inputs.iter().any(|input| input.pdf == pdf) {
            findings.error(format!(
                "order {order} names input {pdf}, which the batch did not read"
            ));
            continue;
        }
        for page in start..=end {
            covered.entry((pdf, page)).or_default().push(order);
        }
    }
    for input in &batch.inputs {
        for page in 1..=input.page_count {
            match covered.get(&(input.pdf, page)).map(Vec::as_slice) {
                None | Some([]) => findings.error(format!(
                    "page {page} of {} is in no copy: it is neither written nor left out",
                    input.filename
                )),
                Some([_]) => {}
                Some(orders) => findings.error(format!(
                    "page {page} of {} is in {} copies (orders {})",
                    input.filename,
                    orders.len(),
                    orders.join(", ")
                )),
            }
        }
        let beyond = covered
            .keys()
            .filter(|(pdf, page)| *pdf == input.pdf && *page > input.page_count)
            .count();
        if beyond > 0 {
            findings.error(format!(
                "{} pages past the end of {} are in a copy",
                beyond, input.filename
            ));
        }
    }
}

pub(crate) fn check_orders(batch: &Batch, live: &BTreeMap<u8, Batch>, findings: &mut Findings) {
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    for order in &batch.orders {
        if !seen.insert(&order.order_id) {
            findings.error(format!("order {} is written twice", order.order_id));
        }
        for (number, other) in live {
            if other.orders.iter().any(|o| o.order_id == order.order_id) {
                findings.error(format!(
                    "order {} is saved in batch {number} already",
                    order.order_id
                ));
            }
        }
    }
    for left in &batch.left_out {
        match (left.reason.as_str(), &left.kept) {
            ("repeat", Kept::Copy { pdf, page_start }) => {
                let written = batch.orders.iter().any(|order| {
                    order.order_id == left.order_id
                        && order.pdf == *pdf
                        && order.page_start == *page_start
                });
                if !written {
                    findings.error(format!(
                        "order {} is left out for a copy that is not written (page {page_start} of {})",
                        left.order_id,
                        input_name(batch, *pdf)
                    ));
                }
            }
            ("saved_earlier", Kept::Batch { batch: earlier }) => {
                let holds = live
                    .get(earlier)
                    .is_some_and(|other| other.orders.iter().any(|o| o.order_id == left.order_id));
                if !holds {
                    findings.error(format!(
                        "order {} is left out as saved in batch {earlier}, which does not hold it",
                        left.order_id
                    ));
                }
            }
            (reason, _) => findings.error(format!(
                "order {} is left out for {reason:?} with what it kept not matching",
                left.order_id
            )),
        }
    }
}
