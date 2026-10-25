//! The v0.2 reading of the committed fixture: `labels-slip.pdf` read into copies gives
//! `expected-labels.json` — one copy per order, its pages, tracking ID, deadline date, lines and
//! printed total — and each courier with the clue the label offers; `labels-plain.pdf` stops.

mod common;

use std::sync::atomic::AtomicBool;

use packing_engine::copies::{CopiesStop, read_copies};
use packing_engine::label::courier_data;
use packing_engine::record::CourierClueKind;
use packing_engine::workflows::{BUNDLED_WORKFLOWS, load_workflows};
use packing_pdf::read;
use serde_json::Value;
use testkit::testdata;

#[test]
fn the_fixture_reads_into_one_copy_per_order_as_its_answer() {
    let Some(pdfium) = common::bind() else {
        eprintln!("skipped: run tools/get-pdfium.sh");
        return;
    };
    let workflows = load_workflows(BUNDLED_WORKFLOWS).unwrap();
    let path = testdata("labels-slip.pdf");
    let pages = read::read_all(
        &pdfium,
        std::slice::from_ref(&path),
        &AtomicBool::new(false),
        courier_data(),
        &mut |_, _| {},
    )
    .expect("read labels-slip.pdf");
    let read = read_copies(pages, &["labels-slip.pdf"], &workflows).expect("copies");
    let answer: Value =
        serde_json::from_str(&std::fs::read_to_string(testdata("expected-labels.json")).unwrap())
            .unwrap();

    assert_eq!(read.pages, 24);
    assert_eq!(read.copies.len(), 23);
    for copy in &read.copies {
        let id = copy.order_id.as_str();
        let want = &answer[id];
        let pages = (copy.page_end - copy.page_start + 1) as u64;
        assert_eq!(pages, want["pages"].as_u64().unwrap(), "{id}: pages");
        assert_eq!(
            copy.tracking_id.as_ref().map(|t| t.as_str()),
            want["tracking_id"].as_str(),
            "{id}"
        );
        let ship_by = want["ship_by"].as_str().map(|s| s[..10].to_string());
        assert_eq!(copy.ship_by_date.map(|d| d.to_string()), ship_by, "{id}");
        let lines: Vec<Value> = copy
            .lines
            .iter()
            .map(|l| serde_json::json!([l.name, l.variation, l.seller_sku, l.quantity]))
            .collect();
        assert_eq!(Value::Array(lines), want["lines"], "{id}: lines");
        assert_eq!(copy.total_quantity, want["qty_total"].as_i64(), "{id}");

        // The fixture prints no image: J&T is named by its printed site, IDX by its lettered
        // form, and SiCepat's bare-number form names nothing.
        let (code, kind) = match want["courier"].as_str().unwrap() {
            "J&T Express" => (Some("jnt"), CourierClueKind::Text),
            "IDX" => (Some("idx"), CourierClueKind::Form),
            _ => (None, CourierClueKind::None),
        };
        assert_eq!(copy.courier.as_ref().map(|c| c.as_str()), code, "{id}");
        assert_eq!(copy.courier_clue.kind, kind, "{id}");
    }
    let differs: Vec<&str> = read
        .copies
        .iter()
        .filter(|copy| copy.total_differs().is_some())
        .map(|copy| copy.order_id.as_str())
        .collect();
    assert_eq!(differs, ["580000000000000901"]);
}

#[test]
fn the_plain_fixture_stops_with_no_slip() {
    let Some(pdfium) = common::bind() else {
        eprintln!("skipped: run tools/get-pdfium.sh");
        return;
    };
    let workflows = load_workflows(BUNDLED_WORKFLOWS).unwrap();
    let path = testdata("labels-plain.pdf");
    let pages = read::read_all(
        &pdfium,
        std::slice::from_ref(&path),
        &AtomicBool::new(false),
        courier_data(),
        &mut |_, _| {},
    )
    .expect("read labels-plain.pdf");
    match read_copies(pages, &["labels-plain.pdf"], &workflows) {
        Err(CopiesStop::NoSlip(no_slip)) => assert_eq!(no_slip.plain_pages, 3),
        other => panic!("expected the no-slip stop, got {other:?}"),
    }
}
