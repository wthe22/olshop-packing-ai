//! label.rs: Order ID search, ship_by and courier deduction, on plain text (cases ported from
//! `script/tests/test_labels.py`).

use packing_engine::label::{deduce_courier, find_order_id, find_ship_by, find_tracking_id};

// --- find_order_id -------------------------------------------------------------------------

#[test]
fn jt_order_id_is_found_before_whitespace_is_removed() {
    // Removing whitespace first would glue tracking ID and Order ID into one number.
    assert_eq!(
        find_order_id("JY0000001234\n580000000000000001").as_deref(),
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
        find_order_id(&spaced).as_deref(),
        Some("580000000000000002")
    );
}

#[test]
fn second_search_accepts_the_fullwidth_colon() {
    let spaced = "O r d e r I d ： 5 8 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 2";
    assert_eq!(find_order_id(spaced).as_deref(), Some("580000000000000002"));
}

// --- find_tracking_id ----------------------------------------------------------------------

#[test]
fn tracking_id_is_never_the_order_id() {
    let order_id = "580000000000000001";
    assert_eq!(
        find_tracking_id(&format!("{order_id} {order_id}"), order_id),
        ""
    );
}

#[test]
fn tracking_id_is_the_most_frequent_token() {
    let text = "JY0000001234 junk JY0000001234 JY0000001234 JY0000009999";
    assert_eq!(find_tracking_id(text, ""), "JY0000001234");
}

#[test]
fn tracking_id_of_a_spaced_page_uses_the_collapsed_text() {
    assert_eq!(
        find_tracking_id("T K P 0 0 0 0 0 0 0 1 0 6 a b c", ""),
        "TKP0000000106"
    );
}

// --- deduce_courier ------------------------------------------------------------------------

#[test]
fn courier_from_the_label_text() {
    assert_eq!(
        deduce_courier("Shipped with www.jet.co.id today", "JY0000001234"),
        "J&T Express"
    );
}

#[test]
fn courier_from_an_unambiguous_tracking_form() {
    assert_eq!(
        deduce_courier("no text clue", "JY0000001234"),
        "J&T Express"
    );
    assert_eq!(deduce_courier("no text clue", "TKP0000000106"), "IDX");
}

#[test]
fn ambiguous_tracking_form_alone_gives_an_empty_courier() {
    // 12 digits starting 00 is shared by SiCepat and J&T Cargo.
    assert_eq!(deduce_courier("no text clue", "000000000202"), "");
}

#[test]
fn courier_is_empty_when_nothing_matches() {
    assert_eq!(deduce_courier("no text clue", ""), "");
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
