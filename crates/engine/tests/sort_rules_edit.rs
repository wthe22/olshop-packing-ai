//! The Sort rules screen's edits of `packing-workflows.toml`: the owner's comments stay, an
//! unchanged filter keeps its text, the groups are written in the screen's order, and an edit that
//! fails a check changes nothing.

use packing_engine::workflows::{
    BUNDLED_WORKFLOWS, EditError, GroupEdit, NewRule, Workflows, add_rule, delete_rule,
    load_workflows, rename_rule, write_groups,
};

const HARIAN: &str = r#"
# The rule for the usual days.
[[label_sort_rules]]
name = "harian"

# Shoes first: they go out with the first pickup.
[[label_sort_rules.label_groups]]
code = "A"
name = "Sepatu"
filter = 'name contains "sepatu"'

[[label_sort_rules.label_groups]]
code = "B"
name = "Spion & Knalpot"
filter = '''
not name contains "sepatu"
and (name contains "spion" or name contains "knalpot")
'''

[[label_sort_rules.label_groups]]
code = "Z"
name = "Lainnya"
"#;

fn file() -> String {
    format!("{BUNDLED_WORKFLOWS}{HARIAN}")
}

fn group(code: &str, name: &str, filter: Option<&str>, origin: Option<usize>) -> GroupEdit {
    GroupEdit {
        code: code.into(),
        name: name.into(),
        filter: filter.map(Into::into),
        origin,
    }
}

fn codes(workflows: &Workflows, rule: &str) -> Vec<String> {
    let rule = workflows.sort_rule(rule).unwrap();
    rule.groups.iter().map(|group| group.code.clone()).collect()
}

#[test]
fn a_rules_groups_are_written_in_order_with_comments_and_unchanged_filters_kept() {
    let text = file();
    let b_filter = load_workflows(&text)
        .unwrap()
        .sort_rule("harian")
        .unwrap()
        .groups[1]
        .filter
        .clone()
        .unwrap()
        .text;
    let groups = [
        group("B", "Spion", Some(&b_filter), Some(1)),
        group(
            "A",
            "Sepatu",
            Some("name contains \"sepatu\" or name contains \"sandal\""),
            Some(0),
        ),
        group("C", "Jok", Some("name contains \"jok\""), None),
        group("Z", "Lainnya", None, Some(2)),
    ];
    let (written, workflows) = write_groups(&text, "harian", &groups).unwrap();
    assert_eq!(codes(&workflows, "harian"), ["B", "A", "C", "Z"]);
    assert_eq!(
        codes(&workflows, "default"),
        ["Z"],
        "another rule is untouched"
    );
    assert!(written.contains("# The rule for the usual days."));
    assert!(written.contains("# Shoes first: they go out with the first pickup.\n[[label_sort_rules.label_groups]]\ncode = \"A\""), "{written}");
    assert!(
        written.contains("filter = '''\nnot name contains \"sepatu\"\nand (name contains"),
        "an unchanged filter keeps its lines"
    );
    assert!(
        written.contains("filter = 'name contains \"sepatu\" or name contains \"sandal\"'"),
        "a changed one is one literal line"
    );
    assert!(written.contains("name = \"Spion\""));
    assert!(
        written.find("code = \"B\"").unwrap() < written.find("code = \"A\"").unwrap(),
        "{written}"
    );
    let as_written: Vec<GroupEdit> = groups
        .iter()
        .enumerate()
        .map(|(at, group)| GroupEdit {
            origin: Some(at),
            ..group.clone()
        })
        .collect();
    let (again, _) = write_groups(&written, "harian", &as_written).unwrap();
    assert_eq!(
        again, written,
        "writing the same groups again changes nothing"
    );
}

#[test]
fn a_rule_is_added_renamed_and_deleted_and_the_last_one_stays() {
    let text = file();
    let (added, workflows) = add_rule(&text, "ramadhan", NewRule::Empty).unwrap();
    assert_eq!(codes(&workflows, "ramadhan"), ["Z"]);
    let (copied, workflows) = add_rule(&added, "lebaran", NewRule::CopyOf("harian")).unwrap();
    assert_eq!(codes(&workflows, "lebaran"), ["A", "B", "Z"]);
    assert!(
        copied.contains("# The rule for the usual days."),
        "the copy leaves the original's comment where it is"
    );
    let (renamed, workflows) = rename_rule(&copied, "lebaran", "idul fitri").unwrap();
    assert!(workflows.sort_rule("lebaran").is_none());
    assert_eq!(codes(&workflows, "idul fitri"), ["A", "B", "Z"]);
    let (deleted, workflows) = delete_rule(&renamed, "ramadhan").unwrap();
    assert!(workflows.sort_rule("ramadhan").is_none());
    assert!(deleted.contains("# The rule for the usual days."));

    assert_eq!(
        delete_rule(BUNDLED_WORKFLOWS, "default").unwrap_err(),
        EditError::LastRule
    );
    assert_eq!(
        rename_rule(&text, "minggu", "x").unwrap_err(),
        EditError::NoRule("minggu".into())
    );
    match add_rule(&text, "harian", NewRule::Empty).unwrap_err() {
        EditError::Check(error) => assert!(error.message.contains("a name of its own"), "{error}"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn an_edit_that_fails_a_check_names_its_place_and_returns_nothing() {
    let text = file();
    let rest_not_last = [
        group("Z", "Lainnya", None, Some(2)),
        group("A", "Sepatu", Some("name contains \"sepatu\""), Some(0)),
    ];
    match write_groups(&text, "harian", &rest_not_last).unwrap_err() {
        EditError::Check(error) => {
            assert!(
                error.message.contains("no filter but is not the last"),
                "{error}"
            );
            assert!(error.line > 0, "{error}");
        }
        other => panic!("{other:?}"),
    }
    let bad_filter = [
        group("A", "Sepatu", Some("name has \"x\""), Some(0)),
        group("Z", "Lainnya", None, Some(2)),
    ];
    assert!(matches!(
        write_groups(&text, "harian", &bad_filter),
        Err(EditError::Check(_))
    ));
}
