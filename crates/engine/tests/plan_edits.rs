//! `plan.rs`: the plan edits (skip, move, add, rename), picks from what is left, runs and
//! numbering (07 › *Plan*; mirrors the relevant cases of tools/analysis/tests/test_picks.py).

use packing_engine::ids::{OrderId, PdfNumber, TrackingId};
use packing_engine::orders::{Line, Order, PageRef, ReadOrder, display_name};
use packing_engine::plan::{Plan, PlanError, REST_CODE, REST_NAME};
use packing_engine::rules::{Category, catch_all_category, parse_condition};

fn line(name: &str, quantity: i64, variation: &str) -> Line {
    Line {
        display_name: display_name(name, variation),
        name: name.to_string(),
        variation: variation.to_string(),
        seller_sku: String::new(),
        quantity,
    }
}

fn order(id: &str, lines: Vec<Line>, courier: Option<&str>) -> ReadOrder {
    ReadOrder {
        order: Order {
            order_id: OrderId::new(id),
            tracking_id: Some(TrackingId::new(format!("JY{id}"))),
            courier: courier.map(str::to_string),
            ship_by: None,
            lines,
            customer_message: String::new(),
        },
        pages: vec![PageRef { file: 0, page: 0 }],
        position: 1,
        has_slip: true,
        qty_total: None,
    }
}

fn category(code: &str, name: &str, when: Option<&str>) -> Category {
    Category {
        code: code.to_string(),
        name: name.to_string(),
        condition: when.map(|text| parse_condition(text).expect("parse")),
        when: when.map(str::to_string),
    }
}

/// Two sepatu, one spion, one other (courier J&T) — enough to tell the entries apart.
fn sample_orders() -> Vec<ReadOrder> {
    vec![
        order(
            "580000000000000001",
            vec![line("Sepatu A", 1, "")],
            Some("J&T Express"),
        ),
        order(
            "580000000000000002",
            vec![line("Sepatu B", 1, "")],
            Some("J&T Express"),
        ),
        order(
            "580000000000000003",
            vec![line("Spion C", 1, "")],
            Some("SiCepat"),
        ),
        order(
            "580000000000000004",
            vec![line("Lainnya D", 1, "")],
            Some("J&T Express"),
        ),
    ]
}

fn base_categories() -> Vec<Category> {
    vec![
        category("A", "Sepatu", Some("name contains \"sepatu\"")),
        category("B", "Spion", Some("name contains \"spion\"")),
    ]
}

#[test]
fn count_takes_from_what_is_left_then_the_rest() {
    let orders = sample_orders();
    let plan = Plan::from_categories(&base_categories());
    let result = plan.count(&orders, PdfNumber::new(1));

    assert_eq!(result.categories.len(), 3);
    assert_eq!(
        result
            .categories
            .iter()
            .map(|p| p.code.as_str())
            .collect::<Vec<_>>(),
        ["A", "B", REST_CODE]
    );
    assert_eq!(
        result.categories[0].orders,
        [
            OrderId::new("580000000000000001"),
            OrderId::new("580000000000000002"),
        ]
    );
    assert_eq!(
        result.categories[1].orders,
        [OrderId::new("580000000000000003")]
    );
    assert_eq!(result.total_orders(), orders.len());
}

#[test]
fn a_zero_order_entry_is_skipped_and_gets_no_number() {
    let orders = sample_orders();
    let categories = vec![
        category("B", "Spion", Some("name contains \"spion\"")),
        category("N", "Never", Some("name contains \"absent\"")),
        category("Z", "Rest", None),
    ];
    let result = Plan::from_categories(&categories).count(&orders, PdfNumber::new(4));

    assert_eq!(
        result
            .categories
            .iter()
            .map(|p| p.code.as_str())
            .collect::<Vec<_>>(),
        ["B", "Z"]
    );
    assert_eq!(
        result
            .categories
            .iter()
            .map(|p| p.number)
            .collect::<Vec<_>>(),
        [PdfNumber::new(4), PdfNumber::new(5)]
    );
}

#[test]
fn the_last_entry_without_a_condition_takes_the_rest() {
    let orders = sample_orders();
    let categories = vec![
        category("A", "Sepatu", Some("name contains \"sepatu\"")),
        category("Z", "Lainnya", None),
    ];
    let result = Plan::from_categories(&categories).count(&orders, PdfNumber::new(1));
    assert_eq!(result.categories.len(), 2);
    assert_eq!(result.categories[1].code, "Z");
    assert_eq!(result.categories[1].orders.len(), 2);
}

