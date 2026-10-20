//! orders.rs: the pages → orders step, display names and the warnings / no-slip stop
//! (03, 07). Hand-built pages only.

use packing_engine::ids::{OrderId, TrackingId};
use packing_engine::orders::{
    LogoAnswer, OrderLabel, PageRef, ReadError, Warning, display_name, read_batch,
};
use packing_engine::text::{LogoImage, PageText, Run};

/// The built-in courier list: the default of a data folder without a `couriers.toml`.
fn table() -> &'static packing_engine::label::CourierTable {
    packing_engine::label::courier_data()
}

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
    page_with(text, rows, qty_total, message, Vec::new())
}

/// `slip_page` with header-band images on the page (the courier's logo clue).
fn page_with(
    text: &str,
    rows: &[Row],
    qty_total: Option<i64>,
    message: Option<&str>,
    logos: Vec<LogoImage>,
) -> PageText {
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
        logos,
    }
}

/// A plain page: the Order ID is in the text but the page carries no slip table.
fn plain_page(text: &str) -> PageText {
    PageText {
        text: text.to_string(),
        runs: Vec::new(),
        width: WIDTH,
        logos: Vec::new(),
    }
}

/// One band image: its byte hash and, when `packing-pdf` computed it, its appearance hash.
fn logo(hash: &str) -> LogoImage {
    LogoImage {
        hash: hash.to_string(),
        appearance: None,
    }
}

fn one_row() -> Vec<Row<'static>> {
    vec![("Spion Beat", "honda", "SELL-SPI-01", 1)]
}

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

#[test]
fn second_page_without_order_id_continues_the_first() {
    let p0 = slip_page(
        "JY0000001234\n580000000000000001\nIn transit by: 07/10/2026 15:30",
        &one_row(),
        Some(1),
        None,
    );
    let p1 = plain_page("Jumlah : 3pcs");
    let reading = read_batch(vec![(0usize, 0usize, p0), (0, 1, p1)], &["a.pdf"], table()).unwrap();

    assert_eq!(reading.orders.len(), 1);
    let order = &reading.orders[0];
    assert_eq!(order.order.order_id.as_str(), "580000000000000001");
    assert_eq!(
        order.order.tracking_id.as_ref().map(TrackingId::as_str),
        Some("JY0000001234")
    );
    assert_eq!(order.order.courier.as_deref(), Some("J&T Express"));
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
    let reading = read_batch(vec![(0usize, 0usize, p0), (0, 1, p1)], &["a.pdf"], table()).unwrap();

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
    let reading = read_batch(
        vec![(0usize, 0usize, p0), (1, 0, p1)],
        &["a.pdf", "b.pdf"],
        table(),
    )
    .unwrap();

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
    let err = read_batch(
        vec![(0usize, 0usize, p0), (1, 0, p1)],
        &["a.pdf", "b.pdf"],
        table(),
    )
    .unwrap_err();
    assert_eq!(
        err,
        ReadError::NoOrderId {
            name: "b.pdf".into(),
            page: 0
        }
    );
}

