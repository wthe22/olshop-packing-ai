//! label.rs: Order ID search, ship_by and courier deduction, on plain text (cases ported from
//! `tools/analysis/tests/test_labels.py`).

use packing_engine::ids::{OrderId, TrackingId};
use packing_engine::label::{deduce_courier, find_order_id, find_ship_by, find_tracking_id};

// --- find_order_id -------------------------------------------------------------------------

#[test]
fn jt_order_id_is_found_before_whitespace_is_removed() {
    // Removing whitespace first would glue tracking ID and Order ID into one number.
    assert_eq!(
        find_order_id("JY0000001234\n580000000000000001")
            .as_ref()
            .map(OrderId::as_str),
        Some("580000000000000001")
    );
}

#[test]
fn lookbehind_and_lookahead_reject_longer_numbers() {
    assert_eq!(find_order_id("Package ID: 1150000000000000001"), None); // 19 digits
    assert_eq!(find_order_id("900580000000000000012"), None); // preceded by a digit
    assert_eq!(find_order_id("5800000000000000012"), None); // followed by a digit
    assert_eq!(find_order_id("Total: 12345"), None);
}

#[test]
fn spaced_characters_page_uses_the_second_search() {
    let spaced = format!("O r d e r I d : {}", "5 8 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 2");
    assert_eq!(
        find_order_id(&spaced).as_ref().map(OrderId::as_str),
        Some("580000000000000002")
    );
}

#[test]
fn second_search_accepts_the_fullwidth_colon() {
    let spaced = "O r d e r I d ： 5 8 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 2";
    assert_eq!(
        find_order_id(spaced).as_ref().map(OrderId::as_str),
        Some("580000000000000002")
    );
}

// --- find_tracking_id ----------------------------------------------------------------------

#[test]
fn tracking_id_is_never_the_order_id() {
    let order_id = OrderId::new("580000000000000001");
    assert_eq!(
        find_tracking_id(&format!("{order_id} {order_id}"), &order_id),
        None
    );
}

#[test]
fn tracking_id_is_the_most_frequent_token() {
    let text = "JY0000001234 junk JY0000001234 JY0000001234 JY0000009999";
    assert_eq!(
        find_tracking_id(text, &OrderId::new(""))
            .as_ref()
            .map(TrackingId::as_str),
        Some("JY0000001234")
    );
}

#[test]
fn tracking_id_of_a_spaced_page_uses_the_collapsed_text() {
    assert_eq!(
        find_tracking_id("T K P 0 0 0 0 0 0 0 1 0 6 a b c", &OrderId::new(""))
            .as_ref()
            .map(TrackingId::as_str),
        Some("TKP0000000106")
    );
}

// --- deduce_courier: the two-clue rule (courier.md items 1, 2, 4, 5) -----------------------

