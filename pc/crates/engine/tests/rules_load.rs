//! `load_rules`: `categories.toml` loading, every validation error and the file-position
//! mapping for condition errors (mirrors the relevant cases of script/tests/test_rules.py).

use std::fs;
use std::path::PathBuf;

use packing_engine::orders::{Line, Order};
use packing_engine::rules::{categorize, format_condition, load_rules};

fn testdata_categories() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../testdata/categories.toml")
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

const VALID_WHEN: &str = "when = 'name contains \"x\"'\n";

fn validation_cases() -> Vec<(&'static str, String, &'static str)> {
    vec![
        ("no categories", "# a comment only\n".to_string(), "categories.toml: no categories"),
        ("empty category array", "category = []\n".to_string(), "categories.toml: no categories"),
        (
            "unknown top-level key",
            format!("title = \"x\"\n[[category]]\ncode = \"A\"\nname = \"N\"\n{VALID_WHEN}"),
            "categories.toml: unknown key \"title\"",
        ),
        (
            "unknown key in category",
            format!("[[category]]\ncode = \"A\"\nname = \"N\"\nfoo = 1\n{VALID_WHEN}"),
            "categories.toml: category \"A\": unknown key \"foo\"",
        ),
        (
            "missing code",
            format!("[[category]]\nname = \"N\"\n{VALID_WHEN}"),
            "categories.toml: category #1: missing \"code\"",
        ),
        (
            "empty code",
            format!("[[category]]\ncode = \"\"\nname = \"N\"\n{VALID_WHEN}"),
            "categories.toml: category #1: \"code\" must not be empty",
        ),
        (
            "code not 1-3 letters/digits",
            format!("[[category]]\ncode = \"ABCD\"\nname = \"N\"\n{VALID_WHEN}"),
            "categories.toml: category \"ABCD\": \"code\" must be 1-3 letters or digits",
        ),
        (
            "duplicate code",
            format!(
                "[[category]]\ncode = \"A\"\nname = \"One\"\n{VALID_WHEN}[[category]]\ncode = \"A\"\nname = \"Two\"\n{VALID_WHEN}"
            ),
            "categories.toml: category \"A\": duplicate \"code\" \"A\"",
        ),
        (
            "missing name",
            format!("[[category]]\ncode = \"A\"\n{VALID_WHEN}"),
            "categories.toml: category \"A\": missing \"name\"",
        ),
        (
            "empty name",
            format!("[[category]]\ncode = \"A\"\nname = \"\"\n{VALID_WHEN}"),
            "categories.toml: category \"A\": \"name\" must not be empty",
        ),
        (
            "when missing on non-last",
            format!(
                "[[category]]\ncode = \"A\"\nname = \"One\"\n[[category]]\ncode = \"B\"\nname = \"Two\"\n{VALID_WHEN}"
            ),
            "categories.toml: category \"A\": missing \"when\" on a category that is not last",
        ),
        (
            "when not a string",
            "[[category]]\ncode = \"A\"\nname = \"N\"\nwhen = 1\n".to_string(),
            "categories.toml: category \"A\": \"when\" must be a string",
        ),
        (
            "condition error",
            "[[category]]\ncode = \"B\"\nname = \"N\"\nwhen = 'name foo \"y\"'\n".to_string(),
            "categories.toml, line 4, col 14: category \"B\" (when): expected a text operator (contains, equals, starts_with) after \"name\"",
        ),
        (
            "condition error line 2",
            "[[category]]\ncode = \"B\"\nname = \"N\"\nwhen = '''name contains \"x\"\nand name foo \"y\"'''\n"
                .to_string(),
            "categories.toml, line 5, col 10: category \"B\" (when): expected a text operator (contains, equals, starts_with) after \"name\"",
        ),
    ]
}

#[test]
fn load_rules_validation() {
    for (name, toml, expected) in validation_cases() {
        let error = load_rules(&toml).expect_err(name);
        assert_eq!(error.message, expected, "case {name}");
    }
}

#[test]
fn load_rules_toml_syntax_error() {
    let error = load_rules("[[category]\n").unwrap_err();
    assert!(
        error.message.starts_with("categories.toml: "),
        "got {error}"
    );
    assert!(error.message.contains("line 1"), "got {error}");
}

