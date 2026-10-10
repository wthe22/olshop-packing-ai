//! Reading `testdata/labels-slip.pdf` and comparing with `testdata/expected-labels.json`
//! (07 › *Testing*: the same results as the Python script), plus the plain-label stop.

mod common;

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

use packing_engine::orders::{ReadError, ReadOrder, read_batch};
use packing_pdf::read;

use serde_json::Value;

fn file_names(paths: &[PathBuf]) -> Vec<String> {
    paths
        .iter()
        .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
        .collect()
}

fn compare(order: &ReadOrder, expected: &Value) {
    let id = &order.order.order_id;
    assert_eq!(
        order.pages.len() as u64,
        expected["pages"].as_u64().unwrap(),
        "{id}: pages"
    );
    assert_eq!(
        order.order.tracking_id,
        expected["tracking_id"].as_str().unwrap(),
        "{id}: tracking_id"
    );
    assert_eq!(
        order.order.courier,
        expected["courier"].as_str().unwrap(),
        "{id}: courier"
    );
    let ship_by = order.order.ship_by.map(|dt| dt.to_string());
    match expected["ship_by"].as_str() {
        Some(want) => assert_eq!(ship_by.as_deref(), Some(want), "{id}: ship_by"),
        None => assert_eq!(ship_by, None, "{id}: ship_by should be absent"),
    }
    assert_eq!(
        order.has_slip,
        expected["has_slip"].as_bool().unwrap(),
        "{id}: has_slip"
    );
    assert_eq!(
        order.qty_total,
        expected["qty_total"].as_i64(),
        "{id}: qty_total"
    );
    assert_eq!(
        order.order.customer_message,
        expected["customer_message"].as_str().unwrap(),
        "{id}: customer_message"
    );
    let lines = expected["lines"].as_array().unwrap();
    assert_eq!(order.order.lines.len(), lines.len(), "{id}: line count");
    for (line, want) in order.order.lines.iter().zip(lines) {
        let want = want.as_array().unwrap();
        assert_eq!(line.name, want[0].as_str().unwrap(), "{id}: line name");
        assert_eq!(
            line.variation,
            want[1].as_str().unwrap(),
            "{id}: line variation"
        );
        assert_eq!(
            line.seller_sku,
            want[2].as_str().unwrap(),
            "{id}: line seller_sku"
        );
        assert_eq!(
            line.quantity,
            want[3].as_i64().unwrap(),
            "{id}: line quantity"
        );
    }
}

#[test]
fn reads_labels_slip_and_matches_the_fixture() {
    let Some(pdfium) = common::bind() else {
        eprintln!("skipped: run pc/tools/get-pdfium.sh");
        return;
    };
    let path = common::testdata("labels-slip.pdf");
    let cancel = AtomicBool::new(false);
    let mut progress = Vec::new();
    let start = std::time::Instant::now();
    let pages = read::read_all(
        &pdfium,
        std::slice::from_ref(&path),
        &cancel,
        &mut |done, total| progress.push((done, total)),
    )
    .expect("read labels-slip.pdf");
    let elapsed = start.elapsed();
    eprintln!(
        "read labels-slip.pdf: {} pages in {:.3} s",
        pages.len(),
        elapsed.as_secs_f64()
    );

    assert_eq!(pages.len(), 24);
    // Progress runs 1..=total and ends at (total, total).
    assert_eq!(progress.first(), Some(&(1, 24)));
    assert_eq!(progress.last(), Some(&(24, 24)));

    let names = file_names(std::slice::from_ref(&path));
    let name_refs: Vec<&str> = names.iter().map(String::as_str).collect();
    let reading = read_batch(pages, &name_refs).expect("read the batch");

    let expected: Value = serde_json::from_str(
        &std::fs::read_to_string(common::testdata("expected-labels.json")).expect("fixture"),
    )
    .expect("parse fixture");

    let mut matched = 0;
    for order in &reading.orders {
        let entry = expected.get(&order.order.order_id).unwrap_or_else(|| {
            panic!(
                "order {} is not in expected-labels.json",
                order.order.order_id
            )
        });
        compare(order, entry);
        matched += 1;
    }
    assert_eq!(matched, 23, "every order must match");
    assert_eq!(reading.orders.len(), 23);
    eprintln!("orders matched: {matched}/23");
}

#[test]
fn plain_labels_stop_with_the_no_slip_error() {
    let Some(pdfium) = common::bind() else {
        eprintln!("skipped: run pc/tools/get-pdfium.sh");
        return;
    };
    let path = common::testdata("labels-plain.pdf");
    let cancel = AtomicBool::new(false);
    let pages = read::read_all(
        &pdfium,
        std::slice::from_ref(&path),
        &cancel,
        &mut |_, _| {},
    )
    .expect("read labels-plain.pdf");
    let names = file_names(std::slice::from_ref(&path));
    let name_refs: Vec<&str> = names.iter().map(String::as_str).collect();
    let error = read_batch(pages, &name_refs).expect_err("plain labels must stop");
    match error {
        ReadError::NoSlip(no_slip) => {
            assert_eq!(no_slip.plain_pages, 3);
            assert_eq!(no_slip.total_pages, 3);
        }
        other => panic!("expected the no-slip stop, got {other:?}"),
    }
}
