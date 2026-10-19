//! The Plan screen's view and its edits, on orders built in code.

use super::*;

#[test]
fn plan_view_numbers_rows_and_adds_the_rest_row() {
    let view = plan_view(&sepatu_plan(), &orders(), 4, &Grouping::default());
    assert_eq!(view.rows.len(), 3);
    assert_eq!(view.add_at, 2); // after the two entries (no rest entry)

    let a = &view.rows[0];
    assert_eq!(a.number, Some(4));
    assert_eq!(a.code, "A");
    assert_eq!(a.orders, 3);
    assert_eq!(a.runs.len(), 3); // three different contents, one order each
    assert_eq!(a.runs[0].label, "4-01");
    assert_eq!(a.runs[0].orders, 1);
    assert_eq!(a.runs[0].contents, "Sepatu A ×1");
    assert_eq!(a.runs[2].label, "4-03");
    assert_eq!(a.file_name, "4 A Sepatu ×3.pdf");
    assert!(!a.empty && !a.is_rest && a.can_use);
    assert!(!a.can_move_up && a.can_move_down); // first row

    let b = &view.rows[1];
    assert_eq!(b.number, Some(5));
    assert_eq!(b.orders, 2);
    assert_eq!(b.condition.as_deref(), Some("name contains \"spion\""));

    let rest = &view.rows[2];
    assert!(rest.generated && rest.is_rest && !rest.can_use);
    assert_eq!(rest.index, None);
    assert_eq!(rest.number, Some(6));
    assert_eq!(rest.code, REST_CODE);
    assert_eq!(rest.name, "Uncategorised");
    assert_eq!(rest.orders, 1);
    assert!(!rest.can_move_up && !rest.can_move_down);

    assert!(!view.has_duplicate_names);
}

#[test]
fn plan_view_shows_zero_order_and_skipped_rows() {
    let mut plan = sepatu_plan();
    // Untick the sepatu row: its three orders fall to the rest row.
    apply_edits(
        &mut plan,
        &[PlanEdit::SetSkipped {
            index: 0,
            skipped: true,
        }],
    )
    .expect("skip");
    let view = plan_view(&plan, &orders(), 1, &Grouping::default());

    assert!(view.rows[0].skipped);
    assert_eq!(view.rows[0].orders, 0);
    assert_eq!(view.rows[0].number, None);
    // Spion takes 2; the rest row takes the 3 sepatu + 1 lain.
    assert_eq!(view.rows[1].orders, 2);
    assert_eq!(view.rows[2].orders, 4);
    assert_eq!(view.rows[2].number, Some(2)); // the skipped row used no number

    // A condition that matches nothing shows a 0-order row with no number.
    let plan = Plan::from_categories(&[
        category("A", "Nope", Some("name contains \"zzz\"")),
        category("Z", "Lainnya", None),
    ]);
    let view = plan_view(&plan, &orders(), 1, &Grouping::default());
    assert!(view.rows[0].empty);
    assert_eq!(view.rows[0].number, None);
    assert_eq!(view.rows[1].number, Some(1)); // the rest entry catches everything
    assert!(view.rows[1].is_rest);
    assert_eq!(view.add_at, 1); // before the rest entry
}

#[test]
fn plan_view_flags_duplicate_file_names() {
    let mut plan = sepatu_plan();
    apply_edits(
        &mut plan,
        &[
            PlanEdit::Rename {
                index: 0,
                name: "same.pdf".to_string(),
            },
            PlanEdit::Rename {
                index: 1,
                name: "Same.pdf".to_string(),
            },
        ],
    )
    .expect("rename");
    let view = plan_view(&plan, &orders(), 1, &Grouping::default());
    assert!(view.has_duplicate_names);
    assert_eq!(view.rows[0].duplicate_of, None);
    assert_eq!(view.rows[1].duplicate_of, Some(1)); // *Same name as PDF 1*
}

#[test]
fn plan_view_flags_the_unconfigured_one_catch_all_entry() {
    // No `categories.toml`: one catch-all entry, every order in one PDF set (Q-UI-4).
    let plan = Plan::from_categories(&[catch_all_category()]);
    let view = plan_view(&plan, &orders(), 1, &Grouping::default());
    assert!(view.no_picks);
    assert_eq!(view.rows.len(), 1);
    assert_eq!(view.rows[0].code, "Z");
    assert_eq!(view.rows[0].orders, 6);
    assert!(view.rows[0].is_rest);
    assert_eq!(view.add_at, 0);

    // A pick with a condition: configured.
    assert!(!plan_view(&sepatu_plan(), &orders(), 1, &Grouping::default()).no_picks);
    // The picks file's own rest entry as the only entry: still unconfigured.
    let view = plan_view(
        &Plan::from_categories(&[category("Z", "Lainnya", None)]),
        &orders(),
        1,
        &Grouping::default(),
    );
    assert!(view.no_picks);
    assert_eq!(view.rows[0].orders, 6);
}

