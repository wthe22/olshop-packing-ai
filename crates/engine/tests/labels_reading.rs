//! orders.rs: the pages → orders step, display names and the warnings / no-slip stop
//! (03, 07). Hand-built pages only.

use packing_engine::ids::{OrderId, TrackingId};
use packing_engine::label::{Courier, CourierTable};
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

/// A page of the Shopee-style label: its own order number and the SPX clues in the text, and
/// the slip's Indonesian header above the given rows.
fn shopee_page(rows: &[Row]) -> PageText {
    let mut runs = vec![
        run(9.2, 300.0, "#"),
        run(23.4, 300.0, "Nama Produk"),
        run(162.6, 300.0, "SKU"),
        run(212.0, 300.0, "V"),
        run(216.5, 300.0, "ariasi"),
        run(269.6, 300.0, "Qty"),
    ];
    let mut y = 300.0 - 8.2;
    for (name, variation, seller_sku, quantity) in rows {
        runs.push(run(23.4, y, name));
        runs.push(run(162.6, y, seller_sku));
        runs.push(run(212.0, y, variation));
        runs.push(run(269.6, y, &quantity.to_string()));
        y -= 8.2;
    }
    y -= 8.2;
    runs.push(run(8.5, y, "Pesan:"));
    runs.push(run(35.1, y, "261014ABCD1234"));
    PageText {
        text: "No.Pesanan: 261014ABCD1234\nSPXID00000000002A\nBatas Kirim: 15-10-2026".to_string(),
        runs,
        width: 297.12,
        logos: vec![logo("4583097ef370a670")],
    }
}

#[test]
fn a_display_name_cuts_at_the_pipe_and_appends_a_variation() {
    let cases = [
        ("Spion Beat | Kaca Spion Motor", "", "Spion Beat"),
        (
            "Spion Beat | Kaca Spion Motor",
            "honda",
            "Spion Beat — honda",
        ),
        ("Sepatu", "Default", "Sepatu"),
    ];
    for (name, variation, expected) in cases {
        assert_eq!(display_name(name, variation), expected);
    }
}

#[test]
fn the_shopee_style_page_reads_into_one_order_keyed_by_its_own_number() {
    let page = shopee_page(&[("Sepatu Standar Samping", "Biru,Honda", "SPION-MPX-ALEVO", 2)]);
    let reading = read_batch(vec![(0usize, 0usize, page)], &["spx.pdf"], table()).unwrap();

    assert_eq!(reading.orders.len(), 1);
    let order = &reading.orders[0];
    assert_eq!(order.order.order_id.as_str(), "261014ABCD1234");
    assert_eq!(
        order.order.tracking_id.as_ref().map(TrackingId::as_str),
        Some("SPXID00000000002")
    );
    assert_eq!(order.order.courier.as_deref(), Some("SPX"));
    assert_eq!(
        order.order.ship_by.unwrap().to_string(),
        "2026-10-15T00:00:00"
    );
    assert_eq!(order.order.lines.len(), 1);
    assert_eq!(order.order.lines[0].variation, "Biru,Honda");
    assert_eq!(order.order.lines[0].seller_sku, "SPION-MPX-ALEVO");
    assert_eq!(order.order.lines[0].quantity, 2);
    assert_eq!(order.qty_total, None);
    assert!(order.order.customer_message.is_empty());
    assert!(reading.warnings.is_empty());
    assert!(reading.unseen_logos.is_empty());
}

