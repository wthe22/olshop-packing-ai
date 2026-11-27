//! `orders.csv`: one row per order written by a live batch, the field names as headers, an
//! absent value as an empty cell. The last column's header is `schema:X.Y` and its cells are empty:
//! a reader finds the columns by their headers and passes over that one.

use packing_engine::record::Weight;

use super::{ContentIds, ORDERS_CSV_SCHEMA};
use crate::fold::Fold;

const HEADER: [&str; 16] = [
    "pdf_num",
    "page_start",
    "page_end",
    "order_id",
    "tracking_id",
    "courier_code",
    "shop",
    "province",
    "city",
    "district",
    "weight_grams",
    "content_id",
    "total_quantity",
    "ship_by_date",
    "batch_id",
    "group_code",
];

pub(super) fn text(fold: &Fold, ids: &ContentIds) -> String {
    let mut out = format!("{},schema:{ORDERS_CSV_SCHEMA}\n", HEADER.join(","));
    for (batch, live) in &fold.live {
        for order in &live.saved.orders {
            let area = order.area.clone().unwrap_or_default();
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
                order.shop.clone().unwrap_or_default(),
                area.province.unwrap_or_default(),
                area.city.unwrap_or_default(),
                area.district.unwrap_or_default(),
                match order.weight {
                    Some(Weight::Grams(grams)) => grams.to_string(),
                    _ => String::new(),
                },
                ids.id(*batch, order.content).to_string(),
                order
                    .total_quantity
                    .map(|n| n.to_string())
                    .unwrap_or_default(),
                order
                    .ship_by_date
                    .map(|d| d.to_string())
                    .unwrap_or_default(),
                batch.to_string(),
                order.group.clone(),
            ];
            let row: Vec<String> = cells.iter().map(|cell| quoted(cell)).collect();
            out.push_str(&row.join(","));
            // The schema column's empty cell.
            out.push_str(",\n");
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