/// The plan view names the picks set the plan was built from (04 › *Sets*; 08 › *3. Plan*):
/// the header shows it, and **Edit categories** opens it.
#[test]
fn plan_view_names_the_grouping() {
    assert_eq!(
        plan_view(&sepatu_plan(), &orders(), 1, &Grouping::default()).grouping,
        Grouping::default()
    );
    assert_eq!(
        plan_view(&sepatu_plan(), &orders(), 1, &Grouping::new("ramadhan")).grouping,
        Grouping::new("ramadhan")
    );
}

/// A set with no file is the one catch-all entry (04 › *Sets*), so the plan is a single row
/// taking every order and the screen shows the *No picks yet* line (Q-UI-4).
#[test]
fn a_grouping_with_no_file_is_the_one_catch_all_entry() {
    let dir = temp_dir("grouping-no-file");
    let categories =
        load_categories(&dir, &Grouping::new("ramadhan")).expect("a set with no file still loads");
    let view = plan_view(
        &Plan::from_categories(&categories),
        &orders(),
        1,
        &Grouping::new("ramadhan"),
    );
    assert!(view.no_picks, "the *No picks yet* line is shown");
    assert_eq!(view.grouping, Grouping::new("ramadhan"));
    assert_eq!(view.rows.len(), 1);
    assert_eq!(view.rows[0].code, "Z");
    assert_eq!(view.rows[0].name, "Lainnya");
    assert_eq!(view.rows[0].orders, 6);
    assert_eq!(view.rows[0].number, Some(1));
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn apply_edits_moves_adds_and_reports_condition_errors() {
    let mut plan = sepatu_plan();
    apply_edits(&mut plan, &[PlanEdit::MoveDown { index: 0 }]).expect("move");
    assert_eq!(plan.entries()[0].category.code, "B");
    apply_edits(&mut plan, &[PlanEdit::MoveUp { index: 1 }]).expect("move back");
    assert_eq!(plan.entries()[0].category.code, "A");

    apply_edits(
        &mut plan,
        &[PlanEdit::Add {
            at: 2,
            code: "X".to_string(),
            name: "One off".to_string(),
            condition: "total_quantity >= 2".to_string(),
        }],
    )
    .expect("add");
    assert_eq!(plan.len(), 3);
    assert_eq!(plan.entries()[2].category.code, "X");
    assert_eq!(
        plan.entries()[2].category.when.as_deref(),
        Some("total_quantity >= 2")
    );

    let error = apply_edits(
        &mut plan,
        &[PlanEdit::Add {
            at: 3,
            code: "Y".to_string(),
            name: "Bad".to_string(),
            condition: "name contains".to_string(),
        }],
    )
    .expect_err("bad condition");
    assert!(error.contains("line 1, col"), "{error}");

    let error = apply_edits(
        &mut plan,
        &[PlanEdit::Rename {
            index: 0,
            name: "   ".to_string(),
        }],
    )
    .expect_err("empty name");
    assert!(error.contains("empty"), "{error}");
}

#[test]
fn check_condition_reports_the_engine_message() {
    assert!(check_condition("name contains \"x\"".to_string()).is_ok());
    let error = check_condition("name contains".to_string()).expect_err("bad");
    assert!(error.starts_with("line 1, col "), "{error}");
}

#[test]
fn save_progress_names_the_two_stages() {
    let writing = SaveProgressEvent::of(SaveProgress {
        stage: SaveStage::Writing,
        page: 3,
        pages: 10,
    });
    assert_eq!(writing.stage, "Writing");
    assert_eq!(writing.page, 3);
    assert_eq!(writing.pages, 10);

    let checking = SaveProgressEvent::of(SaveProgress {
        stage: SaveStage::Checking,
        page: 1,
        pages: 4,
    });
    assert_eq!(checking.stage, "Checking");
    assert_eq!(checking.page, 1);
    assert_eq!(checking.pages, 4);
}