#[test]
fn first_page_without_order_id_stops_with_file_and_page() {
    let p0 = plain_page("JY0000001234\nJumlah : 1pcs");
    let err = read_batch(vec![(0usize, 0usize, p0)], &["noid.pdf"], table()).unwrap_err();
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
    let err = read_batch(vec![(0usize, 0usize, p0), (0, 1, p1)], &["a.pdf"], table()).unwrap_err();
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
    let err = read_batch(vec![(0usize, 0usize, p0), (0, 1, p1)], &["a.pdf"], table()).unwrap_err();
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

#[test]
fn qty_total_difference_is_warned() {
    let p0 = slip_page(
        "JY0000001234\n580000000000000001",
        &one_row(),
        Some(5),
        None,
    );
    let reading = read_batch(vec![(0usize, 0usize, p0)], &["a.pdf"], table()).unwrap();
    assert_eq!(
        reading.warnings,
        vec![Warning::QtyTotalDiffers {
            order_id: OrderId::new("580000000000000001"),
            tracking_id: Some(TrackingId::new("JY0000001234")),
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
    let reading = read_batch(vec![(0usize, 0usize, p0)], &["a.pdf"], table()).unwrap();
    assert_eq!(reading.orders[0].order.courier, None);
    assert_eq!(
        reading.warnings,
        vec![Warning::UnknownCourier {
            orders: vec![OrderLabel::Tracking(TrackingId::new("000000000202"))],
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
    let reading = read_batch(vec![(0usize, 0usize, p0)], &["a.pdf"], table()).unwrap();
    assert!(reading.orders[0].order.tracking_id.is_none());
    // The unknown-courier line names that order by its Order ID, as it has no tracking ID.
    let unknown = reading
        .warnings
        .iter()
        .find(|warning| matches!(warning, Warning::UnknownCourier { .. }))
        .expect("the courier is unknown");
    assert!(unknown.to_string().contains("580000000000000004"));
    assert!(
        reading
            .warnings
            .iter()
            .any(|w| matches!(w, Warning::TrackingIdMissing { order_ids } if order_ids == &vec![OrderId::new("580000000000000004")]))
    );
}

#[test]
fn a_known_logo_names_the_order_and_reports_no_unseen_logo() {
    // The measured J&T Express logo hash (courier.md item 1) with its own JY form.
    let p0 = page_with(
        "JY0000001234\n580000000000000001",
        &one_row(),
        Some(1),
        None,
        vec![logo("39b9b7b9f61d0e7f")],
    );
    let reading = read_batch(vec![(0usize, 0usize, p0)], &["a.pdf"], table()).unwrap();
    assert_eq!(
        reading.orders[0].order.courier.as_deref(),
        Some("J&T Express")
    );
    assert!(reading.unseen_logos.is_empty());
}

#[test]
fn an_unseen_logo_is_reported_with_its_pages_and_orders() {
    // Two orders carry the same unknown strip.
    let p0 = page_with(
        "JY0000001234\n580000000000000001",
        &one_row(),
        Some(1),
        None,
        vec![logo("ffff")],
    );
    let p1 = plain_page("Jumlah : 1pcs");
    let p2 = page_with(
        "JY0000001235\n580000000000000002",
        &one_row(),
        Some(1),
        None,
        vec![logo("ffff")],
    );
    let reading = read_batch(
        vec![(0usize, 0usize, p0), (0, 1, p1), (0, 2, p2)],
        &["a.pdf"],
        table(),
    )
    .unwrap();

    assert_eq!(reading.unseen_logos.len(), 1);
    let entry = &reading.unseen_logos[0];
    assert_eq!(entry.hash, "ffff");
    assert_eq!(entry.appearance, None);
    assert_eq!(entry.pages, 2);
    assert_eq!(entry.orders(), 2);
    assert_eq!(
        entry.order_ids,
        vec![
            OrderId::new("580000000000000001"),
            OrderId::new("580000000000000002")
        ]
    );
    // The unknown logo names neither order: the table has to be asked.
    assert_eq!(reading.orders[0].order.courier, None);
    assert_eq!(reading.orders[1].order.courier, None);
}

#[test]
fn an_appearance_hash_is_carried_on_the_unseen_logo() {
    let p0 = page_with(
        "570000000202\n580000000000000001",
        &one_row(),
        Some(1),
        None,
        vec![LogoImage {
            hash: "ffff".to_string(),
            appearance: Some("2a36acec58f5ab12".to_string()),
        }],
    );
    let reading = read_batch(vec![(0usize, 0usize, p0)], &["a.pdf"], table()).unwrap();
    assert_eq!(
        reading.unseen_logos[0].appearance.as_deref(),
        Some("2a36acec58f5ab12")
    );
    assert_eq!(reading.orders[0].order.courier, None);
}

#[test]
fn an_answered_logo_names_the_orders_only_where_the_form_agrees() {
    // One order with the JY form, one with the shared 00 form; both carry the unknown strip.
    let p0 = page_with(
        "JY0000001234\n580000000000000001",
        &one_row(),
        Some(1),
        None,
        vec![logo("ffff")],
    );
    let p1 = page_with(
        "000000000205\n580000000000000002",
        &one_row(),
        Some(1),
        None,
        vec![logo("ffff")],
    );
    let mut reading =
        read_batch(vec![(0usize, 0usize, p0), (0, 1, p1)], &["a.pdf"], table()).unwrap();
    assert_eq!(reading.orders[0].order.courier, None);
    assert_eq!(reading.orders[1].order.courier, None);

    let named = reading.apply_logo_answer(
        "ffff",
        &LogoAnswer::Courier {
            name: "J&T Express".to_string(),
        },
        table(),
    );
    assert_eq!(named, 1);
    assert_eq!(
        reading.orders[0].order.courier.as_deref(),
        Some("J&T Express")
    );
    assert_eq!(reading.orders[1].order.courier, None);
    assert!(reading.unseen_logos.is_empty());
    // The answer is spent: a second one changes nothing.
    assert_eq!(
        reading.apply_logo_answer(
            "ffff",
            &LogoAnswer::Courier {
                name: "SiCepat".to_string(),
            },
            table(),
        ),
        0
    );
}

#[test]
fn a_not_courier_answer_names_nothing() {
    let p0 = page_with(
        "JY0000001234\n580000000000000001",
        &one_row(),
        Some(1),
        None,
        vec![logo("ffff")],
    );
    let mut reading = read_batch(vec![(0usize, 0usize, p0)], &["a.pdf"], table()).unwrap();
    assert_eq!(
        reading.apply_logo_answer("ffff", &LogoAnswer::NotCourier, table()),
        0
    );
    assert_eq!(
        reading.apply_logo_answer("ffff", &LogoAnswer::Skip, table()),
        0
    );
    assert_eq!(reading.orders[0].order.courier, None);
    assert!(reading.unseen_logos.is_empty());
}