/// A folder without `categories.toml` uses the one catch-all entry: every order goes into one
/// PDF set, and the plan is the unconfigured state (07 › *Inputs*, Q-UI-4).
#[test]
fn the_catch_all_entry_takes_every_order_and_is_the_unconfigured_plan() {
    let orders = sample_orders();
    let plan = Plan::from_categories(&[catch_all_category()]);
    assert!(plan.unconfigured());

    let result = plan.count(&orders, PdfNumber::new(1));
    assert_eq!(result.categories.len(), 1);
    assert_eq!(result.categories[0].code, "Z");
    assert_eq!(result.categories[0].orders.len(), orders.len());
    assert_eq!(result.categories[0].file_name, "1 Z Lainnya ×4.pdf");

    // One entry with a condition, and two entries, are configured picks; the one entry without
    // a condition is the unconfigured state, the catch-all entry or one written by hand.
    assert!(
        !Plan::from_categories(&[category("A", "Sepatu", Some("name contains \"sepatu\""))])
            .unconfigured()
    );
    assert!(!Plan::from_categories(&base_categories()).unconfigured());
    assert!(Plan::from_categories(&[category("Z", "Lainnya", None)]).unconfigured());
}

#[test]
fn numbering_continues_from_the_first_number() {
    let orders = sample_orders();
    let result = Plan::from_categories(&base_categories()).count(&orders, PdfNumber::new(7));
    assert_eq!(result.categories[0].number, PdfNumber::new(7));
    assert_eq!(result.categories[1].number, PdfNumber::new(8));
    assert_eq!(result.categories[2].number, PdfNumber::new(9));
}

#[test]
fn skip_lets_the_orders_fall_to_the_rows_below() {
    let orders = sample_orders();
    let mut plan = Plan::from_categories(&base_categories());
    plan.skip(0).unwrap();
    let result = plan.count(&orders, PdfNumber::new(1));

    // A's sepatu are not taken by A (skipped); B still needs spion, so sepatu fall to the rest.
    assert_eq!(result.categories.len(), 2);
    assert_eq!(result.categories[0].code, "B");
    assert_eq!(result.categories[1].code, REST_CODE);
    assert_eq!(result.categories[1].orders.len(), 3);

    // Unskip restores the first result.
    plan.unskip(0).unwrap();
    let restored = plan.count(&orders, PdfNumber::new(1));
    assert_eq!(restored.categories[0].orders.len(), 2);
}

#[test]
fn move_changes_which_entry_takes_first() {
    let orders = sample_orders();
    let categories = vec![
        category("A", "Sepatu", Some("name contains \"sepatu\"")),
        category("J", "J&T", Some("courier starts_with \"J&T\"")),
    ];
    let mut plan = Plan::from_categories(&categories);
    // A is first: 2 sepatu, then J&T takes Lainnya D (spion C is not J&T).
    assert_eq!(
        plan.count(&orders, PdfNumber::new(1)).categories[0].code,
        "A"
    );

    plan.move_up(1).unwrap();
    let result = plan.count(&orders, PdfNumber::new(1));
    // J&T is first now and takes all three J&T orders (both sepatu included), so A is empty and
    // the SiCepat spion falls to the rest.
    assert_eq!(result.categories[0].code, "J");
    assert_eq!(result.categories[0].orders.len(), 3);
    assert_eq!(result.categories[1].code, REST_CODE);
    assert_eq!(
        result.categories[1].orders,
        [OrderId::new("580000000000000003")]
    );
}

#[test]
fn move_to_places_a_row_at_a_position() {
    let mut plan = Plan::from_categories(&base_categories());
    plan.move_to(0, 1).unwrap();
    assert_eq!(
        plan.entries()
            .iter()
            .map(|e| e.category.code.as_str())
            .collect::<Vec<_>>(),
        ["B", "A"]
    );
    assert!(matches!(
        plan.move_to(0, 5),
        Err(PlanError::Position { .. })
    ));
}

