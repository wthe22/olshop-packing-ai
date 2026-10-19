use super::*;
use packing_engine::ids::{BatchNumber, OrderId, PdfNumber, TrackingId};
use packing_engine::label::courier_data;
use packing_engine::orders::Order;
use packing_engine::plan::{CategoryResult, REST_CODE, RunResult};

fn line(name: &str, variation: &str, quantity: i64, display: &str) -> Line {
    Line {
        name: name.to_string(),
        variation: variation.to_string(),
        seller_sku: String::new(),
        quantity,
        display_name: display.to_string(),
    }
}

fn read_order(id: &str, tracking: &str, lines: Vec<Line>) -> ReadOrder {
    ReadOrder {
        order: Order {
            order_id: OrderId::new(id),
            tracking_id: (!tracking.is_empty()).then(|| TrackingId::new(tracking)),
            courier: Some("J&T Express".to_string()),
            ship_by: None,
            lines,
            customer_message: String::new(),
        },
        pages: vec![packing_engine::orders::PageRef { file: 0, page: 0 }],
        position: 1,
        has_slip: true,
        qty_total: None,
    }
}

fn order_ids() -> Vec<OrderId> {
    vec![
        OrderId::new("580000000000000001"),
        OrderId::new("580000000000000002"),
    ]
}

fn category() -> (CategoryResult, Vec<ReadOrder>) {
    let orders = vec![
        read_order(
            "580000000000000001",
            "JY0000000101",
            vec![line("Sepatu", "honda", 1, "Sepatu — honda")],
        ),
        read_order(
            "580000000000000002",
            "JY0000000102",
            vec![line("Sepatu", "honda", 1, "Sepatu — honda")],
        ),
    ];
    let category = CategoryResult {
        number: PdfNumber::new(1),
        code: "A".to_string(),
        name: "Sepatu".to_string(),
        orders: order_ids(),
        runs: vec![RunResult {
            number: 1,
            orders: order_ids(),
        }],
        pages: 2,
        file_name: "1 A Sepatu ×2.pdf".to_string(),
    };
    (category, orders)
}

#[test]
fn category_section_maps_runs_and_lines() {
    let (category, orders) = category();
    let draft = Draft {
        files: Vec::new(),
        orders,
        duplicates: Vec::new(),
        warnings: Vec::new(),
        pages: 2,
        total_orders: 2,
        slip_pages: 2,
        unseen_logos: Vec::new(),
        read_warnings: Vec::new(),
    };
    let index = order_index(&draft);
    let section = category_section(&category, &index).expect("section");

    assert_eq!(section.number, PdfNumber::new(1));
    assert_eq!(section.code, "A");
    assert_eq!(section.name, "Sepatu");
    assert_eq!(section.runs.len(), 1);
    assert_eq!(section.runs[0].number, 1);
    assert_eq!(section.runs[0].orders.len(), 2);
    assert_eq!(
        section.runs[0].orders[0]
            .tracking_id
            .as_ref()
            .map(TrackingId::as_str),
        Some("JY0000000101")
    );
    assert_eq!(section.runs[0].orders[0].lines[0].quantity, 1);
    assert_eq!(
        section.runs[0].orders[0].lines[0].display_name,
        "Sepatu — honda"
    );
}

#[test]
fn category_pages_and_saved_follow_the_runs() {
    let (category, orders) = category();
    let draft = Draft {
        files: Vec::new(),
        orders,
        duplicates: Vec::new(),
        warnings: Vec::new(),
        pages: 2,
        total_orders: 2,
        slip_pages: 2,
        unseen_logos: Vec::new(),
        read_warnings: Vec::new(),
    };
    let index = order_index(&draft);

    assert_eq!(
        category_pages(&category, &index).expect("pages"),
        vec![(0, 0), (0, 0)]
    );
    let saved = category_saved(&category, &index, BatchNumber::new(1)).expect("saved");
    assert_eq!(saved.len(), 2);
    assert_eq!(saved[0].pdf, PdfNumber::new(1));
    assert_eq!(saved[0].run, 1);
    assert_eq!(saved[1].order_id.as_str(), "580000000000000002");
    // The searched fields are written at save (07 › *`state.json`*; 2.21): the order's lines
    // and the order-level fields.
    assert_eq!(saved[0].courier.as_deref(), Some("J&T Express"));
    assert_eq!(saved[0].total_quantity, 1);
    assert_eq!(saved[0].distinct_items, 1);
    assert!(saved[0].ship_by.is_none());
    assert_eq!(saved[0].items.len(), 1);
    assert_eq!(saved[0].items[0].name, "Sepatu");
    assert_eq!(saved[0].items[0].variation, "honda");
    assert_eq!(saved[0].items[0].quantity, 1);
}

