//! Editing `categories.toml` through `RulesDocument` (08 › 5; task 2.12): comments and unchanged
//! condition texts survive a save, a changed condition is written canonically, add/move/delete
//! give the expected order, and the result loads with `load_rules`.

use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use packing_engine::rules::format_condition;
use packing_engine::rules_edit::RulesDocument;

fn testdata_categories() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../testdata/categories.toml")
}

fn doc(text: &str) -> RulesDocument {
    RulesDocument::from_str(text).expect("parse document")
}

fn codes(document: &RulesDocument) -> Vec<String> {
    document.entries().into_iter().map(|e| e.code).collect()
}

fn loaded_codes(document: &RulesDocument) -> Vec<String> {
    document
        .validate()
        .expect("load_rules")
        .into_iter()
        .map(|c| c.code)
        .collect()
}

const SIMPLE: &str = "\
# Picks. Order matters: an order is taken by the first entry whose condition matches.
[[category]]
code = \"A\"
name = \"One\"
when = 'name contains \"x\"'

[[category]]
code = \"B\"
name = \"Two\"
when = 'name contains \"y\"'

[[category]]
code = \"Z\"
name = \"Rest\"
";

#[test]
fn comments_and_unchanged_texts_survive() {
    let text = fs::read_to_string(testdata_categories()).expect("read testdata");
    let before = doc(&text);
    assert_eq!(
        before
            .entries()
            .iter()
            .map(|e| e.code.as_str())
            .collect::<Vec<_>>(),
        ["A", "B", "C", "Z"]
    );
    // The B entry's condition spans several lines.
    let b_when = before.entries()[1].when.clone().expect("B when");
    assert!(b_when.contains('\n'), "B when is multi-line: {b_when:?}");

    // Change entry A's name only.
    let mut edited = doc(&text);
    edited.set_name(0, "Sepatu Baru").expect("set name");
    let out = edited.to_string();

    // The file comment is still there.
    assert!(
        out.contains("# Packing categories for the test data."),
        "comment kept:\n{out}"
    );
    // B's condition text is byte-for-byte the same.
    assert_eq!(edited.entries()[1].when.as_deref(), Some(b_when.as_str()));
    assert!(out.contains(&b_when), "B when kept byte-for-byte:\n{out}");
    // The change landed and the whole file still loads.
    assert_eq!(edited.entries()[0].name, "Sepatu Baru");
    assert_eq!(
        edited
            .validate()
            .expect("load_rules")
            .iter()
            .map(|c| c.name.as_str())
            .collect::<Vec<_>>(),
        ["Sepatu Baru", "Spion & Knalpot", "Banyak unit", "J&T"]
    );
}

#[test]
fn unchanged_condition_keeps_original_text() {
    let text = fs::read_to_string(testdata_categories()).expect("read testdata");
    let mut edited = doc(&text);
    let original = edited.entries()[1].when.clone().expect("B when");

    // A different spelling with the same canonical form changes nothing in the file.
    edited
        .set_condition(
            1,
            Some("not(name contains \"sepatu\") and (name contains \"spion\" or name contains \"knalpot\")"),
        )
        .expect("set condition");
    assert_eq!(
        edited.entries()[1].when.as_deref(),
        Some(original.as_str()),
        "unchanged condition keeps the original text, line breaks included"
    );
}

#[test]
fn changed_condition_written_canonically() {
    let mut edited = doc(SIMPLE);
    edited
        .set_condition(0, Some("name   starts_with   \"sepatu\""))
        .expect("set condition");

    // The stored text is the printer's one-line canonical form.
    assert_eq!(
        edited.entries()[0].when.as_deref(),
        Some("name starts_with \"sepatu\"")
    );
    // Written as a TOML literal string like the file format (04), so quotes need no escaping.
    assert!(
        edited
            .to_string()
            .contains("when = 'name starts_with \"sepatu\"'"),
        "canonical one-line text:\n{edited}"
    );
    let categories = edited.validate().expect("load_rules");
    assert_eq!(
        format_condition(categories[0].condition.as_ref().expect("condition")),
        "name starts_with \"sepatu\""
    );
}

#[test]
fn add_move_delete_keep_the_order() {
    let mut edited = doc(SIMPLE);
    assert_eq!(codes(&edited), ["A", "B", "Z"].map(String::from));

    // Add above the rest entry (08 › 5: the last entry has no condition).
    edited
        .add(2, "X", "Extra", Some("name contains \"z\""))
        .expect("add");
    assert_eq!(codes(&edited), ["A", "B", "X", "Z"].map(String::from));
    assert!(edited.to_string().contains("[[category]]"));

    // Move X up; the move must survive a save (validate reads the written text).
    edited.move_to(2, 1).expect("move");
    assert_eq!(codes(&edited), ["A", "X", "B", "Z"].map(String::from));
    assert_eq!(
        loaded_codes(&edited),
        ["A", "X", "B", "Z"].map(String::from)
    );

    // Move X back down.
    edited.move_to(1, 2).expect("move");
    assert_eq!(
        loaded_codes(&edited),
        ["A", "B", "X", "Z"].map(String::from)
    );

    // Delete X.
    edited.remove(2).expect("remove");
    assert_eq!(loaded_codes(&edited), ["A", "B", "Z"].map(String::from));
}

#[test]
fn clear_the_last_condition_makes_a_rest_entry() {
    let mut edited = doc(SIMPLE);
    edited
        .set_condition(2, Some("courier starts_with \"J&T\""))
        .expect("set Z");
    assert!(edited.entries()[2].when.is_some());

    edited.set_condition(2, None).expect("clear Z");
    assert!(edited.entries()[2].when.is_none());
    let categories = edited.validate().expect("load_rules");
    assert!(categories[2].condition.is_none());
}

#[test]
fn invalid_edits_are_errors() {
    let mut edited = doc(SIMPLE);

    // A bad condition text is rejected and nothing changes.
    let err = edited.set_condition(0, Some("name foo \"x\"")).unwrap_err();
    assert_eq!(
        err.message,
        "line 1, col 6: expected a text operator (contains, equals, starts_with) after \"name\""
    );
    assert_eq!(
        edited.entries()[0].when.as_deref(),
        Some("name contains \"x\"")
    );

    // An index past the end.
    assert!(edited.set_name(9, "x").is_err());
    assert!(edited.remove(9).is_err());
    assert!(edited.move_to(0, 9).is_err());
}