use packing_engine::label::{Courier, CourierTable, deduce_courier_in};
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
fn tracking(text: &str) -> Option<TrackingId> {
    (!text.is_empty()).then(|| TrackingId::new(text))
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

#[test]
fn the_logo_and_the_tracking_form_together_name_the_courier() {
    let table = table();
    // The byte hash names Alpha, whose form is JY.
    assert_eq!(
        deduce_courier_in(
            &table,
            "no clue",
            tracking("JY0000001234").as_ref(),
            &[logo("aa11")]
        ),
        "Alpha"
    );
    // A masked logo is named by its appearance hash (courier.md item 3).
    assert_eq!(
        deduce_courier_in(
            &table,
            "no clue",
            tracking("000000000202").as_ref(),
            &[masked("dd44", "shape-b")]
        ),
        "Beta"
    );
}

#[test]
fn a_logo_with_a_foreign_form_names_nothing() {
    let table = table();
    // Alpha's logo on a page whose tracking form is the shared 00 one.
    assert_eq!(
        deduce_courier_in(
            &table,
            "no clue",
            tracking("000000000202").as_ref(),
            &[logo("aa11")]
        ),
        ""
    );
}

#[test]
fn a_logo_without_a_form_names_nothing() {
    let table = table();
    assert_eq!(
        deduce_courier_in(&table, "no clue", None, &[logo("aa11")]),
        ""
    );
    // No form and no logo at all, either.
    assert_eq!(deduce_courier_in(&table, "no clue", None, &[]), "");
}

#[test]
fn a_form_without_a_logo_names_the_courier_only_when_it_carries_a_marker() {
    let table = table();
    // Two couriers share the 00 form, so the form alone names neither.
    assert_eq!(
        deduce_courier_in(&table, "no clue", tracking("000000000202").as_ref(), &[]),
        ""
    );
    // Delta's 57 form names only Delta, but a bare number is no clue by itself: nothing.
    assert_eq!(
        deduce_courier_in(&table, "no clue", tracking("570000000202").as_ref(), &[]),
        ""
    );
    // The JY form carries the vendor's marker (a lettered prefix), so a page whose image is
    // missing altogether still gets its courier.
    assert_eq!(
        deduce_courier_in(&table, "no clue", tracking("JY0000001234").as_ref(), &[]),
        "Alpha"
    );
}

#[test]
fn the_text_clue_stands_in_only_where_the_page_has_no_band_image() {
    let table = table();
    // No band image: the printed clue may name a candidate.
    assert_eq!(
        deduce_courier_in(
            &table,
            "FastTrack parcel",
            tracking("000000000202").as_ref(),
            &[]
        ),
        "Gamma"
    );
    // A band image the table does not know means the logo was there but unreadable: the text
    // clue does NOT stand in for it.
    assert_eq!(
        deduce_courier_in(
            &table,
            "FastTrack parcel",
            tracking("000000000202").as_ref(),
            &[logo("dd44")]
        ),
        ""
    );
    // The text clue must be a candidate too.
    assert_eq!(
        deduce_courier_in(
            &table,
            "FastTrack parcel",
            tracking("JY0000001234").as_ref(),
            &[]
        ),
        "Alpha"
    );
}

#[test]
fn a_not_courier_image_is_ignored() {
    let table = table();
    // cc33 was answered Not a courier (a channel strip): it names nothing...
    assert_eq!(
        deduce_courier_in(
            &table,
            "no clue",
            tracking("JY0000001234").as_ref(),
            &[logo("cc33")]
        ),
        ""
    );
    // ...and it does not get in the way of the courier's own logo beside it.
    assert_eq!(
        deduce_courier_in(
            &table,
            "no clue",
            tracking("JY0000001234").as_ref(),
            &[logo("cc33"), logo("aa11")]
        ),
        "Alpha"
    );
}

#[test]
fn two_band_images_naming_different_couriers_name_nothing() {
    let table = table();
    assert_eq!(
        deduce_courier_in(
            &table,
            "no clue",
            tracking("JY0000001234").as_ref(),
            &[logo("aa11"), masked("ee55", "shape-b")]
        ),
        ""
    );
}

#[test]
fn the_built_in_table_names_the_measured_logos() {
    // The measured logo hashes of courier.md item 1, each with the form of its own courier.
    assert_eq!(
        deduce_courier(
            "no clue",
            tracking("JY0000001234").as_ref(),
            &[logo("39b9b7b9f61d0e7f")]
        ),
        "J&T Express"
    );
    assert_eq!(
        deduce_courier(
            "no clue",
            tracking("000000000202").as_ref(),
            &[logo("cf33ed60daa291b1")]
        ),
        "SiCepat"
    );
    assert_eq!(
        deduce_courier(
            "no clue",
            tracking("TKP0000000106").as_ref(),
            &[logo("9131c1f94ad363d4")]
        ),
        "IDX"
    );
    assert_eq!(
        deduce_courier(
            "no clue",
            tracking("SPXID00000000002").as_ref(),
            &[logo("4583097ef370a670")]
        ),
        "SPX"
    );
    // A recognised logo with someone else's form names nothing.
    assert_eq!(
        deduce_courier(
            "no clue",
            tracking("JY0000001234").as_ref(),
            &[logo("cf33ed60daa291b1")]
        ),
        ""
    );
}

#[test]
fn an_unseen_logo_is_not_a_name() {
    // An unseen logo on a page blocks the fallbacks: the table must be asked (D-LABEL-4).
    assert_eq!(
        deduce_courier(
            "Shipping label",
            tracking("JY0000001234").as_ref(),
            &[logo("0123456789abcdef")]
        ),
        ""
    );
    // J&T Cargo: its form is one candidate, so the text clue can name it where no logo is read;
    // its masked logo needs the appearance table (empty until the measured value is written).
    assert_eq!(
        deduce_courier("FastTrack parcel", tracking("570000000202").as_ref(), &[]),
        "J&T Cargo"
    );
    assert_eq!(
        deduce_courier(
            "no clue",
            tracking("570000000202").as_ref(),
            &[masked("331b", "unknown")]
        ),
        ""
    );
}

#[test]
fn the_built_in_text_clue_still_names_jt_where_no_logo_image_is_read() {
    assert_eq!(
        deduce_courier(
            "Shipped with www.jet.co.id today",
            tracking("JY0000001234").as_ref(),
            &[]
        ),
        "J&T Express"
    );
}

#[test]
fn the_shared_twelve_digit_form_alone_gives_an_empty_courier() {
    // 12 digits starting 00 is SiCepat's own form now, but a bare number carries no vendor
    // marker: without a logo (or a text clue) the order stays unknown.
    assert_eq!(
        deduce_courier("no text clue", tracking("000000000202").as_ref(), &[]),
        ""
    );
}

#[test]
fn courier_is_empty_when_nothing_matches() {
    assert_eq!(deduce_courier("no text clue", None, &[]), "");
}

// --- find_ship_by --------------------------------------------------------------------------

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