#[test]
fn add_inserts_a_one_off_category() {
    let orders = sample_orders();
    let categories = vec![category("A", "Sepatu", Some("name contains \"sepatu\""))];
    let mut plan = Plan::from_categories(&categories);
    // Added above the rest row (at the end).
    plan.add(1, category("X", "Spion", Some("name contains \"spion\"")))
        .unwrap();
    let result = plan.count(&orders, PdfNumber::new(1));

    assert_eq!(
        result
            .categories
            .iter()
            .map(|p| p.code.as_str())
            .collect::<Vec<_>>(),
        ["A", "X", REST_CODE]
    );
    assert_eq!(
        result.categories[1].orders,
        [OrderId::new("580000000000000003")]
    );
}

#[test]
fn rename_replaces_the_whole_file_name() {
    let orders = sample_orders();
    let mut plan = Plan::from_categories(&base_categories());
    plan.rename(0, "1 A Sepatu sendiri").unwrap();
    let result = plan.count(&orders, PdfNumber::new(1));
    assert_eq!(result.categories[0].file_name, "1 A Sepatu sendiri.pdf");

    // An invalid (empty) name is refused.
    assert!(plan.rename(0, "   ").is_err());
}

#[test]
fn duplicate_file_names_are_refused() {
    let orders = sample_orders();
    let mut plan = Plan::from_categories(&base_categories());
    plan.rename(0, "same").unwrap();
    plan.rename(1, "SAME.pdf").unwrap(); // case-insensitive, as Windows names are
    let result = plan.count(&orders, PdfNumber::new(1));
    assert!(result.check_file_names().is_err());
}

#[test]
fn suggested_file_name_follows_the_spec() {
    let orders = sample_orders();
    let result = Plan::from_categories(&base_categories()).count(&orders, PdfNumber::new(3));
    assert_eq!(result.categories[0].file_name, "3 A Sepatu ×2.pdf");
    // The rest pick's code `?` becomes `-`.
    assert_eq!(result.categories[2].file_name, "5 - Uncategorised ×1.pdf");
}

#[test]
fn runs_group_by_contents_sorted_by_count_then_contents() {
    let same = line("Spion Beat", 1, "honda");
    let orders = vec![
        order("580000000000000001", vec![same.clone()], Some("J&T")),
        order("580000000000000002", vec![same.clone()], Some("J&T")),
        order(
            "580000000000000003",
            vec![line("Alpha", 1, "")],
            Some("J&T"),
        ),
    ];
    // No entry: everything falls to the rest pick.
    let result = Plan::from_categories(&[]).count(&orders, PdfNumber::new(1));
    assert_eq!(result.categories.len(), 1);
    let runs = &result.categories[0].runs;
    assert_eq!(runs.len(), 2);
    assert_eq!(runs[0].orders.len(), 2); // the two identical-contents orders
    assert_eq!(runs[1].orders, [OrderId::new("580000000000000003")]);
    assert_eq!(runs[1].number, 2);
}

#[test]
fn a_category_reports_its_page_count() {
    let mut first = order(
        "580000000000000001",
        vec![line("Sepatu", 1, "")],
        Some("J&T"),
    );
    first.pages = vec![PageRef { file: 0, page: 0 }, PageRef { file: 0, page: 1 }];
    let orders = vec![first];
    let result = Plan::from_categories(&base_categories()).count(&orders, PdfNumber::new(1));
    assert_eq!(result.categories[0].pages, 2);
    assert_eq!(result.total_pages(), 2);
}

#[test]
fn no_orders_gives_no_categories() {
    let result = Plan::from_categories(&base_categories()).count(&[], PdfNumber::new(1));
    assert!(result.categories.is_empty());
    assert_eq!(result.total_orders(), 0);
}

#[test]
fn set_condition_changes_a_category_for_this_batch_only() {
    let orders = sample_orders();
    let categories = vec![category("A", "Sepatu", Some("name contains \"sepatu\""))];
    let mut plan = Plan::from_categories(&categories);
    plan.set_condition(
        0,
        Some(parse_condition("courier starts_with \"SiCepat\"").unwrap()),
    )
    .unwrap();
    let result = plan.count(&orders, PdfNumber::new(1));
    assert_eq!(
        result.categories[0].orders,
        [OrderId::new("580000000000000003")]
    );
    // The stored rules entry is untouched.
    assert_eq!(
        plan.entries()[0].category.when.as_deref(),
        Some("name contains \"sepatu\"")
    );
}

#[test]
fn rest_category_uses_the_documented_code_and_name() {
    let rest = packing_engine::plan::rest_category();
    assert_eq!(rest.code, REST_CODE);
    assert_eq!(rest.name, REST_NAME);
    assert!(rest.condition.is_none());
}