#[test]
fn summary_text_lists_the_categories_and_the_written_folder() {
    let (category, orders) = category();
    let rest = CategoryResult {
        number: PdfNumber::new(2),
        code: REST_CODE.to_string(),
        name: "Uncategorised".to_string(),
        orders: Vec::new(),
        runs: Vec::new(),
        pages: 0,
        file_name: "2 - Uncategorised ×0.pdf".to_string(),
    };
    let categories = vec![category, rest];
    let warnings = vec!["no courier could be deduced for: 580000000000000009".to_string()];
    let packing_lists = vec!["packing-list.pdf".to_string()];
    let day_dir = Path::new("data/labels/2026-10-07");

    let text = summary_text(&SummaryInput {
        day: "2026-10-07",
        files: 1,
        pages: 2,
        orders: 2,
        slip_pages: 2,
        categories: &categories,
        duplicates: 1,
        warnings: &warnings,
        check_errors: &[],
        check_warnings: &[],
        day_dir,
        packing_lists: &packing_lists,
        quiet_ids: false,
    });

    assert!(text.starts_with("Day 2026-10-07 · 1 label file · no CSV\n"));
    assert!(text.contains("  Pages: 2 read · 2 orders\n"));
    assert!(text.contains("  Item data: slips (2 pages)\n"));
    assert!(text.contains("  Pick A  Sepatu: 2 orders, 1 runs → 1 A Sepatu ×2.pdf\n"));
    assert!(
        text.contains("  Rest ?  Uncategorised: 0 orders, 0 runs → 2 - Uncategorised ×0.pdf\n")
    );
    assert!(text.contains("  Duplicates: 1\n"));
    assert!(text.contains("  Warnings: 1\n"));
    assert!(text.contains("    - no courier could be deduced for: 580000000000000009\n"));
    assert!(text.contains("Written: data/labels/2026-10-07/  (2 saved PDFs, packing-list.pdf)\n"));
    // Keeps the orders variable referenced even though it is only used for the index above.
    assert_eq!(orders.len(), 2);
}

#[test]
fn quiet_summary_hides_the_id_detail_lines() {
    let warnings = vec!["no tracking ID for: 580000000000000004".to_string()];
    let text = summary_text(&SummaryInput {
        day: "2026-10-07",
        files: 1,
        pages: 1,
        orders: 1,
        slip_pages: 1,
        categories: &[],
        duplicates: 0,
        warnings: &warnings,
        check_errors: &[],
        check_warnings: &[],
        day_dir: Path::new("d"),
        packing_lists: &[],
        quiet_ids: true,
    });
    assert!(text.contains("  Warnings: 1\n"));
    assert!(!text.contains("580000000000000004"), "no IDs: {text}");
    assert!(text.contains("Written: d/  (0 saved PDFs, no packing list)\n"));
}

#[test]
fn printed_time_slices_the_clock() {
    assert_eq!(printed_time("2026-10-07T07:40:12+07:00"), "07:40");
}

// --------------------------------------------------- the logo prompt's answer (D-LABEL-4)

/// Two orders of the batch with no courier: one `JY…` (J&T Express alone) and one `00…`
/// (shared with SiCepat), both carrying one unseen logo.
fn draft_with_unseen_logo(order_ids: Vec<&str>) -> Draft {
    let mut first = read_order(
        "580000000000000001",
        "JY0000001234",
        vec![line("Sepatu", "", 1, "Sepatu")],
    );
    first.order.courier = None;
    let mut second = read_order(
        "580000000000000002",
        "000000000202",
        vec![line("Sepatu", "", 1, "Sepatu")],
    );
    second.order.courier = None;
    let mut draft = Draft {
        files: Vec::new(),
        orders: vec![first, second],
        duplicates: Vec::new(),
        warnings: Vec::new(),
        pages: 2,
        total_orders: 2,
        slip_pages: 2,
        unseen_logos: vec![UnseenLogo {
            hash: "ffff".to_string(),
            appearance: None,
            pages: 2,
            order_ids: order_ids.into_iter().map(OrderId::new).collect(),
        }],
        read_warnings: Vec::new(),
    };
    draft.read_warnings = vec![Warning::unknown_courier(&draft.orders).expect("both unknown")];
    draft.warnings = draft
        .read_warnings
        .iter()
        .map(|warning| warning.to_string())
        .collect();
    draft
}

