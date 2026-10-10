//! slip.rs: the packing-slip table read by position, on hand-built runs (cases ported from
//! `script/tests/test_slip.py`).

use packing_engine::slip::{Slip, SlipLine, read_slip};
use packing_engine::text::{PageText, Run};

const WIDTH: f64 = 298.0;
const HEADER_Y: f64 = 300.0;
const STEP: f64 = 8.2;
const ORDER_ID: &str = "580000000000000001";

const COLS: [(&str, f64); 4] = [
    ("Product Name", 5.8),
    ("SKU", 129.6),
    ("Seller SKU", 172.2),
    ("Qty", 264.6),
];

fn run(x: f64, y: f64, text: &str) -> Run {
    Run {
        x,
        y,
        text: text.to_string(),
    }
}

/// One row: (Product Name lines, variation lines, Seller SKU lines, quantity).
type Row<'a> = (&'a [&'a str], &'a [&'a str], &'a [&'a str], i64);

fn make_slip_with(
    cols: [(&str, f64); 4],
    rows: &[Row],
    qty_total: Option<i64>,
    message: Option<&str>,
) -> PageText {
    let product = cols[0].1;
    let sku = cols[1].1;
    let seller = cols[2].1;
    let qty = cols[3].1;

    let mut runs = Vec::new();
    for (word, x) in cols {
        runs.push(run(x, HEADER_Y, word));
    }
    let mut y = HEADER_Y - STEP;
    for (name, variation, seller_sku, quantity) in rows {
        let lines = name.len().max(variation.len()).max(seller_sku.len()).max(1);
        for (i, line) in name.iter().enumerate() {
            runs.push(run(product, y - i as f64 * STEP, line));
        }
        for (i, line) in variation.iter().enumerate() {
            runs.push(run(sku, y - i as f64 * STEP, line));
        }
        for (i, line) in seller_sku.iter().enumerate() {
            runs.push(run(seller, y - i as f64 * STEP, line));
        }
        runs.push(run(qty + 4.0, y, &quantity.to_string()));
        y -= STEP * lines as f64;
    }
    if let Some(total) = qty_total {
        y -= STEP;
        runs.push(run(252.2, y, "Qty Total:"));
        runs.push(run(288.3, y, &total.to_string()));
    }
    y -= STEP;
    runs.push(run(208.1, y, &format!("Order ID: {ORDER_ID}")));
    if let Some(message) = message {
        y -= STEP;
        runs.push(run(10.5, y, "Customer Message"));
        runs.push(run(70.0, y, ":"));
        runs.push(run(86.6, y, message));
    }
    PageText {
        text: String::new(),
        runs,
        width: WIDTH,
    }
}

fn make_slip(rows: &[Row], qty_total: Option<i64>, message: Option<&str>) -> PageText {
    make_slip_with(COLS, rows, qty_total, message)
}

#[test]
fn page_without_a_slip_header_returns_none() {
    let page = PageText {
        text: String::new(),
        runs: vec![run(5.8, 40.0, "JY0000001234"), run(5.8, 60.0, ORDER_ID)],
        width: WIDTH,
    };
    assert_eq!(read_slip(&page), None);
}

#[test]
fn columns_come_from_the_header_positions_not_fixed_x() {
    let shifted = [
        ("Product Name", 20.0),
        ("SKU", 140.0),
        ("Seller SKU", 185.0),
        ("Qty", 260.0),
    ];
    let page = make_slip_with(
        shifted,
        &[(&["Spion Beat"], &["honda"], &["SELL-SPI-01"], 2)],
        Some(2),
        None,
    );
    assert_eq!(
        read_slip(&page),
        Some(Slip {
            lines: vec![SlipLine {
                name: "Spion Beat".into(),
                variation: "honda".into(),
                seller_sku: "SELL-SPI-01".into(),
                quantity: 2,
            }],
            qty_total: Some(2),
            customer_message: String::new(),
        })
    );
}

#[test]
fn one_row_and_qty_total() {
    let page = make_slip(
        &[(&["Sepatu Standar"], &["honda"], &["SELL-SEP-01"], 1)],
        Some(1),
        None,
    );
    assert_eq!(
        read_slip(&page),
        Some(Slip {
            lines: vec![SlipLine {
                name: "Sepatu Standar".into(),
                variation: "honda".into(),
                seller_sku: "SELL-SEP-01".into(),
                quantity: 1,
            }],
            qty_total: Some(1),
            customer_message: String::new(),
        })
    );
}

#[test]
fn default_variation_is_stored_empty() {
    let page = make_slip(&[(&["Knalpot Beat"], &["Default"], &[], 3)], Some(3), None);
    assert_eq!(
        read_slip(&page).unwrap().lines,
        vec![SlipLine {
            name: "Knalpot Beat".into(),
            variation: String::new(),
            seller_sku: String::new(),
            quantity: 3,
        }]
    );
}

#[test]
fn two_rows_keep_the_top_to_bottom_order() {
    let page = make_slip(
        &[
            (&["Spion Beat"], &["Standard"], &["SELL-SPI-01"], 1),
            (&["Knalpot Beat"], &["Default"], &[], 2),
        ],
        Some(3),
        None,
    );
    let slip = read_slip(&page).unwrap();
    assert_eq!(
        slip.lines,
        vec![
            SlipLine {
                name: "Spion Beat".into(),
                variation: "Standard".into(),
                seller_sku: "SELL-SPI-01".into(),
                quantity: 1,
            },
            SlipLine {
                name: "Knalpot Beat".into(),
                variation: String::new(),
                seller_sku: String::new(),
                quantity: 2,
            },
        ]
    );
    assert_eq!(slip.qty_total, Some(3));
}

#[test]
fn a_qty_total_that_does_not_match_is_still_read_raw() {
    let page = make_slip(&[(&["Spion Beat"], &["honda"], &[], 1)], Some(5), None);
    assert_eq!(read_slip(&page).unwrap().qty_total, Some(5));
}

#[test]
fn seller_sku_may_be_absent() {
    let page = make_slip(&[(&["Cover Body"], &["Default"], &[], 1)], Some(1), None);
    assert_eq!(read_slip(&page).unwrap().lines[0].seller_sku, "");
}

#[test]
fn wrapped_cells_join_with_a_space() {
    let page = make_slip(
        &[(
            &["Sepatu Standar Samping", "Motor | Pelindung Rantai"],
            &["Standard"],
            &["SELL-SEP-01"],
            1,
        )],
        Some(1),
        None,
    );
    assert_eq!(
        read_slip(&page).unwrap().lines[0].name,
        "Sepatu Standar Samping Motor | Pelindung Rantai"
    );
}

#[test]
fn a_wrap_right_after_a_dash_joins_without_a_space() {
    let page = make_slip(
        &[(
            &["Cover Knalpot Beat", "FI 2012-", "2015 Full Set"],
            &["Yamaha-", "Mio"],
            &["SPION-SCOL-DH-", "CY"],
            1,
        )],
        Some(1),
        None,
    );
    let line = &read_slip(&page).unwrap().lines[0];
    assert_eq!(line.name, "Cover Knalpot Beat FI 2012-2015 Full Set");
    assert_eq!(line.variation, "Yamaha-Mio");
    assert_eq!(line.seller_sku, "SPION-SCOL-DH-CY");
}

#[test]
fn continuation_page_has_the_header_but_no_rows() {
    let page = make_slip(&[], None, Some("Tolong bubble wrap, jangan dilipat"));
    let slip = read_slip(&page).unwrap();
    assert!(slip.lines.is_empty());
    assert_eq!(slip.qty_total, None);
    assert_eq!(slip.customer_message, "Tolong bubble wrap, jangan dilipat");
}

#[test]
fn customer_message_accepts_the_fullwidth_colon() {
    let mut page = make_slip(&[], None, None);
    page.runs.push(run(10.5, 100.0, "Customer Message"));
    page.runs.push(run(78.0, 100.0, "\u{FF1A}"));
    page.runs.push(run(86.6, 100.0, "Halo"));
    assert_eq!(read_slip(&page).unwrap().customer_message, "Halo");
}

#[test]
fn header_only_page_reads_as_an_empty_slip() {
    let page = make_slip(&[], None, None);
    assert_eq!(
        read_slip(&page),
        Some(Slip {
            lines: Vec::new(),
            qty_total: None,
            customer_message: String::new(),
        })
    );
}
