//! The v0.2 reading of the committed fixture: `labels-slip.pdf` read into copies gives
//! `expected-labels.json` — one copy per order, its pages, tracking ID, deadline date, lines and
//! printed total — each courier with the clue the label offers, and its sender's name; the repeat
//! inputs ask about the shop no account lists, as their answer has it; `labels-plain.pdf` stops.

mod common;

use std::sync::atomic::AtomicBool;

use packing_engine::account_questions::{
    AccountChoices, ShopQuestion, account_questions, assign_accounts,
};

use packing_engine::copies::{CopiesStop, read_copies};
use packing_engine::record::CourierClueKind;

use packing_engine::workflows::load_bundled;
use packing_pdf::read;
use serde_json::Value;
use testkit::{shop_accounts, testdata};

#[test]
fn the_fixture_reads_into_one_copy_per_order_as_its_answer() {
    let Some(pdfium) = common::bind() else {
        eprintln!("skipped: run tools/get-pdfium.sh");
        return;
    };
    let workflows = load_bundled();
    let path = testdata("labels-slip.pdf");
    let pages = read::read_all(
        &pdfium,
        std::slice::from_ref(&path),
        &AtomicBool::new(false),
        &workflows,
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
        match (&copy.sender, want["sender"].as_str()) {
            (Some(sender), Some(name)) => {
                assert_eq!(sender.name, name, "{id}: sender");
                assert!(sender.text.shows(name), "{id}: sender");
            }
            (sender, name) => assert_eq!(sender.is_some(), name.is_some(), "{id}: sender"),
        }
        assert!(
            copy.sender
                .as_ref()
                .is_none_or(|s| !s.text.shows("Toko Lain")),
            "{id}"
        );
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
    let workflows = load_bundled();
    let path = testdata("labels-plain.pdf");
    let pages = read::read_all(
        &pdfium,
        std::slice::from_ref(&path),
        &AtomicBool::new(false),
        &workflows,
        &mut |_, _| {},
    )
    .expect("read labels-plain.pdf");
    match read_copies(pages, &["labels-plain.pdf"], &workflows) {
        Err(CopiesStop::NoSlip(no_slip)) => assert_eq!(no_slip.plain_pages, 3),
        other => panic!("expected the no-slip stop, got {other:?}"),
    }
}

#[test]
fn the_repeat_inputs_ask_about_the_shop_no_account_lists() {
    let Some(pdfium) = common::bind() else {
        eprintln!("skipped: run tools/get-pdfium.sh");
        return;
    };
    let workflows = load_bundled();
    let names = ["labels-repeat-1.pdf", "labels-repeat-2.pdf"];
    let paths: Vec<_> = names.iter().map(|name| testdata(name)).collect();
    let pages = read::read_all(
        &pdfium,
        &paths,
        &AtomicBool::new(false),
        &workflows,
        &mut |_, _| {},
    )
    .expect("read the repeat inputs");
    let mut copies = read_copies(pages, &names, &workflows)
        .expect("copies")
        .copies;
    let answer: Value =
        serde_json::from_str(&std::fs::read_to_string(testdata("expected-repeat.json")).unwrap())
            .unwrap();
    let accounts = shop_accounts(&["Toko Contoh"]);
    assign_accounts(&mut copies, &accounts, &AccountChoices::default());

    let other: Vec<&Value> = answer["copies"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|copy| copy["sender"] == "Toko Lain")
        .collect();
    let mut inputs: Vec<usize> = other
        .iter()
        .map(|copy| copy["input"].as_u64().unwrap() as usize)
        .collect();
    inputs.dedup();
    let want = ShopQuestion {
        name: "Toko Lain".into(),
        orders: other.len(),
        inputs,
    };
    let questions = account_questions(&copies);
    assert_eq!(questions.shops, [want]);
    assert!(questions.no_sender.is_empty());
}
