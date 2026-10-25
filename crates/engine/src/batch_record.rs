//! A batch's `batch_saved` record (records.md) from what the read and the plan decided: its
//! inputs, the plan as used, the distinct contents, the orders in printing order, the copies left
//! out and the read's findings. The outputs come from whoever wrote them.

use crate::copies::Copy;
use crate::ids::{BatchId, OrderId};
use crate::record::{
    BatchSaved, Finding, FindingKind, InputPdf, LeftOutCopy, LeftOutReason, OrderContent,
    OutputFile, SavedOrder,
};
use crate::repeats::{LeftOutBecause, Orders};
use crate::sorting::{PlanGroup, SortedBatch, content_of, plan_snapshot};

/// What a batch's record is built from.
pub struct BatchFacts<'a> {
    pub batch: BatchId,
    pub amends: bool,
    /// In reading order: a copy's `input` is its index here.
    pub inputs: &'a [InputPdf],
    pub rule_name: &'a str,
    pub plan: &'a [PlanGroup],
    pub copies: &'a [Copy],
    pub orders: &'a Orders,
    pub sorted: &'a SortedBatch,
    /// The group PDFs in group order, then the summary PDF.
    pub outputs: Vec<OutputFile>,
}

pub fn batch_saved(facts: BatchFacts<'_>) -> BatchSaved {
    let mut contents: Vec<OrderContent> = Vec::new();
    let mut orders = Vec::new();
    for (group, at) in facts.sorted.printing_order() {
        let copy = &facts.copies[at];
        let (content, skus) = content_of(copy);
        let index = match contents.iter().position(|known| *known == content) {
            Some(index) => index,
            None => {
                contents.push(content);
                contents.len() - 1
            }
        };
        orders.push(SavedOrder {
            order_id: copy.order_id.clone(),
            tracking_id: copy.tracking_id.clone(),
            courier: copy.courier.clone(),
            courier_clue: copy.courier_clue.clone(),
            content: index as u16,
            skus,
            total_quantity: copy
                .total_quantity
                .map(|n| u16::try_from(n).unwrap_or(u16::MAX)),
            ship_by_date: copy.ship_by_date,
            pickup_code: copy.pickup_code.clone(),
            pdf: facts.inputs[copy.input].pdf,
            page_start: copy.page_start,
            page_end: copy.page_end,
            group: facts.sorted.groups[group].code.clone(),
        });
    }
    let left_out = facts
        .orders
        .left_out
        .iter()
        .map(|left| {
            let copy = &facts.copies[left.copy];
            let reason = match left.because {
                LeftOutBecause::Repeat { kept } => LeftOutReason::Repeat {
                    pdf: facts.inputs[facts.copies[kept].input].pdf,
                    page_start: facts.copies[kept].page_start,
                },
                LeftOutBecause::SavedEarlier { batch } => LeftOutReason::SavedEarlier { batch },
            };
            LeftOutCopy {
                order_id: copy.order_id.clone(),
                pdf: facts.inputs[copy.input].pdf,
                page_start: copy.page_start,
                page_end: copy.page_end,
                reason,
            }
        })
        .collect();
    BatchSaved {
        batch: facts.batch,
        amends: facts.amends,
        inputs: facts.inputs.to_vec(),
        sort_rule: plan_snapshot(facts.rule_name, facts.plan),
        contents,
        orders,
        left_out,
        outputs: facts.outputs,
        warnings: read_findings(facts.copies, &facts.orders.kept, facts.inputs),
        errors: Vec::new(),
    }
}

/// The read's warnings of the orders kept, one per order and kind, in reading order: a printed
/// total that differs from the lines, a courier not named, no tracking ID (an instant-service
/// label prints none and is not warned).
pub fn read_findings(copies: &[Copy], kept: &[usize], inputs: &[InputPdf]) -> Vec<Finding> {
    let mut findings = Vec::new();
    for &at in kept {
        let copy = &copies[at];
        let label = copy
            .tracking_id
            .as_ref()
            .map_or_else(|| copy.order_id.to_string(), ToString::to_string);
        let finding = |kind: &str, text: String| Finding {
            kind: FindingKind::new(kind),
            pdf: Some(inputs[copy.input].pdf),
            page: Some(copy.page_start),
            order_id: Some(copy.order_id.clone()),
            text,
        };
        if let Some((printed, sum)) = copy.total_differs() {
            findings.push(finding(
                "slip_total_differs",
                format!(
                    "Slip total differs for {label}: printed {printed}, lines add up to {sum}. \
                     Check that order's slip when packing."
                ),
            ));
        }
        if copy.courier.is_none() {
            findings.push(finding(
                "courier_unknown",
                format!(
                    "Courier unknown for {label}. It is sorted normally; only a filter on \
                     courier cannot see it."
                ),
            ));
        }
        if copy.tracking_id.is_none() && copy.pickup_code.is_none() {
            findings.push(finding(
                "tracking_id_missing",
                format!(
                    "No tracking ID read for order {}. Its label is saved as usual; only a filter \
                     on tracking_id cannot see it.",
                    copy.order_id
                ),
            ));
        }
    }
    findings
}

/// The orders a fold's live batches saved, for the already-saved guard.
pub fn saved_orders<'a>(
    batches: impl IntoIterator<Item = &'a BatchSaved>,
) -> std::collections::BTreeMap<OrderId, BatchId> {
    batches
        .into_iter()
        .flat_map(|saved| {
            saved
                .orders
                .iter()
                .map(move |order| (order.order_id.clone(), saved.batch))
        })
        .collect()
}
