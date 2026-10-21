//! label.rs: Order ID search, ship_by and courier deduction, on plain text.

use packing_engine::ids::{OrderId, TrackingId};
use packing_engine::label::{
    Courier, CourierTable, courier_data, deduce_courier, deduce_courier_in, find_order_id,
    find_ship_by, find_tracking_id,
};
use packing_engine::text::LogoImage;

/// A table of four couriers and one image answered *Not a courier*: two of them share the
/// `00` + 10-digit form, like SiCepat and J&T Cargo used to; Delta's form is a bare number too,
/// but names only itself.
fn table() -> CourierTable {
    CourierTable {
        couriers: vec![
            Courier::new(
                "Alpha",
                &["aa11"],
                &["shape-a"],
                Some("JY\\d{10}"),
                &["jet.co.id"],
            ),
            Courier::new("Beta", &["bb22"], &["shape-b"], Some("00\\d{10}"), &[]),
            Courier::new("Gamma", &[], &[], Some("00\\d{10}"), &["fasttrack"]),
            Courier::new(
                "Delta",
                &["dd77"],
                &["shape-d"],
                Some("57\\d{10}"),
                &["deltatrack"],
            ),
        ],
        not_courier: vec!["cc33".to_string()],
    }
}

/// A tracking ID as the tests write it: no text is the page whose label printed none.
fn tracking(form: Option<&str>) -> Option<TrackingId> {
    form.map(TrackingId::new)
}

/// A band image whose encoded bytes were hashed.
fn logo(hash: &str) -> LogoImage {
    LogoImage {
        hash: hash.to_string(),
        appearance: None,
    }
}

/// A band image whose appearance was hashed (a masked logo).
fn masked(hash: &str, shape: &str) -> LogoImage {
    LogoImage {
        hash: hash.to_string(),
        appearance: Some(shape.to_string()),
    }
}

/// One courier case: the page's printed clue, its tracking form, its band images, and the courier
/// the table must name (`None`: no courier).
type Case<'a> = (&'a str, Option<&'a str>, Vec<LogoImage>, Option<&'a str>);

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
fn a_tracking_id_is_the_most_frequent_token_and_never_the_order_id() {
    let table = courier_data();
    let order_id = OrderId::new("580000000000000001");
    // The Order ID printed twice is not a tracking ID.
    assert_eq!(
        find_tracking_id(&format!("{order_id} {order_id}"), &order_id, table),
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
            find_tracking_id(text, &OrderId::new(""), table)
                .as_ref()
                .map(TrackingId::as_str),
            expected,
            "{text}"
        );
    }
}

#[test]
fn a_courier_is_named_by_its_logo_its_form_or_the_text_clue() {
    let table = table();
    let cases: Vec<Case> = vec![
        // The byte hash names Alpha, whose form is JY.
        (
            "no clue",
            Some("JY0000001234"),
            vec![logo("aa11")],
            Some("Alpha"),
        ),
        // A masked logo is named by its appearance hash.
        (
            "no clue",
            Some("000000000202"),
            vec![masked("dd44", "shape-b")],
            Some("Beta"),
        ),
        // Alpha's logo on a page whose tracking form is the shared 00 one.
        ("no clue", Some("000000000202"), vec![logo("aa11")], None),
        // A logo with no form at all, and no form and no logo either.
        ("no clue", None, vec![logo("aa11")], None),
        ("no clue", None, vec![], None),
        // Two couriers share the 00 form, so the form alone names neither.
        ("no clue", Some("000000000202"), vec![], None),
        // A bare number carries no vendor marker, even where the form is one courier's.
        ("no clue", Some("570000000202"), vec![], None),
        // The JY form's lettered prefix is the marker, so a missing image still gets Alpha.
        ("no clue", Some("JY0000001234"), vec![], Some("Alpha")),
        // No band image: the printed clue may name a candidate.
        (
            "FastTrack parcel",
            Some("000000000202"),
            vec![],
            Some("Gamma"),
        ),
        // An image the table does not know was there but unreadable: the clue does not stand in.
        (
            "FastTrack parcel",
            Some("000000000202"),
            vec![logo("dd44")],
            None,
        ),
        // The clue must name a candidate of its own.
        (
            "FastTrack parcel",
            Some("JY0000001234"),
            vec![],
            Some("Alpha"),
        ),
        // An image answered Not a courier names nothing, and does not block the logo beside it.
        ("no clue", Some("JY0000001234"), vec![logo("cc33")], None),
        (
            "no clue",
            Some("JY0000001234"),
            vec![logo("cc33"), logo("aa11")],
            Some("Alpha"),
        ),
        // Two band images naming different couriers name nothing.
        (
            "no clue",
            Some("JY0000001234"),
            vec![logo("aa11"), masked("ee55", "shape-b")],
            None,
        ),
    ];

    for (clue, form, logos, expected) in cases {
        assert_eq!(
            deduce_courier_in(&table, clue, tracking(form).as_ref(), &logos),
            expected.map(str::to_string),
            "clue {clue:?}, form {form:?}"
        );
    }
}

#[test]
fn the_built_in_table_names_the_measured_logos_and_the_text_clue() {
    let cases: Vec<Case> = vec![
        // The measured logo hashes, each with the form of its own courier.
        (
            "no clue",
            Some("JY0000001234"),
            vec![logo("39b9b7b9f61d0e7f")],
            Some("J&T Express"),
        ),
        (
            "no clue",
            Some("000000000202"),
            vec![logo("cf33ed60daa291b1")],
            Some("SiCepat"),
        ),
        (
            "no clue",
            Some("TKP0000000106"),
            vec![logo("9131c1f94ad363d4")],
            Some("IDX"),
        ),
        (
            "no clue",
            Some("SPXID00000000002"),
            vec![logo("4583097ef370a670")],
            Some("SPX"),
        ),
        // A recognised logo with someone else's form names nothing.
        (
            "no clue",
            Some("JY0000001234"),
            vec![logo("cf33ed60daa291b1")],
            None,
        ),
        // An unseen logo blocks the fallbacks: the table must be asked.
        (
            "Shipping label",
            Some("JY0000001234"),
            vec![logo("0123456789abcdef")],
            None,
        ),
        // J&T Cargo's form is one candidate, so the clue names it where no logo is read; its
        // masked logo needs the appearance table, empty until the measured value is written.
        (
            "FastTrack parcel",
            Some("570000000202"),
            vec![],
            Some("J&T Cargo"),
        ),
        (
            "no clue",
            Some("570000000202"),
            vec![masked("331b", "unknown")],
            None,
        ),
        (
            "Shipped with www.jet.co.id today",
            Some("JY0000001234"),
            vec![],
            Some("J&T Express"),
        ),
        // 12 digits starting 00 is SiCepat's own form, but a bare number is no clue.
        ("no text clue", Some("000000000202"), vec![], None),
        ("no text clue", None, vec![], None),
    ];

    for (clue, form, logos, expected) in cases {
        assert_eq!(
            deduce_courier(clue, tracking(form).as_ref(), &logos),
            expected.map(str::to_string),
            "clue {clue:?}, form {form:?}"
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
}
