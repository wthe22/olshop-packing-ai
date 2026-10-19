use super::*;
use packing_engine::day::{BatchEntry, PackingListChoice, PdfEntry, SavedItem, SavedOrder};
use packing_engine::groupings::Grouping;
use packing_engine::ids::{BatchNumber, OrderId, PdfNumber, TrackingId};
use packing_engine::plan::Plan;
use packing_engine::rules_edit::condition_to_tree;
use packing_pdf::batch::Draft;

use crate::OpenDraft;

fn temp_dir(label: &str) -> std::path::PathBuf {
    let dir =
        std::env::temp_dir().join(format!("packing-app-search-{}-{label}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}

fn batch_entry(batch: BatchNumber, pdf_file: &str) -> BatchEntry {
    BatchEntry {
        batch,
        save_time: "2026-10-07T07:40:12+07:00".to_string(),
        grouping: Grouping::default(),
        files: Vec::new(),
        pdfs: vec![PdfEntry {
            number: PdfNumber::new(1),
            code: "A".to_string(),
            name: "Sepatu".to_string(),
            file: pdf_file.to_string(),
            orders: 1,
            runs: 1,
        }],
        packing_list: PackingListChoice::default(),
        printed: Vec::new(),
        warnings: Vec::new(),
        errors: Vec::new(),
    }
}

/// One saved order with the searched fields written (as a save does, 07 › *`state.json`*).
fn saved_order(order_id: &str, tracking_id: &str, name: &str, quantity: i64) -> SavedOrder {
    SavedOrder {
        order_id: OrderId::new(order_id),
        tracking_id: Some(TrackingId::new(tracking_id)),
        batch: BatchNumber::new(1),
        pdf: PdfNumber::new(1),
        run: 1,
        items: vec![SavedItem {
            name: name.to_string(),
            variation: "42".to_string(),
            seller_sku: "SP-0001".to_string(),
            quantity,
        }],
        courier: "J&T Express".to_string(),
        total_quantity: quantity,
        distinct_items: 1,
        ship_by: None,
    }
}

/// One saved order as an older day holds it: only its IDs, no searched fields.
fn legacy_order(order_id: &str, tracking_id: &str) -> SavedOrder {
    SavedOrder {
        order_id: OrderId::new(order_id),
        tracking_id: Some(TrackingId::new(tracking_id)),
        batch: BatchNumber::new(1),
        pdf: PdfNumber::new(1),
        run: 1,
        items: Vec::new(),
        courier: String::new(),
        total_quantity: 0,
        distinct_items: 0,
        ship_by: None,
    }
}

fn write_day(folder: &Path, day: &str, saved: Vec<SavedOrder>) {
    let day_dir = folder.join(LABELS_DIR).join(day);
    let mut state = DayState::new(day);
    state
        .batches
        .push(batch_entry(BatchNumber::new(1), "1 A Sepatu ×1.pdf"));
    state.saved = saved;
    state.save(&day_dir).expect("save state");
}

fn app_state(folder: &Path, day: &str) -> AppState {
    let state = AppState::new();
    state.set_data_folder(folder.to_path_buf());
    state.set_day(day);
    state
}

fn tree(text: &str) -> Tree {
    condition_to_tree(text).expect("boxes")
}

#[test]
fn a_tracking_id_finds_its_one_order_with_the_saved_pdf_path() {
    let dir = temp_dir("tracking");
    write_day(
        &dir,
        "2026-10-07",
        vec![
            saved_order("580000000000000001", "JY0000001234", "Sepatu", 2),
            saved_order("580000000000000002", "JY0000005678", "Spion", 1),
        ],
    );
    let state = app_state(&dir, "2026-10-07");

    let view = search_orders_impl(
        &state,
        &tree("tracking_id equals \"JY0000001234\""),
        SearchScope::Day,
    )
    .expect("search");
    assert_eq!(view.orders, 1);
    assert_eq!(view.batches, 1);
    assert_eq!(view.rows.len(), 1);
    let row = &view.rows[0];
    assert_eq!(row.order_id.as_str(), "580000000000000001");
    assert_eq!(row.courier, "J&T Express");
    assert_eq!(row.items.len(), 1);
    assert_eq!(row.items[0].name, "Sepatu");
    assert_eq!(row.items[0].quantity, 2);
    assert_eq!(row.file, "1 A Sepatu ×1.pdf");
    assert_eq!(row.batch, BatchNumber::new(1));
    assert_eq!(row.day, "2026-10-07");
    assert!(!row.items_not_stored);
    assert!(
        row.path
            .ends_with("labels\\2026-10-07\\batch 1\\1 A Sepatu ×1.pdf")
            || row
                .path
                .ends_with("labels/2026-10-07/batch 1/1 A Sepatu ×1.pdf"),
        "{}",
        row.path
    );

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn this_day_does_not_see_another_days_order_but_all_days_does() {
    let dir = temp_dir("scope");
    write_day(
        &dir,
        "2026-10-07",
        vec![saved_order(
            "580000000000000001",
            "JY0000001234",
            "Sepatu",
            1,
        )],
    );
    write_day(
        &dir,
        "2026-10-06",
        vec![saved_order(
            "580000000000000009",
            "JY0000009999",
            "Spion",
            1,
        )],
    );
    let state = app_state(&dir, "2026-10-07");
    let cond = tree("tracking_id starts_with \"JY\"");

    let this_day = search_orders_impl(&state, &cond, SearchScope::Day).expect("day");
    assert_eq!(this_day.orders, 1);
    assert_eq!(this_day.rows[0].day, "2026-10-07");

    let all = search_orders_impl(&state, &cond, SearchScope::All).expect("all");
    assert_eq!(all.orders, 2);
    // Newest day first (the Day switch's order).
    assert_eq!(all.rows[0].day, "2026-10-07");
    assert_eq!(all.rows[1].day, "2026-10-06");
    assert_eq!(all.batches, 2, "two batches, one per day");

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn an_order_of_an_open_draft_is_not_found() {
    let dir = temp_dir("draft");
    write_day(
        &dir,
        "2026-10-07",
        vec![saved_order(
            "580000000000000001",
            "JY0000001234",
            "Sepatu",
            1,
        )],
    );
    let state = app_state(&dir, "2026-10-07");
    // An open draft holding an order that was not saved: Search reads `saved[]` only.
    let order = packing_engine::orders::ReadOrder {
        order: packing_engine::orders::Order {
            order_id: OrderId::new("580000000000000777"),
            tracking_id: Some(TrackingId::new("JY0000000777")),
            courier: "JNE".to_string(),
            ship_by: None,
            lines: vec![packing_engine::orders::Line {
                name: "Sepatu".to_string(),
                variation: String::new(),
                seller_sku: String::new(),
                quantity: 1,
                display_name: "Sepatu".to_string(),
            }],
            customer_message: String::new(),
        },
        pages: vec![packing_engine::orders::PageRef { file: 0, page: 0 }],
        position: 1,
        has_slip: true,
        qty_total: None,
    };
    state.set_draft(OpenDraft {
        draft: Draft {
            files: Vec::new(),
            orders: vec![order],
            duplicates: Vec::new(),
            warnings: Vec::new(),
            pages: 1,
            total_orders: 1,
            slip_pages: 1,
            unseen_logos: Vec::new(),
            read_warnings: Vec::new(),
        },
        plan: Plan::from_categories(&[]),
        first_number: PdfNumber::new(1),
        couriers: packing_engine::label::courier_data().clone(),
        grouping: Grouping::default(),
        amend: None,
        logo_pictures: Default::default(),
    });

    let view = search_orders_impl(
        &state,
        &tree("order_id contains \"580000000000000777\""),
        SearchScope::Day,
    )
    .expect("search");
    assert_eq!(view.orders, 0, "the draft's order is not saved");

    // The saved order is still found.
    let view = search_orders_impl(
        &state,
        &tree("order_id contains \"580000000000000001\""),
        SearchScope::Day,
    )
    .expect("search");
    assert_eq!(view.orders, 1);

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn an_order_saved_before_search_is_found_by_its_ids_only() {
    let dir = temp_dir("legacy");
    write_day(
        &dir,
        "2026-10-07",
        vec![legacy_order("580000000000000001", "JY0000001234")],
    );
    let state = app_state(&dir, "2026-10-07");

    let by_tracking = search_orders_impl(
        &state,
        &tree("tracking_id equals \"JY0000001234\""),
        SearchScope::Day,
    )
    .expect("search");
    assert_eq!(by_tracking.orders, 1);
    assert!(by_tracking.rows[0].items_not_stored);
    assert!(by_tracking.rows[0].items.is_empty());

    let by_order = search_orders_impl(
        &state,
        &tree("order_id contains \"580000000000000001\""),
        SearchScope::Day,
    )
    .expect("search");
    assert_eq!(by_order.orders, 1, "Order ID also finds it");

    // A condition on an item or other stored field cannot see it.
    let by_name = search_orders_impl(&state, &tree("name contains \"sepatu\""), SearchScope::Day)
        .expect("search");
    assert_eq!(by_name.orders, 0);

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn a_bad_condition_is_a_message() {
    let dir = temp_dir("bad");
    let state = app_state(&dir, "2026-10-07");
    // An empty root group is an error (08 › *5. Categories*): the panel shows it.
    let empty = Tree {
        root: packing_engine::rules_edit::Group {
            kind: packing_engine::rules_edit::GroupKind::All,
            not: false,
            children: Vec::new(),
        },
    };
    let error = search_orders_impl(&state, &empty, SearchScope::Day).expect_err("empty group");
    assert!(error.contains("empty"), "{error}");
    std::fs::remove_dir_all(&dir).ok();
}
