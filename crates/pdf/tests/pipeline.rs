//! The v0.2 chain on the repeat fixture, read through PDFium: pages → copies → orders (one copy
//! each, the rest left out) → plan → the `batch_saved` record, against `expected-repeat.json`.

mod common;

use std::collections::BTreeMap;
use std::sync::atomic::AtomicBool;

use packing_engine::batch_record::{BatchFacts, batch_saved, saved_orders};
use packing_engine::copies::read_copies;
use packing_engine::ids::{BatchId, PdfId};
use packing_engine::label::courier_data;
use packing_engine::record::{Datetime, InputPdf, LeftOutReason, Record, RecordBody, Sha256};
use packing_engine::repeats::{Choices, sort_out};
use packing_engine::sorting::{plan_groups, sort_batch};
use packing_engine::workflows::{BUNDLED_WORKFLOWS, load_workflows};
use packing_pdf::read;
use serde_json::Value;
use testkit::testdata;

fn input(pdf: u8, filename: &str, pages: u16) -> InputPdf {
    let time = Datetime::parse("2026-10-08T07:00:00+07:00").unwrap();
    InputPdf {
        pdf: PdfId::new(pdf),
        sha256: Sha256::try_from(char::from(b'a' + pdf).to_string().repeat(64)).unwrap(),
        filename: filename.into(),
        size: 1,
        page_count: pages,
        create_time: time.clone(),
        modify_time: time,
    }
}

#[test]
fn the_repeat_fixture_reads_plans_and_records_as_its_answer() {
    let Some(pdfium) = common::bind() else {
        eprintln!("skipped: run tools/get-pdfium.sh");
        return;
    };
    let answer: Value =
        serde_json::from_str(&std::fs::read_to_string(testdata("expected-repeat.json")).unwrap())
            .unwrap();
    let names: Vec<&str> = answer["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|name| name.as_str().unwrap())
        .collect();
    let paths: Vec<_> = names.iter().map(|name| testdata(name)).collect();
    let pages = read::read_all(
        &pdfium,
        &paths,
        &AtomicBool::new(false),
        courier_data(),
        &mut |_, _| {},
    )
    .expect("read the repeat fixture");
    let workflows = load_workflows(BUNDLED_WORKFLOWS).unwrap();

    let copies = read_copies(pages, &names, &workflows)
        .expect("copies")
        .copies;
    let got: Vec<Value> = copies
        .iter()
        .map(|copy| {
            serde_json::json!({
                "order_id": copy.order_id.as_str(),
                "input": copy.input,
                "page_start": copy.page_start,
                "page_end": copy.page_end,
                "lines": copy.lines.iter().map(|l| serde_json::json!([l.name, l.variation, l.seller_sku, l.quantity])).collect::<Vec<_>>(),
            })
        })
        .collect();
    assert_eq!(Value::Array(got), answer["copies"], "the copies");

    let orders = sort_out(&copies, &BTreeMap::new(), &Choices::default()).unwrap();
    let kept: Vec<u64> = orders.kept.iter().map(|&at| at as u64).collect();
    let want_kept: Vec<u64> = answer["kept"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_u64().unwrap())
        .collect();
    assert_eq!(kept, want_kept, "keep the first copies");
    assert_eq!(
        orders.left_out.len(),
        answer["left_out"].as_array().unwrap().len()
    );
    assert_eq!(
        orders.repeat_counts(&copies),
        (2, 2, 3),
        "2 repeats, 2 extra copies, 3 pages"
    );
    assert_eq!(orders.groups.len(), 1, "one order joins the two inputs");

    let rule = workflows.sort_rule("default").unwrap();
    let plan = plan_groups(rule);
    let sorted = sort_batch(
        &copies,
        &orders.kept,
        &plan,
        BatchId::new(1),
        &workflows.couriers,
    );
    let inputs = [input(1, names[0], 6), input(2, names[1], 2)];
    let saved = batch_saved(BatchFacts {
        batch: BatchId::new(1),
        amends: false,
        inputs: &inputs,
        rule_name: &rule.name,
        plan: &plan,
        copies: &copies,
        orders: &orders,
        sorted: &sorted,
        outputs: Vec::new(),
    });

    let order_ids: Vec<&str> = saved.orders.iter().map(|o| o.order_id.as_str()).collect();
    assert_eq!(order_ids.len(), 4);
    assert!(saved.orders.iter().all(|order| order.group == "Z"));
    assert_eq!(saved.contents.len(), 4, "four distinct contents");
    assert_eq!(saved.sort_rule.name, "default");
    let repeat = saved
        .orders
        .iter()
        .find(|order| order.order_id.as_str() == "580000000000000102")
        .unwrap();
    assert_eq!(
        (repeat.pdf, repeat.page_start, repeat.page_end),
        (PdfId::new(1), 2, 3)
    );
    let left: Vec<(&str, u8, u16, LeftOutReason)> = saved
        .left_out
        .iter()
        .map(|copy| {
            (
                copy.order_id.as_str(),
                copy.pdf.get(),
                copy.page_start,
                copy.reason.clone(),
            )
        })
        .collect();
    assert_eq!(
        left,
        [
            (
                "580000000000000102",
                1,
                4,
                LeftOutReason::Repeat {
                    pdf: PdfId::new(1),
                    page_start: 2
                }
            ),
            (
                "580000000000000103",
                2,
                1,
                LeftOutReason::Repeat {
                    pdf: PdfId::new(1),
                    page_start: 6
                }
            ),
        ]
    );
    let kinds: Vec<(&str, &str)> = saved
        .warnings
        .iter()
        .map(|f| (f.order_id.as_ref().unwrap().as_str(), f.kind.as_str()))
        .collect();
    assert_eq!(
        kinds,
        [("580000000000000103", "courier_unknown")],
        "SiCepat's form alone names nothing"
    );

    // The record reads back as written, and its orders are the already-saved set of a next batch.
    let line = Record {
        seq: 2,
        time: Datetime::parse("2026-10-08T09:00:00+07:00").unwrap(),
        app: "0.2.0".into(),
        body: RecordBody::BatchSaved(saved.clone()),
    };
    let json = serde_json::to_string(&line).unwrap();
    assert_eq!(serde_json::from_str::<Record>(&json).unwrap(), line);
    let guard = saved_orders([&saved]);
    assert_eq!(guard.len(), 4);
    let again = sort_out(&copies, &guard, &Choices::default()).unwrap();
    assert!(
        again.kept.is_empty(),
        "a second save of the same files finds every order saved"
    );
    assert_eq!(again.left_out.len(), copies.len());
}
