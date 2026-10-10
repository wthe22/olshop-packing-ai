//! `testdata/conditions.json`: every case through the Rust condition parser, printer and
//! evaluator (mirrors script/tests/test_rules.py::test_conditions_fixture).
//!
//! The app reads no orders CSV, so a case whose text uses a CSV-only field expects the
//! "needs the orders CSV" error instead of Python's result. Cases with `app_fields: true`
//! (category/batch/group, scan filters only) are skipped and counted.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

use packing_engine::orders::{Line, Order};
use packing_engine::rules::{evaluate, fields_used, format_condition, parse_condition};
use serde_json::Value;

const CSV_FIELDS: &[&str] = &[
    "sku_id",
    "product_category",
    "channel",
    "paid_time",
    "rts_time",
    "created_time",
];

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../testdata/conditions.json")
}

fn parse_ship_by(value: &Value) -> Option<jiff::civil::DateTime> {
    value.as_str().map(|s| s.parse().expect("ISO date-time"))
}

fn build_order(value: &Value) -> Order {
    let lines = value["lines"]
        .as_array()
        .expect("lines array")
        .iter()
        .map(|line| Line {
            name: line["name"].as_str().unwrap_or_default().to_string(),
            variation: line["variation"].as_str().unwrap_or_default().to_string(),
            seller_sku: line["seller_sku"].as_str().unwrap_or_default().to_string(),
            quantity: line["quantity"].as_i64().expect("quantity"),
            display_name: line["display_name"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
        })
        .collect();
    Order {
        order_id: value["order_id"].as_str().unwrap_or_default().to_string(),
        tracking_id: value["tracking_id"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        courier: value["courier"].as_str().unwrap_or_default().to_string(),
        ship_by: parse_ship_by(&value["ship_by"]),
        lines,
        customer_message: String::new(),
    }
}

/// The first CSV-only field name appearing in the condition text, with its 1-based line/col
/// (the same position rule the parser uses: the field token's start).
fn csv_field(text: &str) -> Option<(String, usize, usize)> {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();
    let mut i = 0;
    let mut line = 1;
    let mut col = 1;
    while i < n {
        let c = chars[i];
        if c == '\n' {
            line += 1;
            col = 1;
            i += 1;
            continue;
        }
        if !(c.is_ascii_alphanumeric() || c == '_') {
            col += 1;
            i += 1;
            continue;
        }
        let (start_line, start_col) = (line, col);
        let mut word = String::new();
        while i < n && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
            word.push(chars[i]);
            i += 1;
            col += 1;
        }
        if CSV_FIELDS.contains(&word.to_lowercase().as_str()) {
            return Some((word.to_lowercase(), start_line, start_col));
        }
    }
    None
}

#[test]
fn conditions_fixture() {
    let text = fs::read_to_string(fixture_path()).expect("read conditions.json");
    let root: Value = serde_json::from_str(&text).expect("parse conditions.json");

    let mut orders: BTreeMap<String, Order> = BTreeMap::new();
    for (id, value) in root["orders"].as_object().expect("orders object") {
        orders.insert(id.clone(), build_order(value));
    }

    let empty: Vec<Value> = Vec::new();
    let cases = root["cases"].as_array().unwrap_or(&empty);

    let mut ok = 0usize;
    let mut csv = 0usize;
    let mut errors = 0usize;
    let mut skipped = 0usize;

    for (index, case) in cases.iter().enumerate() {
        let text = case["text"].as_str().expect("case text");
        if case
            .get("app_fields")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            skipped += 1;
            continue;
        }
        match parse_condition(text) {
            Ok(cond) => {
                assert!(
                    case.get("error").is_none(),
                    "case {index} ({text:?}) parsed but the fixture expects an error"
                );
                let canonical = case["canonical"].as_str().expect("canonical");
                assert_eq!(
                    format_condition(&cond),
                    canonical,
                    "case {index} ({text:?}) printer"
                );
                let reparsed = parse_condition(canonical).expect("re-parse canonical");
                assert_eq!(reparsed, cond, "case {index} ({text:?}) round trip");
                for (order_id, expected) in case["values"].as_object().expect("values") {
                    let order = orders.get(order_id).expect("order id");
                    assert_eq!(
                        evaluate(&cond, order),
                        expected.as_bool().expect("bool"),
                        "case {index} ({text:?}) order {order_id}"
                    );
                }
                if let Some(fields) = case.get("fields") {
                    let want: BTreeSet<String> = fields
                        .as_array()
                        .expect("fields array")
                        .iter()
                        .map(|f| f.as_str().expect("field name").to_string())
                        .collect();
                    assert_eq!(fields_used(&cond), want, "case {index} ({text:?}) fields");
                }
                ok += 1;
            }
            Err(e) => {
                if let Some(expected) = case.get("error").and_then(Value::as_str) {
                    assert_eq!(e.message, expected, "case {index} ({text:?}) error text");
                    errors += 1;
                } else {
                    let (field, line, col) = csv_field(text)
                        .unwrap_or_else(|| panic!("case {index} ({text:?}): unexpected error {e}"));
                    let expected = format!(
                        "line {line}, col {col}: field \"{field}\" needs the orders CSV, which the PC app does not read"
                    );
                    assert_eq!(e.message, expected, "case {index} ({text:?}) csv error");
                    csv += 1;
                }
            }
        }
    }

    println!(
        "conditions.json: {} cases -> {ok} parsed/evaluated, {csv} need the orders CSV, \
         {errors} error texts, {skipped} skipped (app_fields)",
        cases.len()
    );
    assert!(
        ok > 0 && csv > 0 && errors > 0 && skipped > 0,
        "each kind should appear"
    );
}

