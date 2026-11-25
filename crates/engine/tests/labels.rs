//! label.rs: Order ID, tracking ID and ship_by search, on plain text.

use packing_engine::ids::{OrderId, TrackingId};
use packing_engine::label::{Form, find_order_id, find_ship_by, find_tracking_id};
use packing_engine::workflows::load_bundled;

#[test]
fn an_order_id_is_the_eighteen_digits_between_boundaries() {
    // A spaced page is read by a second search, which accepts the fullwidth colon too.
    let spaced = format!("O r d e r I d : {}", "5 8 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 2");
    let fullwidth = "O r d e r I d ： 5 8 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 2";
    let cases = [
        // Removing whitespace first would glue tracking ID and Order ID into one number.
        (
            "JY0000001234\n580000000000000001",
            Some("580000000000000001"),
        ),
        ("Package ID: 1150000000000000001", None), // 19 digits
        ("900580000000000000012", None),           // preceded by a digit
        ("5800000000000000012", None),             // followed by a digit
        ("Total: 12345", None),
        (&spaced, Some("580000000000000002")),
        (fullwidth, Some("580000000000000002")),
    ];

    for (text, expected) in cases {
        assert_eq!(
            find_order_id(text).as_ref().map(OrderId::as_str),
            expected,
            "{text}"
        );
    }
}

#[test]
fn the_shopee_style_label_is_keyed_by_its_own_order_number() {
    let cases = [
        // The caption's number: six digits then eight letters or digits.
        ("No.Pesanan: 261014ABCD1234", Some("261014ABCD1234")),
        // The instant layout prints a space after the dot.
        (
            "No. Pesanan: 261111ABCD1234 Kode Pengambilan: 12AB",
            Some("261111ABCD1234"),
        ),
        // Whole only in the collapsed text on a spaced-out page.
        (
            "N o . P e s a n a n : 2 6 1 0 1 4 A B C D 1 2 3 4",
            Some("261014ABCD1234"),
        ),
        // A longer token is not cut down to its first fourteen characters.
        ("No.Pesanan: 261014ABCD1234X", None),
    ];
    for (text, expected) in cases {
        assert_eq!(
            find_order_id(text).as_ref().map(OrderId::as_str),
            expected,
            "{text}"
        );
    }

    // A TikTok-style page is still keyed by its 18-digit Order ID.
    assert_eq!(
        find_order_id("JY0000001234\n580000000000000001")
            .as_ref()
            .map(OrderId::as_str),
        Some("580000000000000001")
    );
}

#[test]
fn a_tracking_id_is_the_most_frequent_token_and_never_the_order_id() {
    // The forms of the bundled parser.toml.
    let workflows = load_bundled();
    let forms: Vec<Form<'_>> = workflows
        .classifiers
        .iter()
        .filter_map(|c| c.tracking_id.as_ref().map(|form| form.form()))
        .collect();
    let order_id = OrderId::new("580000000000000001");
    // The Order ID printed twice is not a tracking ID.
    assert_eq!(
        find_tracking_id(&format!("{order_id} {order_id}"), &order_id, &forms),
        None
    );

    // The most frequent token wins; a spaced page is read from its collapsed text.
    let cases = [
        (
            "JY0000001234 junk JY0000001234 JY0000001234 JY0000009999",
            Some("JY0000001234"),
        ),
        (
            "JX0000001234 junk JX0000001234 JX0000001234",
            Some("JX0000001234"),
        ),
        ("T K P 0 0 0 0 0 0 0 1 0 6 a b c", Some("TKP0000000106")),
    ];
    for (text, expected) in cases {
        assert_eq!(
            find_tracking_id(text, &OrderId::new(""), &forms)
                .as_ref()
                .map(TrackingId::as_str),
            expected,
            "{text}"
        );
    }
}

#[test]
fn ship_by_is_read_from_the_label_and_from_a_spaced_page() {
    let raw = find_ship_by("In transit by: 07/10/2026 15:30").unwrap();
    assert_eq!(raw.to_string(), "2026-10-07T15:30:00");
    assert_eq!(find_ship_by("no deadline here"), None);

    let spaced_line = "In transit by: 07/10/2026 15:30"
        .chars()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    let spaced = find_ship_by(&spaced_line).unwrap();
    assert_eq!(spaced.hour(), 15);

    // The Shopee-style label's deadline is a bare date: that day at 00:00, raw and spaced out.
    assert_eq!(
        find_ship_by("Batas Kirim: 15-10-2026").unwrap().to_string(),
        "2026-10-15T00:00:00"
    );
    let spaced_day = "Batas Kirim: 15-10-2026"
        .chars()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    assert_eq!(
        find_ship_by(&spaced_day).unwrap().to_string(),
        "2026-10-15T00:00:00"
    );
    // The `No. Resi` layout's text puts the empty box's `-` between the caption and the date.
    assert_eq!(
        find_ship_by("Berat:\nBatas Kirim:\n-\n15-10-2026\n300 gr")
            .unwrap()
            .to_string(),
        "2026-10-15T00:00:00"
    );
}
