//! The boxes of a filter: text → boxes → text gives the printer's form, nested boxes of one kind
//! merge, and the filter grammar decides what a box may hold.

use packing_engine::rules_edit::{GroupKind, Node, RowValue, filter_to_tree, tree_to_filter};

#[test]
fn a_filter_goes_to_boxes_and_back_in_the_printers_form() {
    for (text, printed) in [
        ("name contains \"sepatu\"", "name contains \"sepatu\""),
        (
            "not name contains \"sepatu\"\nand (name contains \"spion\" or name contains \"knalpot\")",
            "not name contains \"sepatu\" and (name contains \"spion\" or name contains \"knalpot\")",
        ),
        ("total_quantity >= 2", "total_quantity >= 2"),
        ("ship_by < \"2026-10-07\"", "ship_by < \"2026-10-07\""),
        ("courier equals \"jnt\"", "courier equals \"jnt\""),
        ("courier IS_EMPTY", "courier is_empty"),
        (
            "not tracking_id is_empty and name contains \"a\"",
            "not tracking_id is_empty and name contains \"a\"",
        ),
    ] {
        let tree = filter_to_tree(text).unwrap_or_else(|error| panic!("{text}: {error}"));
        assert_eq!(tree_to_filter(&tree).unwrap(), printed, "{text}");
    }
}

#[test]
fn nested_boxes_of_one_kind_merge_and_the_grammar_is_the_filters() {
    let tree =
        filter_to_tree("name contains \"a\" and (name contains \"b\" and name contains \"c\")")
            .unwrap();
    assert_eq!(tree.root.kind, GroupKind::All);
    assert_eq!(tree.root.children.len(), 3);
    assert!(
        tree.root
            .children
            .iter()
            .all(|node| matches!(node, Node::Row(_)))
    );

    assert!(
        filter_to_tree("ship_by < \"2026-10-07 17:00\"").is_err(),
        "a clock time is not a date"
    );
    assert!(
        filter_to_tree("paid_time < \"2026-10-07\"").is_err(),
        "a CSV field is not a filter field"
    );
    let mut empty = filter_to_tree("name contains \"a\"").unwrap();
    empty.root.children.clear();
    assert!(tree_to_filter(&empty).is_err(), "an empty box is an error");
}

#[test]
fn is_empty_is_a_row_without_a_value() {
    let tree = filter_to_tree("not courier is_empty").unwrap();
    let Node::Row(row) = &tree.root.children[0] else {
        panic!("a row")
    };
    assert!(row.not);
    assert_eq!(row.operator, "is_empty");
    assert_eq!(row.value, RowValue::NoValue);

    let mut needs_value = tree.clone();
    if let Node::Row(row) = &mut needs_value.root.children[0] {
        row.operator = "equals".to_string();
    }
    assert!(
        tree_to_filter(&needs_value).is_err(),
        "equals needs a value"
    );
}
