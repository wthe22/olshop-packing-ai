//! The boxes editor's condition text <-> tree round trip over `testdata/conditions.json`, the
//! tree shape, and the live counts (08 › 5. Categories; task 2.12).
//!
//! Mirrors the skip rules of `tests/rules_conditions.rs`: cases with `app_fields: true` are
//! skipped and a case whose text uses a CSV-only field gives the "needs the orders CSV" error.

use std::fs;
use std::path::PathBuf;

use packing_engine::orders::{Line, Order};
use packing_engine::rules::{Category, parse_condition};
use packing_engine::rules_edit::{GroupKind, Node, condition_to_tree, counts, tree_to_condition};
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

/// The first CSV-only field name appearing in the condition text, with its 1-based line/col.
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
fn conditions_round_trip_through_boxes() {
    let text = fs::read_to_string(fixture_path()).expect("read conditions.json");
    let root: Value = serde_json::from_str(&text).expect("parse conditions.json");
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
        match condition_to_tree(text) {
            Ok(tree) => {
                assert!(
                    case.get("error").is_none(),
                    "case {index} ({text:?}) gave boxes but the fixture expects an error"
                );
                let canonical = case["canonical"].as_str().expect("canonical");
                assert_eq!(
                    tree_to_condition(&tree).expect("tree to text"),
                    canonical,
                    "case {index} ({text:?}) text -> boxes -> text"
                );
                // The canonical text itself goes through the same path.
                let again = condition_to_tree(canonical).expect("boxes of the canonical text");
                assert_eq!(
                    tree_to_condition(&again).expect("tree to text"),
                    canonical,
                    "case {index} ({text:?}) canonical -> boxes -> text"
                );
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
        "conditions.json boxes round trip: {} cases -> {ok} text->boxes->text, {csv} need the \
         orders CSV, {errors} error texts, {skipped} skipped (app_fields)",
        cases.len()
    );
    assert!(
        ok > 0 && csv > 0 && errors > 0 && skipped > 0,
        "each kind should appear"
    );
}

fn single_row(text: &str) -> (GroupKind, bool, usize, bool) {
    let tree = condition_to_tree(text).expect("boxes");
    let rows = tree
        .root
        .children
        .iter()
        .filter(|node| matches!(node, Node::Row(_)))
        .count();
    (
        tree.root.kind,
        tree.root.not,
        rows,
        tree.root.children.len() == rows,
    )
}

#[test]
fn tree_shape() {
    // A single comparison is an "All of these" group with one row.
    let (kind, not, rows, all_rows) = single_row("name contains \"spion\"");
    assert_eq!(
        (kind, not, rows, all_rows),
        (GroupKind::All, false, 1, true)
    );

    // `not` on a comparison is the row's flag, not a group's.
    let tree = condition_to_tree("not name contains \"spion\"").expect("boxes");
    assert!(!tree.root.not);
    let Node::Row(row) = &tree.root.children[0] else {
        panic!("expected a row");
    };
    assert!(row.not);

    // Nested same-kind groups stay as one group with three rows (`a and (b and c)`); the parser
    // flattens them, so the tree is one "All of these" with three rows.
    let (kind, _, rows, all_rows) =
        single_row("name contains \"a\" and (name contains \"b\" and name contains \"c\")");
    assert_eq!((kind, rows, all_rows), (GroupKind::All, 3, true));

    // `(a or b) and c`: an "All of these" group holding an "Any of these" group and a row.
    let tree =
        condition_to_tree("(name contains \"a\" or name contains \"b\") and total_quantity >= 3")
            .expect("boxes");
    assert_eq!(tree.root.kind, GroupKind::All);
    assert!(matches!(&tree.root.children[0], Node::Group(g) if g.kind == GroupKind::Any));
    assert!(matches!(&tree.root.children[1], Node::Row(_)));

    // `not (a and b)`: an "All of these" group with `not`.
    let tree =
        condition_to_tree("not (name contains \"a\" and name contains \"b\")").expect("boxes");
    assert_eq!(tree.root.kind, GroupKind::All);
    assert!(tree.root.not);

    // `not not a` cannot fold onto a row's single flag: the group carries the outer `not` and
    // the row carries the inner one.
    let tree = condition_to_tree("not not name contains \"spion\"").expect("boxes");
    assert_eq!(tree.root.kind, GroupKind::All);
    assert!(tree.root.not);
    assert!(matches!(&tree.root.children[0], Node::Row(r) if r.not));
    assert_eq!(
        tree_to_condition(&tree).expect("text"),
        "not not name contains \"spion\""
    );
}

#[test]
fn empty_group_and_bad_row_are_errors() {
    let tree = condition_to_tree("name contains \"x\"").expect("boxes");
    let mut empty = tree;
    empty.root.children.clear();
    assert_eq!(
        tree_to_condition(&empty).unwrap_err().message,
        "this group is empty"
    );

    // A hand-built row with an unknown field is caught by re-parsing the printer's text.
    let mut bad = condition_to_tree("name contains \"x\"").expect("boxes");
    if let Node::Row(row) = &mut bad.root.children[0] {
        row.field = "foo".to_string();
    }
    assert_eq!(
        tree_to_condition(&bad).unwrap_err().message,
        "line 1, col 1: unknown field \"foo\""
    );
}

fn line(name: &str, quantity: i64) -> Line {
    Line {
        name: name.to_string(),
        variation: String::new(),
        seller_sku: String::new(),
        quantity,
        display_name: name.to_string(),
    }
}

fn order(name: &str, quantity: i64, courier: &str) -> Order {
    Order {
        order_id: "580000000000000999".to_string(),
        tracking_id: "JY0000009999".to_string(),
        courier: courier.to_string(),
        ship_by: None,
        lines: vec![line(name, quantity)],
        customer_message: String::new(),
    }
}

fn category(code: &str, when: Option<&str>) -> Category {
    Category {
        code: code.to_string(),
        name: code.to_string(),
        condition: when.map(|t| parse_condition(t).expect("parse category")),
        when: when.map(str::to_string),
    }
}

#[test]
fn counts_take_from_what_is_left() {
    let categories = vec![
        category("A", Some("name contains \"sepatu\"")),
        category("B", Some("name contains \"spion\"")),
        category("Z", None), // the rest: no condition
    ];
    let orders = vec![
        order("Sepatu Standar", 1, "JNE"), // A
        order("Sepatu Standar", 1, "JNE"), // A
        order("Spion Beat", 1, "JNE"),     // B
        order("Busi Iridium", 1, "JNE"),   // the rest
        order("Knalpot", 1, "JNE"),        // the rest
    ];
    assert_eq!(counts(&categories, &orders), vec![2, 1, 2]);

    // Without a rest entry, the leftovers are counted nowhere.
    let no_rest = vec![
        category("A", Some("name contains \"sepatu\"")),
        category("B", Some("name contains \"spion\"")),
    ];
    assert_eq!(counts(&no_rest, &orders), vec![2, 1]);
}
