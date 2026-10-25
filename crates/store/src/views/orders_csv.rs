//! `orders.csv`: one row per order written by a live batch, the field names as headers, an
//! absent value as an empty cell.

use packing_engine::record::CourierClueKind;

use super::ContentIds;
use crate::fold::Fold;

const HEADER: [&str; 14] = [
    "pdf_num",
    "page_start",
    "page_end",
    "order_id",
    "tracking_id",
    "courier_code",
    "courier_clue",
    "content_id",
    "skus",
    "total_quantity",
    "ship_by_date",
    "pickup_code",
    "batch_id",
    "group_code",
];

pub(super) fn text(fold: &Fold, ids: &ContentIds) -> String {
    let mut out = HEADER.join(",");
    out.push('\n');
    for (batch, live) in &fold.live {
        for order in &live.saved.orders {
            let clue = match order.courier_clue.kind {
                CourierClueKind::None => "",
                kind => kind.as_str(),
            };
            let skus: Vec<&str> = order
                .skus
                .iter()
                .map(|sku| sku.as_deref().unwrap_or(""))
                .collect();
            let cells = [
                order.pdf.to_string(),
                order.page_start.to_string(),
                order.page_end.to_string(),
                order.order_id.to_string(),
                order
                    .tracking_id
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
                order
                    .courier
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
                clue.to_string(),
                ids.id(*batch, order.content).to_string(),
                skus.join("; "),
                order
                    .total_quantity
                    .map(|n| n.to_string())
                    .unwrap_or_default(),
                order
                    .ship_by_date
                    .map(|d| d.to_string())
                    .unwrap_or_default(),
                order.pickup_code.clone().unwrap_or_default(),
                batch.to_string(),
                order.group.clone(),
            ];
            let row: Vec<String> = cells.iter().map(|cell| quoted(cell)).collect();
            out.push_str(&row.join(","));
            out.push('\n');
        }
    }
    out
}

/// A cell as CSV: quoted, with its quotes doubled, when it holds a comma, a quote or a line break.
fn quoted(cell: &str) -> String {
    if cell.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", cell.replace('"', "\"\""))
    } else {
        cell.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::quoted;

    #[test]
    fn a_cell_is_quoted_only_when_it_must_be() {
        let cases = [
            ("JY0000001234", "JY0000001234"),
            ("", ""),
            ("A, B", "\"A, B\""),
            ("say \"hi\"", "\"say \"\"hi\"\"\""),
            ("two\nlines", "\"two\nlines\""),
        ];
        for (cell, want) in cases {
            assert_eq!(quoted(cell), want, "{cell:?}");
        }
    }
}
