//! orders.rs: the pages → orders step, display names and the warnings / no-slip stop
//! (03, 07). Hand-built pages only.

use packing_engine::orders::{PageRef, ReadError, Warning, display_name, read_batch};
use packing_engine::text::{PageText, Run};

const WIDTH: f64 = 298.0;

fn run(x: f64, y: f64, text: &str) -> Run {
    Run {
        x,
        y,
        text: text.to_string(),
    }
}

/// One slip row: (Product Name, variation, Seller SKU, quantity).
type Row<'a> = (&'a str, &'a str, &'a str, i64);

/// A page carrying a slip header and the given rows, plus an optional `Qty Total` and
/// `Customer Message`.
fn slip_page(text: &str, rows: &[Row], qty_total: Option<i64>, message: Option<&str>) -> PageText {
    let mut runs = vec![
        run(5.8, 300.0, "Product Name"),
        run(129.6, 300.0, "SKU"),
        run(172.2, 300.0, "Seller SKU"),
        run(264.6, 300.0, "Qty"),
    ];
    let mut y = 300.0 - 8.2;
    for (name, variation, seller_sku, quantity) in rows {
        runs.push(run(5.8, y, name));
        runs.push(run(129.6, y, variation));
        if !seller_sku.is_empty() {
            runs.push(run(172.2, y, seller_sku));
        }
        runs.push(run(268.6, y, &quantity.to_string()));
        y -= 8.2;
    }
    if let Some(total) = qty_total {
        y -= 8.2;
        runs.push(run(252.2, y, "Qty Total:"));
        runs.push(run(288.3, y, &total.to_string()));
    }
    y -= 8.2;
    runs.push(run(208.1, y, "Order ID:"));
    if let Some(message) = message {
        y -= 8.2;
        runs.push(run(10.5, y, "Customer Message"));
        runs.push(run(86.6, y, message));
    }
    PageText {
        text: text.to_string(),
        runs,
        width: WIDTH,
    }
}

/// A plain page: the Order ID is in the text but the page carries no slip table.
fn plain_page(text: &str) -> PageText {
    PageText {
        text: text.to_string(),
        runs: Vec::new(),
        width: WIDTH,
    }
}

fn one_row() -> Vec<Row<'static>> {
    vec![("Spion Beat", "honda", "SELL-SPI-01", 1)]
}

// --- display_name --------------------------------------------------------------------------

#[test]
fn display_name_cuts_at_the_pipe_and_appends_the_variation() {
    assert_eq!(
        display_name("Spion Beat | Kaca Spion Motor", ""),
        "Spion Beat"
    );
    assert_eq!(
        display_name("Spion Beat | Kaca Spion Motor", "honda"),
        "Spion Beat — honda"
    );
    assert_eq!(display_name("Sepatu", "Default"), "Sepatu");
}

// --- read_batch: multi-page order ----------------------------------------------------------

#[test]
fn second_page_without_order_id_continues_the_first() {
    let p0 = slip_page(
        "JY0000001234\n580000000000000001\nIn transit by: 07/10/2026 15:30",
        &one_row(),
        Some(1),
        None,
    );
    let p1 = plain_page("Jumlah : 3pcs");
    let reading = read_batch(vec![(0usize, 0usize, p0), (0, 1, p1)], &["a.pdf"]).unwrap();

    assert_eq!(reading.orders.len(), 1);
    let order = &reading.orders[0];
    assert_eq!(order.order.order_id, "580000000000000001");
    assert_eq!(order.order.tracking_id, "JY0000001234");
    assert_eq!(order.order.courier, "J&T Express");
    assert_eq!(
        order.order.ship_by.unwrap().to_string(),
        "2026-10-07T15:30:00"
    );
    assert_eq!(order.position, 1);
    assert_eq!(
        order.pages,
        vec![PageRef { file: 0, page: 0 }, PageRef { file: 0, page: 1 },]
    );
    assert_eq!(
        reading.continuation_pages,
        vec![PageRef { file: 0, page: 1 }]
    );
    assert!(reading.warnings.is_empty());
}

#[test]
fn a_slip_continuation_page_merges_the_message_and_keeps_the_lines() {
    let p0 = slip_page(
        "JY0000001234\n580000000000000001",
        &one_row(),
        Some(1),
        None,
    );
    let p1 = slip_page(
        "Jumlah : 3pcs",
        &[],
        None,
        Some("Tolong bubble wrap, jangan dilipat"),
    );
    let reading = read_batch(vec![(0usize, 0usize, p0), (0, 1, p1)], &["a.pdf"]).unwrap();

    let order = &reading.orders[0];
    assert_eq!(order.order.lines.len(), 1);
    assert_eq!(
        order.order.customer_message,
        "Tolong bubble wrap, jangan dilipat"
    );
    assert!(order.has_slip);
    assert_eq!(order.qty_total, Some(1));
    assert_eq!(order.pages.len(), 2);
}