#[test]
fn fixture_shape() {
    let text = fs::read_to_string(fixture_path()).expect("read conditions.json");
    let root: Value = serde_json::from_str(&text).expect("parse conditions.json");
    assert!(
        root["about"]
            .as_str()
            .map(|s| !s.is_empty())
            .unwrap_or(false)
    );
    assert!(
        root["orders"]
            .as_object()
            .map(|o| o.len() >= 2)
            .unwrap_or(false)
    );
    let cases = root["cases"].as_array().expect("cases array");
    let orders = root["orders"].as_object().expect("orders object");
    for case in cases {
        assert!(case.get("text").is_some(), "every case has text");
        let has_error = case.get("error").is_some();
        let has_value = case.get("canonical").is_some() && case.get("values").is_some();
        assert!(has_error ^ has_value, "error xor (canonical and values)");
        if let Some(values) = case.get("values").and_then(Value::as_object) {
            for key in values.keys() {
                assert!(orders.contains_key(key), "value order id {key} exists");
            }
        }
    }
}

#[test]
fn fields_used_collects_every_field() {
    let cond = parse_condition(
        "ship_by < \"2026-10-07 17:00\" and (name contains \"x\" or not courier starts_with \"J&T\")",
    )
    .expect("parse");
    assert_eq!(
        fields_used(&cond),
        BTreeSet::from([
            "courier".to_string(),
            "name".to_string(),
            "ship_by".to_string()
        ])
    );
    assert_eq!(
        fields_used(&parse_condition("total_quantity = 1").expect("parse")),
        BTreeSet::from(["total_quantity".to_string()])
    );
    assert_eq!(
        fields_used(&parse_condition("not tracking_id starts_with \"JY\"").expect("parse")),
        BTreeSet::from(["tracking_id".to_string()])
    );
}

#[test]
fn app_and_csv_fields_are_rejected() {
    // App scan-filter fields keep the 04 error.
    assert_eq!(
        parse_condition("category equals \"A\"")
            .unwrap_err()
            .message,
        "line 1, col 1: field \"category\" is allowed in app scan filters only"
    );
    assert_eq!(
        parse_condition("batch = 1").unwrap_err().message,
        "line 1, col 1: field \"batch\" is allowed in app scan filters only"
    );
    // CSV-only fields get the PC-app error with the position of the field name.
    assert_eq!(
        parse_condition("product_category contains \"x\"")
            .unwrap_err()
            .message,
        "line 1, col 1: field \"product_category\" needs the orders CSV, which the PC app does not read"
    );
    assert_eq!(
        parse_condition("  paid_time < \"14:00\"")
            .unwrap_err()
            .message,
        "line 1, col 3: field \"paid_time\" needs the orders CSV, which the PC app does not read"
    );
}
