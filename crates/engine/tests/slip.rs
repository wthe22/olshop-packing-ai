//! slip.rs: the packing-slip table read by position, on hand-built runs.

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
        logos: Vec::new(),
    }
}

fn make_slip(rows: &[Row], qty_total: Option<i64>, message: Option<&str>) -> PageText {
    make_slip_with(COLS, rows, qty_total, message)
}

fn line(name: &str, variation: &str, seller_sku: &str, quantity: i64) -> SlipLine {
    SlipLine {
        name: name.to_string(),
        variation: variation.to_string(),
        seller_sku: seller_sku.to_string(),
        quantity,
    }
}

#[test]
fn the_slip_rows_come_from_the_header_columns_top_to_bottom() {
    // The columns are read from where the header words sit, not from a fixed x.
    let shifted = [
        ("Product Name", 20.0),
        ("SKU", 140.0),
        ("Seller SKU", 185.0),
        ("Qty", 260.0),
    ];
    let cases: Vec<(&str, PageText, Slip)> = vec![
        (
            "shifted header",
            make_slip_with(
                shifted,
                &[(&["Spion Beat"], &["honda"], &["SELL-SPI-01"], 2)],
                Some(2),
                None,
            ),
            Slip {
                lines: vec![line("Spion Beat", "honda", "SELL-SPI-01", 2)],
                qty_total: Some(2),
                customer_message: String::new(),
            },
        ),
        (
            "one row",
            make_slip(
                &[(&["Sepatu Standar"], &["honda"], &["SELL-SEP-01"], 1)],
                Some(1),
                None,
            ),
            Slip {
                lines: vec![line("Sepatu Standar", "honda", "SELL-SEP-01", 1)],
                qty_total: Some(1),
                customer_message: String::new(),
            },
        ),
        // Two rows keep the top-to-bottom order; `Default` is stored as no variation, and a
        // missing Seller SKU as an empty one.
        (
            "two rows",
            make_slip(
                &[
                    (&["Spion Beat"], &["Standard"], &["SELL-SPI-01"], 1),
                    (&["Knalpot Beat"], &["Default"], &[], 2),
                ],
                Some(3),
                None,
            ),
            Slip {
                lines: vec![
                    line("Spion Beat", "Standard", "SELL-SPI-01", 1),
                    line("Knalpot Beat", "", "", 2),
                ],
                qty_total: Some(3),
                customer_message: String::new(),
            },
        ),
        // A quantity total that does not match the rows is still read raw.
        (
            "wrong total",
            make_slip(&[(&["Spion Beat"], &["honda"], &[], 1)], Some(5), None),
            Slip {
                lines: vec![line("Spion Beat", "honda", "", 1)],
                qty_total: Some(5),
                customer_message: String::new(),
            },
        ),
        // Wrapped cells join with a space.
        (
            "wrapped cell",
            make_slip(
                &[(
                    &["Sepatu Standar Samping", "Motor | Pelindung Rantai"],
                    &["Standard"],
                    &["SELL-SEP-01"],
                    1,
                )],
                Some(1),
                None,
            ),
            Slip {
                lines: vec![line(
                    "Sepatu Standar Samping Motor | Pelindung Rantai",
                    "Standard",
                    "SELL-SEP-01",
                    1,
                )],
                qty_total: Some(1),
                customer_message: String::new(),
            },
        ),
        // A wrap right after a dash joins without a space, in every column.
        (
            "wrap after a dash",
            make_slip(
                &[(
                    &["Cover Knalpot Beat", "FI 2012-", "2015 Full Set"],
                    &["Yamaha-", "Mio"],
                    &["SPION-SCOL-DH-", "CY"],
                    1,
                )],
                Some(1),
                None,
            ),
            Slip {
                lines: vec![line(
                    "Cover Knalpot Beat FI 2012-2015 Full Set",
                    "Yamaha-Mio",
                    "SPION-SCOL-DH-CY",
                    1,
                )],
                qty_total: Some(1),
                customer_message: String::new(),
            },
        ),
    ];

    for (label, page, expected) in cases {
        assert_eq!(read_slip(&page), Some(expected), "{label}");
    }
}

#[test]
fn a_page_without_the_header_or_without_rows_reads_as_no_slip_or_an_empty_one() {
    // No slip table on the page at all.
    let plain = PageText {
        text: String::new(),
        runs: vec![run(5.8, 40.0, "JY0000001234"), run(5.8, 60.0, ORDER_ID)],
        width: WIDTH,
        logos: Vec::new(),
    };
    assert_eq!(read_slip(&plain), None);

    // The header alone.
    assert_eq!(
        read_slip(&make_slip(&[], None, None)),
        Some(Slip {
            lines: Vec::new(),
            qty_total: None,
            customer_message: String::new(),
        })
    );

    // A continuation page has the header and the message, but no rows and no total.
    let slip = read_slip(&make_slip(
        &[],
        None,
        Some("Tolong bubble wrap, jangan dilipat"),
    ))
    .unwrap();
    assert!(slip.lines.is_empty());
    assert_eq!(slip.qty_total, None);
    assert_eq!(slip.customer_message, "Tolong bubble wrap, jangan dilipat");

    // The message is read after a fullwidth colon too.
    let mut page = make_slip(&[], None, None);
    page.runs.push(run(10.5, 100.0, "Customer Message"));
    page.runs.push(run(78.0, 100.0, "\u{FF1A}"));
    page.runs.push(run(86.6, 100.0, "Halo"));
    assert_eq!(read_slip(&page).unwrap().customer_message, "Halo");
}
