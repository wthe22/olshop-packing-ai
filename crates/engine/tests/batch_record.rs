//! The `batch_saved` record (records.md) from a plan: the read's findings per order, the copies
//! left out as saved earlier, an amend, the distinct contents in printing order.

use std::collections::BTreeMap;

use packing_engine::batch_record::{BatchFacts, batch_saved, read_findings};
use packing_engine::ids::{BatchId, CourierCode, OrderId, PdfId, TrackingId};
use packing_engine::record::{Datetime, InputPdf, LeftOutReason, Sha256};
use packing_engine::repeats::{Choices, sort_out};
use packing_engine::sorting::{PlanGroup, sort_batch};
use testkit::{copy, slip_line};

fn input(pdf: u8) -> InputPdf {
    let time = Datetime::parse("2026-10-08T07:00:00+07:00").unwrap();
    InputPdf {
        pdf: PdfId::new(pdf),
        sha256: Sha256::try_from("c".repeat(64)).unwrap(),
        filename: format!("labels_{pdf}.pdf"),
        size: 1,
        page_count: 3,
        create_time: time.clone(),
        modify_time: time,
    }
}

#[test]
fn each_kept_order_is_warned_for_what_its_read_lacks() {
    let named = |mut c: packing_engine::copies::Copy| {
        c.courier = Some(CourierCode::new("jnt"));
        c.tracking_id = Some(TrackingId::new("JY0000000001"));
        c
    };
    let mut wrong_total = named(copy(
        "580000000000000001",
        0,
        1,
        vec![slip_line("A", "", "", 1)],
    ));
    wrong_total.total_quantity = Some(2);
    let mut no_courier = named(copy(
        "580000000000000002",
        0,
        2,
        vec![slip_line("A", "", "", 1)],
    ));
    no_courier.courier = None;
    let mut no_tracking = named(copy(
        "580000000000000003",
        0,
        3,
        vec![slip_line("A", "", "", 1)],
    ));
    no_tracking.tracking_id = None;
    let mut instant = named(copy(
        "580000000000000004",
        1,
        1,
        vec![slip_line("A", "", "", 1)],
    ));
    instant.tracking_id = None;
    instant.pickup_code = Some("WX1234".into());
    let fine = named(copy(
        "580000000000000005",
        1,
        2,
        vec![slip_line("A", "", "", 1)],
    ));
    let copies = [wrong_total, no_courier, no_tracking, instant, fine];

    let findings = read_findings(&copies, &[0, 1, 2, 3, 4], &[input(1), input(2)]);
    let found: Vec<(&str, &str, Option<u8>, Option<u16>)> = findings
        .iter()
        .map(|f| {
            (
                f.order_id.as_ref().unwrap().as_str(),
                f.kind.as_str(),
                f.pdf.map(PdfId::get),
                f.page,
            )
        })
        .collect();
    assert_eq!(
        found,
        [
            ("580000000000000001", "slip_total_differs", Some(1), Some(1)),
            ("580000000000000002", "courier_unknown", Some(1), Some(2)),
            (
                "580000000000000003",
                "tracking_id_missing",
                Some(1),
                Some(3)
            ),
        ],
        "an instant label prints no tracking ID and is not warned for it"
    );
    assert!(findings[0].text.contains("printed 2, lines add up to 1"));
    assert!(
        findings[1].text.contains("JY0000000001"),
        "named by its tracking ID"
    );
}

#[test]
fn an_amend_records_the_copies_saved_earlier_and_its_contents_once() {
    let copies = [
        copy(
            "580000000000000001",
            0,
            1,
            vec![slip_line("Sepatu", "", "S-1", 1)],
        ),
        copy(
            "580000000000000002",
            0,
            2,
            vec![slip_line("Sepatu", "", "S-2", 1)],
        ),
        copy(
            "580000000000000003",
            0,
            3,
            vec![slip_line("Jok", "", "", 1)],
        ),
    ];
    let saved = BTreeMap::from([(OrderId::new("580000000000000003"), BatchId::new(1))]);
    let orders = sort_out(&copies, &saved, &Choices::default()).unwrap();
    let plan = [PlanGroup {
        code: "Z".into(),
        name: "Lainnya".into(),
        filter: None,
        file_name: None,
    }];
    let sorted = sort_batch(&copies, &orders.kept, &plan, BatchId::new(2));
    let inputs = [input(1)];
    let record = batch_saved(BatchFacts {
        batch: BatchId::new(2),
        amends: true,
        inputs: &inputs,
        rule_name: "default",
        plan: &plan,
        copies: &copies,
        orders: &orders,
        sorted: &sorted,
        outputs: Vec::new(),
    });
    assert!(record.amends);
    assert_eq!(
        record.contents.len(),
        1,
        "two orders that differ only in Seller SKU"
    );
    let skus: Vec<Vec<Option<String>>> = record.orders.iter().map(|o| o.skus.clone()).collect();
    assert_eq!(
        skus,
        [vec![Some("S-1".to_string())], vec![Some("S-2".to_string())]]
    );
    assert_eq!(record.left_out.len(), 1);
    assert_eq!(
        record.left_out[0].reason,
        LeftOutReason::SavedEarlier {
            batch: BatchId::new(1)
        }
    );
    assert_eq!(record.sort_rule.label_groups[0].filter, None);
}
