//! `packing-cli dump`: what reading the label files gives, as JSON, with nothing saved — the
//! local baseline of the owner's sample files is made of it. Read as a first batch of an empty
//! day, with the courier list given or else the built-in one, so the same files always give the
//! same text.

use packing_engine::orders::ReadOrder;
use packing_pdf::batch::Draft;
use serde_json::{Value, json};

/// The draft as pretty JSON: the files, the counts, every order and the read warnings.
pub fn draft_json(draft: &Draft) -> String {
    let files: Vec<Value> = draft
        .files
        .iter()
        .map(|file| json!({ "name": file.name, "pages": file.pages }))
        .collect();
    let orders: Vec<Value> = draft.orders.iter().map(order_json).collect();
    let unseen_logos: Vec<&str> = draft
        .unseen_logos
        .iter()
        .map(|logo| logo.hash.as_str())
        .collect();
    let value = json!({
        "files": files,
        "pages": draft.pages,
        "slip_pages": draft.slip_pages,
        "total_orders": draft.total_orders,
        "orders": orders,
        "unseen_logos": unseen_logos,
        "warnings": draft.warnings,
    });
    let mut text = serde_json::to_string_pretty(&value).expect("JSON of plain values");
    text.push('\n');
    text
}

fn order_json(read: &ReadOrder) -> Value {
    let order = &read.order;
    let pages: Vec<[usize; 2]> = read.pages.iter().map(|at| [at.file, at.page]).collect();
    let lines: Vec<Value> = order
        .lines
        .iter()
        .map(|line| json!([line.name, line.variation, line.seller_sku, line.quantity]))
        .collect();
    json!({
        "order_id": order.order_id.as_str(),
        "pages": pages,
        "tracking_id": order.tracking_id.as_ref().map(|id| id.as_str()),
        "courier": order.courier,
        "ship_by": order.ship_by.map(|time| time.to_string()),
        "has_slip": read.has_slip,
        "lines": lines,
        "qty_total": read.qty_total,
        "customer_message": order.customer_message,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use packing_engine::day::DayState;
    use packing_engine::label::courier_data;
    use packing_pdf::worker::PdfWorker;
    use serde_json::Value;
    use testkit::{testdata, vendor_dir};

    use super::draft_json;
    use crate::read_draft;

    /// Every order of the fixture with the lines, total, message and page count of its answer,
    /// and the wrong printed total warned.
    #[test]
    fn the_fixture_dumps_as_its_answer() {
        let Some(vendor) = vendor_dir() else {
            eprintln!("skipped: pdfium.dll not downloaded (tools/get-pdfium.sh)");
            return;
        };
        let _guard = testkit::pdfium_lock();
        let worker = PdfWorker::spawn(vendor).expect("spawn worker");
        let files = [testdata("labels-slip.pdf")];
        let draft = read_draft(&worker, courier_data(), &DayState::new(""), &files).expect("read");
        let dump: Value = serde_json::from_str(&draft_json(&draft)).expect("JSON");
        let answer: Value = serde_json::from_str(
            &fs::read_to_string(testdata("expected-labels.json")).expect("read the answer"),
        )
        .expect("parse the answer");

        assert_eq!(dump["files"][0]["pages"], 24);
        let orders = dump["orders"].as_array().expect("orders");
        assert_eq!(orders.len(), 23);
        for order in orders {
            let id = order["order_id"].as_str().expect("order_id");
            let want = &answer[id];
            for field in ["lines", "qty_total", "customer_message", "tracking_id"] {
                assert_eq!(order[field], want[field], "{id}: {field}");
            }
            assert_eq!(
                order["pages"].as_array().unwrap().len(),
                want["pages"],
                "{id}"
            );
        }
        let warnings = dump["warnings"].as_array().expect("warnings");
        assert!(
            warnings
                .iter()
                .any(|warning| warning.as_str().unwrap().contains("JY0000000901")),
            "the wrong printed total is warned: {warnings:?}"
        );
    }
}
