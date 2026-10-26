//! A made-up `batch_saved` record and its parts, for the tests of the crates that store, write and
//! check one.

use jiff::civil::date;
use packing_engine::ids::{BatchId, CourierCode, OrderId, PdfId, TrackingId};
use packing_engine::record::{
    BatchSaved, CourierClue, CourierClueKind, Datetime, Finding, FindingKind, InputPdf, ItemPick,
    LabelGroupRule, LabelSortRule, LeftOutCopy, LeftOutReason, OrderContent, OutputFile,
    SavedOrder, Sha256,
};

pub fn time(text: &str) -> Datetime {
    Datetime::parse(text).expect("a test time")
}

pub fn sha(letter: char) -> Sha256 {
    Sha256::try_from(letter.to_string().repeat(64)).expect("hex")
}

fn saved_order(id: &str, content: u16, group: &str, page: u16) -> SavedOrder {
    SavedOrder {
        order_id: OrderId::new(id),
        tracking_id: Some(TrackingId::new(format!("JY{}", &id[8..]))),
        courier: Some(CourierCode::new("jnt")),
        courier_clue: CourierClue {
            kind: CourierClueKind::Logo,
            value: Some("39b9b7b9f61d0e7f".into()),
            form: Some(r"J[XY]\d{10}".into()),
        },
        content,
        skus: vec![Some("SELL-SEP-01".into())],
        total_quantity: Some(1),
        ship_by_date: Some(date(2026, 10, 8)),
        pickup_code: None,
        pdf: PdfId::new(1),
        page_start: page,
        page_end: page,
        group: group.to_string(),
    }
}

/// Batch `number` of three orders read from input 1 (`sha('a')`): two of one content in group A,
/// one in group B, a repeat left out, a warning; its outputs as given.
pub fn batch(number: u8, amends: bool, outputs: Vec<OutputFile>) -> BatchSaved {
    BatchSaved {
        batch: BatchId::new(number),
        amends,
        inputs: vec![InputPdf {
            pdf: PdfId::new(1),
            sha256: sha('a'),
            filename: "labels_1.pdf".into(),
            size: 1000,
            page_count: 4,
            create_time: time("2026-10-07T07:00:00+07:00"),
            modify_time: time("2026-10-07T07:01:00+07:00"),
        }],
        sort_rule: LabelSortRule {
            name: "default".into(),
            label_groups: vec![
                LabelGroupRule {
                    code: "A".into(),
                    name: "Sepatu".into(),
                    filter: Some("name contains \"sepatu\"".into()),
                },
                LabelGroupRule {
                    code: "Z".into(),
                    name: "Lainnya".into(),
                    filter: None,
                },
            ],
        },
        contents: vec![
            OrderContent {
                items: vec![ItemPick {
                    name: "Sepatu Standar Samping Motor | Pelindung Rantai".into(),
                    variant: "honda".into(),
                    quantity: 1,
                }],
            },
            OrderContent {
                items: vec![ItemPick {
                    name: "Spion Beat".into(),
                    variant: String::new(),
                    quantity: 2,
                }],
            },
        ],
        orders: vec![
            saved_order("580000000000000001", 0, "A", 1),
            saved_order("580000000000000003", 0, "A", 3),
            saved_order("580000000000000002", 1, "Z", 2),
        ],
        left_out: vec![LeftOutCopy {
            order_id: OrderId::new("580000000000000001"),
            pdf: PdfId::new(1),
            page_start: 4,
            page_end: 4,
            reason: LeftOutReason::Repeat {
                pdf: PdfId::new(1),
                page_start: 1,
            },
        }],
        outputs,
        warnings: vec![Finding {
            kind: FindingKind::new("slip_total_differs"),
            pdf: Some(PdfId::new(1)),
            page: Some(2),
            order_id: Some(OrderId::new("580000000000000002")),
            text: "Slip total differs for JY0000000002: printed 3, lines add up to 2.".into(),
        }],
        errors: Vec::new(),
    }
}