#[test]
fn load_rules_time_error_uses_the_file_position() {
    let toml = "[[category]]\ncode = \"A\"\nname = \"One\"\nwhen = 'name contains \"x\"'\n\
                [[category]]\ncode = \"B\"\nname = \"Two\"\nwhen = 'rts_time < \"13:99\"'\n";
    let error = load_rules(toml).unwrap_err();
    assert_eq!(
        error.message,
        "categories.toml, line 8, col 20: category \"B\" (when): expected a date \"YYYY-MM-DD\", \
         a time \"HH:MM\" or a date and time \"YYYY-MM-DD HH:MM\""
    );
}

#[test]
fn load_rules_csv_field_uses_the_file_position() {
    // The app cannot read a CSV: a CSV field inside a `when` is a condition error, and the
    // position is mapped into the file (cf. 07-pc-app).
    let toml =
        "[[category]]\ncode = \"B\"\nname = \"N\"\nwhen = 'product_category contains \"x\"'\n";
    let error = load_rules(toml).unwrap_err();
    assert_eq!(
        error.message,
        "categories.toml, line 4, col 9: category \"B\" (when): field \"product_category\" \
         needs the orders CSV, which the PC app does not read"
    );
}

#[test]
fn load_rules_last_category_may_omit_when() {
    let toml = format!(
        "[[category]]\ncode = \"A\"\nname = \"One\"\n{VALID_WHEN}[[category]]\ncode = \"Z\"\nname = \"Rest\"\n"
    );
    let categories = load_rules(&toml).expect("load");
    assert_eq!(
        categories
            .iter()
            .map(|c| c.code.as_str())
            .collect::<Vec<_>>(),
        ["A", "Z"]
    );
    assert!(categories[0].condition.is_some());
    assert!(categories[0].when.is_some());
    assert!(categories[1].condition.is_none());
    assert!(categories[1].when.is_none());
}

#[test]
fn load_testdata_categories() {
    let text = fs::read_to_string(testdata_categories()).expect("read testdata categories");
    let categories = load_rules(&text).expect("load");
    assert_eq!(
        categories
            .iter()
            .map(|c| c.code.as_str())
            .collect::<Vec<_>>(),
        ["A", "B", "C", "Z"]
    );
    assert_eq!(
        categories
            .iter()
            .map(|c| c.name.as_str())
            .collect::<Vec<_>>(),
        ["Sepatu", "Spion & Knalpot", "Banyak unit", "J&T"]
    );
    let b = categories.iter().find(|c| c.code == "B").expect("B");
    let condition = b.condition.as_ref().expect("B condition");
    assert_eq!(
        format_condition(condition),
        "not name contains \"sepatu\" and (name contains \"spion\" or name contains \"knalpot\")"
    );
    // Round trip through the printer.
    for category in &categories {
        let Some(condition) = &category.condition else {
            continue;
        };
        let text = format_condition(condition);
        let reparsed = packing_engine::rules::parse_condition(&text).expect("re-parse");
        assert_eq!(format_condition(&reparsed), text);
    }
}

#[test]
fn categorize_first_match_and_none() {
    let text = fs::read_to_string(testdata_categories()).expect("read testdata categories");
    let categories = load_rules(&text).expect("load");

    let sepatu = order("Sepatu Standar", 1, "JNE");
    assert_eq!(
        categorize(&sepatu, &categories).map(|c| c.code.as_str()),
        Some("A")
    );

    let spion = order("Spion Beat", 1, "JNE");
    assert_eq!(
        categorize(&spion, &categories).map(|c| c.code.as_str()),
        Some("B")
    );

    let knalpot = order("Cover Knalpot Beat", 1, "JNE");
    assert_eq!(
        categorize(&knalpot, &categories).map(|c| c.code.as_str()),
        Some("B")
    );

    // A sepatu with quantity 3 would also match C, but A comes first.
    let sepatu_many = order("Sepatu Standar", 3, "JNE");
    assert_eq!(
        categorize(&sepatu_many, &categories).map(|c| c.code.as_str()),
        Some("A")
    );

    // Matches no entry.
    let other = order("Busi Iridium", 1, "JNE");
    assert!(categorize(&other, &categories).is_none());

    // Only the last entry (courier J&T) matches.
    let jnt = order("Busi Iridium", 1, "J&T Express");
    assert_eq!(
        categorize(&jnt, &categories).map(|c| c.code.as_str()),
        Some("Z")
    );

    // The returned reference is the category itself.
    assert_eq!(categorize(&spion, &categories), Some(&categories[1]));
}