#[test]
fn two_files_keep_the_download_order() {
    let p0 = slip_page(
        "JY0000001234\n580000000000000001",
        &one_row(),
        Some(1),
        None,
    );
    let p1 = slip_page(
        "JY0000009999\n580000000000000002",
        &one_row(),
        Some(1),
        None,
    );
    let reading = read_batch(vec![(0usize, 0usize, p0), (1, 0, p1)], &["a.pdf", "b.pdf"]).unwrap();

    let ids: Vec<&str> = reading
        .orders
        .iter()
        .map(|o| o.order.order_id.as_str())
        .collect();
    assert_eq!(ids, vec!["580000000000000001", "580000000000000002"]);
    assert_eq!(reading.orders[0].position, 1);
    assert_eq!(reading.orders[1].position, 2);
}

#[test]
fn a_page_without_order_id_cannot_continue_across_a_file() {
    let p0 = slip_page(
        "JY0000001234\n580000000000000001",
        &one_row(),
        Some(1),
        None,
    );
    let p1 = plain_page("Jumlah : 1pcs");
    let err = read_batch(vec![(0usize, 0usize, p0), (1, 0, p1)], &["a.pdf", "b.pdf"]).unwrap_err();
    assert_eq!(
        err,
        ReadError::NoOrderId {
            name: "b.pdf".into(),
            page: 0
        }
    );
}

// --- read_batch: errors --------------------------------------------------------------------

#[test]
fn first_page_without_order_id_stops_with_file_and_page() {
    let p0 = plain_page("JY0000001234\nJumlah : 1pcs");
    let err = read_batch(vec![(0usize, 0usize, p0)], &["noid.pdf"]).unwrap_err();
    assert_eq!(
        err.to_string(),
        "Page 1 of noid.pdf has no Order ID. This does not look like a TikTok Shop label \
         download. Check the file."
    );
}

#[test]
fn all_plain_pages_stop_with_the_all_pages_message() {
    let p0 = plain_page("JY0000001234\n580000000000000001");
    let p1 = plain_page("JY0000009999\n580000000000000002");
    let err = read_batch(vec![(0usize, 0usize, p0), (0, 1, p1)], &["a.pdf"]).unwrap_err();
    match err {
        ReadError::NoSlip(no_slip) => {
            assert_eq!(no_slip.plain_pages, 2);
            assert_eq!(no_slip.total_pages, 2);
            assert_eq!(
                no_slip.to_string(),
                "These labels have no packing slip. 2 of 2 pages are plain shipping labels. \
                 In the seller centre, download the labels again with Shipping label + Packing \
                 slip."
            );
        }
        other => panic!("expected NoSlip, got {other:?}"),
    }
}

#[test]
fn some_plain_pages_stop_naming_the_file_and_pages() {
    let p0 = slip_page(
        "JY0000001234\n580000000000000001",
        &one_row(),
        Some(1),
        None,
    );
    let p1 = plain_page("JY0000009999\n580000000000000002");
    let err = read_batch(vec![(0usize, 0usize, p0), (0, 1, p1)], &["a.pdf"]).unwrap_err();
    match err {
        ReadError::NoSlip(no_slip) => {
            assert_eq!(no_slip.plain_pages, 1);
            assert_eq!(no_slip.total_pages, 2);
            assert_eq!(
                no_slip.to_string(),
                "These labels have no packing slip. 1 of 2 pages are plain shipping labels \
                 (a.pdf, pages 2). In the seller centre, download the labels again with \
                 Shipping label + Packing slip."
            );
        }
        other => panic!("expected NoSlip, got {other:?}"),
    }
}

// --- read_batch: warnings ------------------------------------------------------------------

#[test]
fn qty_total_difference_is_warned() {
    let p0 = slip_page(
        "JY0000001234\n580000000000000001",
        &one_row(),
        Some(5),
        None,
    );
    let reading = read_batch(vec![(0usize, 0usize, p0)], &["a.pdf"]).unwrap();
    assert_eq!(
        reading.warnings,
        vec![Warning::QtyTotalDiffers {
            order_id: "580000000000000001".into(),
            tracking_id: "JY0000001234".into(),
            qty_total: 5,
            sum: 1,
        }]
    );
}

#[test]
fn unknown_courier_is_warned() {
    let p0 = slip_page(
        "580000000000000003\n000000000202",
        &one_row(),
        Some(1),
        None,
    );
    let reading = read_batch(vec![(0usize, 0usize, p0)], &["a.pdf"]).unwrap();
    assert_eq!(reading.orders[0].order.courier, "");
    assert_eq!(
        reading.warnings,
        vec![Warning::UnknownCourier {
            order_ids: vec!["580000000000000003".into()],
            tracking_ids: vec!["000000000202".into()],
        }]
    );
    assert_eq!(
        reading.warnings[0].to_string(),
        "Courier unknown for 1 order: 000000000202. They are sorted normally; only a \
         condition on courier cannot see them."
    );
}

#[test]
fn a_missing_tracking_id_is_warned() {
    let p0 = slip_page("580000000000000004", &one_row(), Some(1), None);
    let reading = read_batch(vec![(0usize, 0usize, p0)], &["a.pdf"]).unwrap();
    assert!(reading.orders[0].order.tracking_id.is_empty());
    assert!(
        reading
            .warnings
            .iter()
            .any(|w| matches!(w, Warning::TrackingIdMissing { order_ids } if order_ids == &vec!["580000000000000004".to_string()]))
    );
}
