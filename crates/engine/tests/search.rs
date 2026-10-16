//! `search.rs`: finding a saved order by a condition over its stored fields (2.21,
//! [Search](../../docs/software-design/ui/search.md)).
//!
//! Search reads `saved[]` only: an order of an open draft is never there, so it is never found
//! (07 › *`state.json`*). An order saved before search existed keeps only its IDs, so only a
//! condition on `order_id` or `tracking_id` can match it; its row says *items not stored*
//! (Q-UI-3).

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use packing_engine::day::{
    BatchEntry, DayState, FileEntry, PackingListChoice, PdfEntry, SavedItem, SavedOrder,
};
use packing_engine::orders::{Line, Order};
use packing_engine::rules::{evaluate, parse_condition};
use packing_engine::search::search;
use serde_json::Value;

struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "packing-engine-search-{tag}-{}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("temp dir");
        TempDir(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// A saved order with the searched fields written, as a save does; `lines` are `(name, quantity)`.
fn saved(order_id: &str, tracking_id: &str, lines: &[(&str, i64)]) -> SavedOrder {
    SavedOrder {
        order_id: order_id.to_string(),
        tracking_id: tracking_id.to_string(),
        batch: 1,
        pdf: 1,
        run: 1,
        items: lines
            .iter()
            .map(|(name, quantity)| SavedItem {
                name: (*name).to_string(),
                variation: "42".to_string(),
                seller_sku: "SP-0001".to_string(),
                quantity: *quantity,
            })
            .collect(),
        courier: "J&T Express".to_string(),
        total_quantity: lines.iter().map(|(_, quantity)| quantity).sum(),
        distinct_items: lines.len() as i64,
        ship_by: None,
    }
}

/// An order saved before search existed: only its IDs, no searched fields.
fn legacy(order_id: &str, tracking_id: &str) -> SavedOrder {
    SavedOrder {
        order_id: order_id.to_string(),
        tracking_id: tracking_id.to_string(),
        batch: 1,
        pdf: 1,
        run: 1,
        items: Vec::new(),
        courier: String::new(),
        total_quantity: 0,
        distinct_items: 0,
        ship_by: None,
    }
}

/// A day with one batch and its single saved PDF, holding `saved`.
fn day_state(day: &str, saved: Vec<SavedOrder>, pdf_file: &str) -> DayState {
    let mut state = DayState::new(day);
    state.batches.push(BatchEntry {
        batch: 1,
        save_time: "2026-10-07T07:40:12+07:00".to_string(),
        files: vec![FileEntry {
            name: "labels_1.pdf".to_string(),
            size: 10,
            xxh3: "9f2c".to_string(),
            pages: 1,
            legacy_sha256: None,
        }],
        pdfs: vec![PdfEntry {
            number: 1,
            code: "A".to_string(),
            name: "Sepatu".to_string(),
            file: pdf_file.to_string(),
            orders: 1,
            runs: 1,
        }],
        packing_list: PackingListChoice::default(),
        warnings: Vec::new(),
        errors: Vec::new(),
    });
    state.saved = saved;
    state
}

/// The order the saved fields describe, for a "same as a category" comparison.
fn read_order(order_id: &str, tracking_id: &str, lines: &[(&str, i64)]) -> Order {
    Order {
        order_id: order_id.to_string(),
        tracking_id: tracking_id.to_string(),
        courier: "J&T Express".to_string(),
        ship_by: None,
        lines: lines
            .iter()
            .map(|(name, quantity)| Line {
                name: (*name).to_string(),
                variation: "42".to_string(),
                seller_sku: "SP-0001".to_string(),
                quantity: *quantity,
                display_name: format!("{name} — 42"),
            })
            .collect(),
        customer_message: String::new(),
    }
}

#[test]
fn a_tracking_id_finds_its_one_order() {
    let state = day_state(
        "2026-10-07",
        vec![
            saved("580000000000000001", "JY0000001234", &[("Sepatu", 2)]),
            saved("580000000000000002", "JY0000005678", &[("Spion", 1)]),
        ],
        "1 A Sepatu ×1.pdf",
    );
    let cond = parse_condition("tracking_id equals \"JY0000001234\"").expect("condition");

    let results = search(&cond, &[("2026-10-07", &state)]);
    assert_eq!(results.orders, 1);
    assert_eq!(results.batches, 1);
    assert_eq!(results.rows.len(), 1);
    let row = &results.rows[0];
    assert_eq!(row.order_id, "580000000000000001");
    assert_eq!(row.tracking_id, "JY0000001234");
    assert_eq!(row.courier, "J&T Express");
    assert_eq!(row.items.len(), 1);
    assert_eq!(row.items[0].name, "Sepatu");
    assert_eq!(row.items[0].quantity, 2);
    assert_eq!(row.batch, 1);
    assert_eq!(row.pdf, 1);
    assert_eq!(row.file, "1 A Sepatu ×1.pdf");
    assert!(!row.items_not_stored);
}

#[test]
fn an_item_field_matches_when_one_line_matches() {
    // Two lines: the first `Sepatu`, the second `Spion`.
    let state = day_state(
        "2026-10-07",
        vec![saved(
            "580000000000000001",
            "JY0000001234",
            &[("Sepatu", 1), ("Spion Beat", 2)],
        )],
        "1 A Sepatu ×1.pdf",
    );

    let sepatu = parse_condition("name contains \"sepatu\"").expect("condition");
    assert_eq!(search(&sepatu, &[("2026-10-07", &state)]).orders, 1);

    let spion = parse_condition("name starts_with \"spion\"").expect("condition");
    assert_eq!(search(&spion, &[("2026-10-07", &state)]).orders, 1);

    // A field no line carries matches nothing.
    let none = parse_condition("seller_sku equals \"XX\"").expect("condition");
    assert_eq!(search(&none, &[("2026-10-07", &state)]).orders, 0);
}

#[test]
fn not_and_at_least_behave_as_in_a_category() {
    let state = day_state(
        "2026-10-07",
        vec![saved(
            "580000000000000001",
            "JY0000001234",
            &[("Sepatu", 1), ("Spion Beat", 2)],
        )],
        "1 A Sepatu ×1.pdf",
    );
    // The same order as a read order; every condition must match the same way as a category's
    // `evaluate` on it (the item-field "some line" rule, conditions.md).
    let order = read_order(
        "580000000000000001",
        "JY0000001234",
        &[("Sepatu", 1), ("Spion Beat", 2)],
    );
    for text in [
        "not name contains \"sepatu\"",  // false: a line is a sepatu
        "not name contains \"knalpot\"", // true: no line is a knalpot
        "line_quantity at least 2",      // written by the boxes as `line_quantity >= 2`: true
        "total_quantity at least 3",     // true: 1 + 2
        "distinct_items = 2",            // true
        "name contains \"spion\" and line_quantity >= 2", // true: different lines may carry each
    ] {
        // The boxes editor writes "at least"/"at most" as the number operators.
        let normalized = text
            .replace(" at least ", " >= ")
            .replace(" at most ", " <= ");
        let cond = parse_condition(&normalized).expect("condition");
        let expected = evaluate(&cond, &order);
        let found = search(&cond, &[("2026-10-07", &state)]).orders;
        assert_eq!(
            found,
            if expected { 1 } else { 0 },
            "{normalized:?}: search disagreed with a category"
        );
    }
}

#[test]
fn this_day_does_not_see_another_days_order_but_all_days_does() {
    let day_1 = day_state(
        "2026-10-07",
        vec![saved(
            "580000000000000001",
            "JY0000001234",
            &[("Sepatu", 1)],
        )],
        "1 A Sepatu ×1.pdf",
    );
    let day_2 = day_state(
        "2026-10-06",
        vec![saved("580000000000000009", "JY0000009999", &[("Spion", 1)])],
        "1 A Sepatu ×1.pdf",
    );
    let cond = parse_condition("tracking_id starts_with \"JY\"").expect("condition");

    // *This day*: one day state only.
    let this_day = search(&cond, &[("2026-10-07", &day_1)]);
    assert_eq!(this_day.orders, 1);
    assert_eq!(this_day.rows[0].order_id, "580000000000000001");

    // *All days*: every day state; both orders match.
    let all = search(&cond, &[("2026-10-07", &day_1), ("2026-10-06", &day_2)]);
    assert_eq!(all.orders, 2);
    assert_eq!(all.batches, 2, "one batch per day");
    assert_eq!(all.rows[0].order_id, "580000000000000001");
    assert_eq!(all.rows[1].order_id, "580000000000000009");
}

#[test]
fn an_order_saved_before_search_is_found_by_its_ids_only() {
    let state = day_state(
        "2026-10-07",
        vec![legacy("580000000000000001", "JY0000001234")],
        "1 A Sepatu ×1.pdf",
    );

    let by_tracking = parse_condition("tracking_id equals \"JY0000001234\"").expect("condition");
    let results = search(&by_tracking, &[("2026-10-07", &state)]);
    assert_eq!(results.orders, 1);
    assert!(results.rows[0].items_not_stored);
    assert!(results.rows[0].items.is_empty());

    let by_order = parse_condition("order_id contains \"580000000000000001\"").expect("condition");
    assert_eq!(search(&by_order, &[("2026-10-07", &state)]).orders, 1);

    // A condition on an item or other stored field cannot see it.
    for text in [
        "name contains \"sepatu\"",
        "courier starts_with \"J&T\"",
        "total_quantity >= 0",
        "distinct_items = 0",
    ] {
        let cond = parse_condition(text).expect("condition");
        assert_eq!(
            search(&cond, &[("2026-10-07", &state)]).orders,
            0,
            "{text:?} must not see an order without stored items"
        );
    }
}

#[test]
fn a_stored_ship_by_is_searched_as_the_label_time() {
    // The label's `In transit by` is stored as `state.json` writes it (a jiff ISO date-time,
    // 07 › *`state.json`*); the condition language reads it as any date-time (conditions.md).
    let stored = jiff::civil::date(2026, 10, 7).at(17, 0, 0, 0);
    assert_eq!(stored.to_string(), "2026-10-07T17:00:00");
    let mut order = saved("580000000000000001", "JY0000001234", &[("Sepatu", 1)]);
    order.ship_by = Some(stored.to_string());
    let state = day_state("2026-10-07", vec![order], "1 A Sepatu ×1.pdf");

    let before = parse_condition("ship_by < \"2026-10-07 18:00\"").expect("condition");
    assert_eq!(search(&before, &[("2026-10-07", &state)]).orders, 1);
    let after = parse_condition("ship_by > \"2026-10-07 18:00\"").expect("condition");
    assert_eq!(search(&after, &[("2026-10-07", &state)]).orders, 0);
    let that_day = parse_condition("ship_by = \"2026-10-07\"").expect("condition");
    assert_eq!(search(&that_day, &[("2026-10-07", &state)]).orders, 1);
}

#[test]
fn the_searched_fields_round_trip_through_state_json() {
    let dir = TempDir::new("roundtrip");
    let day_dir = dir.path().join("2026-10-07");
    let state = day_state(
        "2026-10-07",
        vec![saved(
            "580000000000000001",
            "JY0000001234",
            &[("Sepatu", 2)],
        )],
        "1 A Sepatu ×1.pdf",
    );
    state.save(&day_dir).expect("save");

    // The written `state.json` carries the order's searched fields (07 › *`state.json`*).
    let text = fs::read_to_string(day_dir.join("state.json")).expect("read");
    let value: Value = serde_json::from_str(&text).expect("parse");
    let entry = &value["saved"][0];
    assert_eq!(entry["courier"], "J&T Express");
    assert_eq!(entry["total_quantity"], 2);
    assert_eq!(entry["distinct_items"], 1);
    assert_eq!(entry["items"][0]["name"], "Sepatu");
    assert_eq!(entry["items"][0]["variation"], "42");
    assert_eq!(entry["items"][0]["seller_sku"], "SP-0001");
    assert_eq!(entry["items"][0]["quantity"], 2);

    // A reloaded day searches the same way.
    let loaded = DayState::load(&day_dir).expect("load");
    let cond = parse_condition("name contains \"sepatu\" and total_quantity = 2").expect("cond");
    assert_eq!(search(&cond, &[("2026-10-07", &loaded)]).orders, 1);
}
