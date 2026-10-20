//! `plan.rs`: the plan edits (skip, move, add, rename), picks from what is left, runs and
//! numbering (mirrors the relevant cases of tools/analysis/tests/test_picks.py).

use packing_engine::ids::{OrderId, PdfNumber, TrackingId};
use packing_engine::orders::{Line, Order, PageRef, ReadOrder};
use packing_engine::plan::{Plan, PlanError, REST_CODE, REST_NAME};
use packing_engine::rules::{Category, catch_all_category, parse_condition};
use testkit::{category, line_with_variation};

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

/// Two sepatu, one spion, one other (courier J&T) — enough to tell the entries apart.
fn sample_orders() -> Vec<ReadOrder> {
    vec![
        order(
            "580000000000000001",
            vec![line_with_variation("Sepatu A", "", 1)],
            Some("J&T Express"),
        ),
        order(
            "580000000000000002",
            vec![line_with_variation("Sepatu B", "", 1)],
            Some("J&T Express"),
        ),
        order(
            "580000000000000003",
            vec![line_with_variation("Spion C", "", 1)],
            Some("SiCepat"),
        ),
        order(
            "580000000000000004",
            vec![line_with_variation("Lainnya D", "", 1)],
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
fn counting_a_batch_numbers_only_the_pdfs_that_get_orders() {
    let orders = sample_orders();
    let result = Plan::from_categories(&base_categories()).count(&orders, PdfNumber::new(1));

    // The entries in order, then the rest; each entry takes what the ones above left it.
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

    // The numbering continues from the first number given.
    let numbered = Plan::from_categories(&base_categories()).count(&orders, PdfNumber::new(7));
    assert_eq!(
        numbered
            .categories
            .iter()
            .map(|p| p.number)
            .collect::<Vec<_>>(),
        [PdfNumber::new(7), PdfNumber::new(8), PdfNumber::new(9)]
    );

    // An entry that takes no order is left out, so the entry below it gets its number.
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

    // The last entry without a condition takes the rest.
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
/// PDF set, and the plan is the unconfigured state.
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

    // The rest entry carries the documented code and name, and takes whatever is left.
    let rest = packing_engine::plan::rest_category();
    assert_eq!(rest.code, REST_CODE);
    assert_eq!(rest.name, REST_NAME);
    assert!(rest.condition.is_none());
}

#[test]
fn skipping_moving_and_adding_change_which_entry_takes_first() {
    let orders = sample_orders();

    // A skipped entry takes nothing, so its orders fall to the rows below.
    let mut plan = Plan::from_categories(&base_categories());
    plan.skip(0).unwrap();
    let result = plan.count(&orders, PdfNumber::new(1));
    assert_eq!(result.categories.len(), 2);
    assert_eq!(result.categories[0].code, "B");
    assert_eq!(result.categories[1].code, REST_CODE);
    assert_eq!(result.categories[1].orders.len(), 3);
    // Unskip restores the first result.
    plan.unskip(0).unwrap();
    assert_eq!(
        plan.count(&orders, PdfNumber::new(1)).categories[0]
            .orders
            .len(),
        2
    );

    // Moving a row up gives it the first pick.
    let categories = vec![
        category("A", "Sepatu", Some("name contains \"sepatu\"")),
        category("J", "J&T", Some("courier starts_with \"J&T\"")),
    ];
    let mut plan = Plan::from_categories(&categories);
    assert_eq!(
        plan.count(&orders, PdfNumber::new(1)).categories[0].code,
        "A"
    );
    plan.move_up(1).unwrap();
    let result = plan.count(&orders, PdfNumber::new(1));
    assert_eq!(result.categories[0].code, "J");
    assert_eq!(result.categories[0].orders.len(), 3);
    assert_eq!(result.categories[1].code, REST_CODE);
    assert_eq!(
        result.categories[1].orders,
        [OrderId::new("580000000000000003")]
    );

    // Moving to a position is bounded by the row count.
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

    // An added entry takes its place above the rest row and takes what is left for it.
    let categories = vec![category("A", "Sepatu", Some("name contains \"sepatu\""))];
    let mut plan = Plan::from_categories(&categories);
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
fn renaming_and_a_condition_change_this_batch_and_not_the_stored_rules() {
    let orders = sample_orders();

    // The suggested file names follow the spec, the rest code `?` becoming `-`.
    let result = Plan::from_categories(&base_categories()).count(&orders, PdfNumber::new(3));
    assert_eq!(result.categories[0].file_name, "3 A Sepatu ×2.pdf");
    assert_eq!(result.categories[2].file_name, "5 - Uncategorised ×1.pdf");

    // A rename replaces the whole file name, and an empty one is refused.
    let mut plan = Plan::from_categories(&base_categories());
    plan.rename(0, "1 A Sepatu sendiri").unwrap();
    let result = plan.count(&orders, PdfNumber::new(1));
    assert_eq!(result.categories[0].file_name, "1 A Sepatu sendiri.pdf");
    assert!(plan.rename(0, "   ").is_err());

    // Two names that differ only in case are refused: Windows treats them as one file.
    let mut plan = Plan::from_categories(&base_categories());
    plan.rename(0, "same").unwrap();
    plan.rename(1, "SAME.pdf").unwrap();
    assert!(
        plan.count(&orders, PdfNumber::new(1))
            .check_file_names()
            .is_err()
    );

    // A condition set for this batch takes the order it names, and leaves the stored rules alone.
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
    assert_eq!(
        plan.entries()[0].category.when.as_deref(),
        Some("name contains \"sepatu\"")
    );
}

#[test]
fn runs_group_by_contents_sorted_by_count_then_contents() {
    let same = line_with_variation("Spion Beat", "honda", 1);
    let orders = vec![
        order("580000000000000001", vec![same.clone()], Some("J&T")),
        order("580000000000000002", vec![same.clone()], Some("J&T")),
        order(
            "580000000000000003",
            vec![line_with_variation("Alpha", "", 1)],
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
fn the_plan_counts_pages_and_an_empty_batch_has_no_categories() {
    let mut first = order(
        "580000000000000001",
        vec![line_with_variation("Sepatu", "", 1)],
        Some("J&T"),
    );
    first.pages = vec![PageRef { file: 0, page: 0 }, PageRef { file: 0, page: 1 }];
    let result = Plan::from_categories(&base_categories()).count(&[first], PdfNumber::new(1));
    assert_eq!(result.categories[0].pages, 2);
    assert_eq!(result.total_pages(), 2);

    // No order at all: no entry gets a PDF.
    let result = Plan::from_categories(&base_categories()).count(&[], PdfNumber::new(1));
    assert!(result.categories.is_empty());
    assert_eq!(result.total_orders(), 0);
}