#[test]
fn an_answer_names_only_the_form_agreeing_orders_and_refreshes_the_warning() {
    let mut draft = draft_with_unseen_logo(vec!["580000000000000001", "580000000000000002"]);
    assert_eq!(
        draft.warnings,
        vec![
            "Courier unknown for 2 orders: JY0000001234, 000000000202. They are sorted \
             normally; only a condition on courier cannot see them."
                .to_string()
        ]
    );

    // The JY form names J&T Express; the shared 00 form does not.
    let named = draft.answer_logo(
        "ffff",
        &LogoAnswer::Courier {
            name: "J&T Express".to_string(),
        },
        courier_data(),
    );
    assert_eq!(named, 1);
    assert_eq!(
        draft.orders[0].order.courier.as_deref(),
        Some("J&T Express")
    );
    assert_eq!(draft.orders[1].order.courier, None);
    assert!(draft.unseen_logos.is_empty(), "the answered logo is spent");
    assert_eq!(
        draft.warnings,
        vec![
            "Courier unknown for 1 order: 000000000202. They are sorted normally; only a \
             condition on courier cannot see them."
                .to_string()
        ]
    );
    // A second answer about the same hash changes nothing.
    assert_eq!(
        draft.answer_logo(
            "ffff",
            &LogoAnswer::Courier {
                name: "SiCepat".to_string(),
            },
            courier_data(),
        ),
        0
    );
}

#[test]
fn an_answer_naming_every_order_removes_the_warning() {
    let mut draft = draft_with_unseen_logo(vec!["580000000000000002"]);
    let named = draft.answer_logo(
        "ffff",
        &LogoAnswer::Courier {
            name: "SiCepat".to_string(),
        },
        courier_data(),
    );
    assert_eq!(named, 1);
    assert_eq!(draft.orders[1].order.courier.as_deref(), Some("SiCepat"));
    // The JY order was not carrying the logo: it stays unknown and keeps its warning.
    assert_eq!(draft.orders[0].order.courier, None);
    assert_eq!(draft.warnings.len(), 1);
    assert!(draft.warnings[0].contains("Courier unknown for 1 order: JY0000001234"));
}

#[test]
fn not_a_courier_and_skip_name_nothing_but_leave_the_list() {
    for answer in [LogoAnswer::NotCourier, LogoAnswer::Skip] {
        let mut draft = draft_with_unseen_logo(vec!["580000000000000001"]);
        assert_eq!(draft.answer_logo("ffff", &answer, courier_data()), 0);
        assert_eq!(draft.orders[0].order.courier, None);
        assert!(draft.unseen_logos.is_empty());
        // Both orders are still unknown: the warning is unchanged.
        assert!(draft.warnings[0].contains("Courier unknown for 2 orders"));
    }
}

#[test]
fn the_duplicate_warning_stays_after_a_logo_answer() {
    let mut draft = draft_with_unseen_logo(vec!["580000000000000001"]);
    draft.duplicates = vec![Duplicate {
        order_id: OrderId::new("580000000000000009"),
        tracking_id: Some(TrackingId::new("JY0000009999")),
        batch: BatchNumber::new(1),
        pdf: PdfNumber::new(2),
        time: "07:40".to_string(),
    }];
    draft.warnings.push(duplicates_warning(&draft.duplicates));

    draft.answer_logo(
        "ffff",
        &LogoAnswer::Courier {
            name: "J&T Express".to_string(),
        },
        courier_data(),
    );
    assert_eq!(draft.warnings.len(), 2);
    assert!(draft.warnings[0].contains("Courier unknown for 1 order"));
    assert!(draft.warnings[1].starts_with("1 orders were already saved today"));
}