#[test]
fn a_batch_reads_into_orders_whose_pages_follow_the_download_order() {
    let slip = "JY0000001234\n580000000000000001\nIn transit by: 07/10/2026 15:30";

    // A plain page after the slip continues the order before it; the tracking ID, the courier
    // and the ship-by time come from the slip's text.
    let p0 = slip_page(slip, &one_row(), Some(1), None);
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
        vec![PageRef { file: 0, page: 0 }, PageRef { file: 0, page: 1 }]
    );
    assert_eq!(
        reading.continuation_pages,
        vec![PageRef { file: 0, page: 1 }]
    );
    assert!(reading.warnings.is_empty());

    // A slip continuation page keeps the lines and merges its message.
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

    // Two files keep the download order, numbered by position.
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
fn reading_stops_when_a_page_has_no_order_id_or_when_the_labels_have_no_slip() {
    // The first page without an Order ID stops before anything is read.
    let err = read_batch(
        vec![(0usize, 0usize, plain_page("JY0000001234\nJumlah : 1pcs"))],
        &["noid.pdf"],
        table(),
    )
    .unwrap_err();
    assert_eq!(
        err.to_string(),
        "Page 1 of noid.pdf has no Order ID. This does not look like a TikTok Shop label \
         download. Check the file."
    );

    // A page without an Order ID cannot continue an order from another file.
    let p0 = slip_page(
        "JY0000001234\n580000000000000001",
        &one_row(),
        Some(1),
        None,
    );
    let err = read_batch(
        vec![(0usize, 0usize, p0), (1, 0, plain_page("Jumlah : 1pcs"))],
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

    // Every page plain: the labels were downloaded without a packing slip.
    let p0 = plain_page("JY0000001234\n580000000000000001");
    let p1 = plain_page("JY0000009999\n580000000000000002");
    let err = read_batch(vec![(0usize, 0usize, p0), (0, 1, p1)], &["a.pdf"], table()).unwrap_err();
    let ReadError::NoSlip(no_slip) = err else {
        panic!("expected NoSlip, got {err:?}");
    };
    assert_eq!(no_slip.plain_pages, 2);
    assert_eq!(no_slip.total_pages, 2);
    assert_eq!(
        no_slip.to_string(),
        "These labels have no packing slip. 2 of 2 pages are plain shipping labels. \
         In the seller centre, download the labels again with Shipping label + Packing \
         slip."
    );

    // Some pages plain: the stop names the file and the pages, counting them.
    let p0 = slip_page(
        "JY0000001234\n580000000000000001",
        &one_row(),
        Some(1),
        None,
    );
    let p1 = plain_page("JY0000009999\n580000000000000002");
    let err = read_batch(vec![(0usize, 0usize, p0), (0, 1, p1)], &["a.pdf"], table()).unwrap_err();
    let ReadError::NoSlip(no_slip) = err else {
        panic!("expected NoSlip, got {err:?}");
    };
    assert_eq!(no_slip.plain_pages, 1);
    assert_eq!(no_slip.total_pages, 2);
    assert_eq!(
        no_slip.to_string(),
        "These labels have no packing slip. 1 of 2 pages are plain shipping labels \
         (a.pdf, pages 2). In the seller centre, download the labels again with \
         Shipping label + Packing slip."
    );
}

#[test]
fn the_reading_warns_about_a_quantity_difference_a_missing_tracking_id_and_an_unknown_courier() {
    // The slip's `Qty Total` disagrees with the quantity sum.
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

    // The shared 00 form names no courier on its own.
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

    // Without a tracking ID the order is named by its Order ID, and the miss is its own warning.
    let p0 = slip_page("580000000000000004", &one_row(), Some(1), None);
    let reading = read_batch(vec![(0usize, 0usize, p0)], &["a.pdf"], table()).unwrap();
    assert!(reading.orders[0].order.tracking_id.is_none());
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
fn the_courier_file_decides_which_tracking_forms_are_read() {
    // The owner's `couriers.toml`: a form the built-in list does not carry.
    let file = CourierTable {
        couriers: vec![Courier::new("Alpha", &[], &[], Some("AC\\d{10}"), &[])],
        not_courier: Vec::new(),
    };

    // The form the file carries is read.
    let p0 = slip_page(
        "AC0000001234\n580000000000000001",
        &one_row(),
        Some(1),
        None,
    );
    let reading = read_batch(vec![(0usize, 0usize, p0)], &["a.pdf"], &file).unwrap();
    assert_eq!(
        reading.orders[0]
            .order
            .tracking_id
            .as_ref()
            .map(TrackingId::as_str),
        Some("AC0000001234")
    );

    // A form only the built-in list carries is not: the file is the authority.
    let p0 = slip_page(
        "JY0000001234\n580000000000000002",
        &one_row(),
        Some(1),
        None,
    );
    let reading = read_batch(vec![(0usize, 0usize, p0)], &["a.pdf"], &file).unwrap();
    assert!(reading.orders[0].order.tracking_id.is_none());
}

#[test]
fn an_unseen_logo_is_reported_and_an_answer_names_the_orders_whose_form_agrees() {
    // The measured J&T Express logo names its order and reports nothing unseen.
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

    // One unknown strip on two orders is reported once, with its pages and orders.
    let mut reading = read_batch(
        vec![
            (
                0usize,
                0usize,
                page_with(
                    "JY0000001234\n580000000000000001",
                    &one_row(),
                    Some(1),
                    None,
                    vec![logo("ffff")],
                ),
            ),
            (0, 1, plain_page("Jumlah : 1pcs")),
            (
                0,
                2,
                page_with(
                    "000000000205\n580000000000000002",
                    &one_row(),
                    Some(1),
                    None,
                    vec![logo("ffff")],
                ),
            ),
        ],
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

    // The appearance hash is carried on the report, for the appearance table.
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
    let with_appearance = read_batch(vec![(0usize, 0usize, p0)], &["a.pdf"], table()).unwrap();
    assert_eq!(
        with_appearance.unseen_logos[0].appearance.as_deref(),
        Some("2a36acec58f5ab12")
    );
    assert_eq!(with_appearance.orders[0].order.courier, None);

    // The answer names only the orders whose form agrees with the courier, and is spent once.
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

    // *Not a courier* and *Skip* name nothing, and both spend the question.
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
