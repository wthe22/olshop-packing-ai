//! Reading and saving a batch on `testdata/`, skipped when `vendor/pdfium.dll` is missing.

use super::*;
use packing_engine::ids::{BatchNumber, PdfNumber};

#[test]
fn read_and_save_testdata_labels_slip() {
    let Some(_vendor) = vendor_dir() else {
        eprintln!("skipped: run tools/get-pdfium.sh");
        return;
    };
    let dir = temp_dir("slip");
    std::fs::copy(testdata("categories.toml"), dir.join(CATEGORIES_FILE)).expect("copy rules");
    let state = app_state(&dir);

    let read = read_draft(
        &state,
        vec![testdata("labels-slip.pdf")],
        &Grouping::default(),
        |_| {},
    )
    .expect("read");
    assert_eq!(read.orders, 23);
    assert_eq!(read.total_orders, 23);
    assert_eq!(read.already_saved, 0);
    assert_eq!(read.pages, 24);
    let numbered: Vec<(PdfNumber, usize)> = read
        .plan
        .rows
        .iter()
        .filter_map(|row| row.number.map(|number| (number, row.orders)))
        .collect();
    assert_eq!(
        numbered,
        vec![
            (PdfNumber::new(1), 7),
            (PdfNumber::new(2), 8),
            (PdfNumber::new(3), 4),
            (PdfNumber::new(4), 2),
            (PdfNumber::new(5), 2),
        ]
    );

    // The save reports its two stages through the progress channel (the window's
    // `save-progress` event): *Writing* then *Checking*.
    let seen = Arc::new(std::sync::Mutex::new(Vec::new()));
    let collect = Arc::clone(&seen);
    let save = save_batch_impl(&state, None, false, move |progress| {
        collect.lock().unwrap().push(progress);
    })
    .expect("save");
    assert_eq!(save.batch, BatchNumber::new(1));
    assert_eq!(save.pdfs, 5);
    assert_eq!(save.packing_lists, 1);
    assert!(dir.join("labels/2026-10-07/state.json").is_file());
    {
        let seen = seen.lock().unwrap();
        assert!(
            seen.iter()
                .any(|progress| progress.stage == SaveStage::Writing),
            "{seen:?}"
        );
        assert!(
            seen.iter()
                .any(|progress| progress.stage == SaveStage::Checking),
            "{seen:?}"
        );
    }
    // A second read of the same file: every order was already saved today.
    let again = read_draft(
        &state,
        vec![testdata("labels-slip.pdf")],
        &Grouping::default(),
        |_| {},
    )
    .err()
    .expect("already saved");
    assert!(again.contains("already saved today"), "{again}");

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn read_plain_labels_stops_with_no_slip() {
    let Some(_vendor) = vendor_dir() else {
        eprintln!("skipped: run tools/get-pdfium.sh");
        return;
    };
    let dir = temp_dir("plain");
    std::fs::copy(testdata("categories.toml"), dir.join(CATEGORIES_FILE)).expect("copy rules");
    let state = app_state(&dir);

    let error = read_draft(
        &state,
        vec![testdata("labels-plain.pdf")],
        &Grouping::default(),
        |_| {},
    )
    .err()
    .expect("no slip");
    assert!(error.contains("no packing slip"), "{error}");

    std::fs::remove_dir_all(&dir).ok();
}

/// A data folder without `categories.toml`: the picks are the one catch-all entry, so the
/// batch saves every order in one PDF set.
#[test]
fn a_read_without_a_categories_file_saves_every_order_in_one_pdf() {
    let Some(_vendor) = vendor_dir() else {
        eprintln!("skipped: run tools/get-pdfium.sh");
        return;
    };
    let dir = temp_dir("no-categories");
    let state = app_state(&dir);

    let read = read_draft(
        &state,
        vec![testdata("labels-slip.pdf")],
        &Grouping::default(),
        |_| {},
    )
    .expect("read");
    assert_eq!(read.orders, 23);
    assert_eq!(read.plan.rows.len(), 1);
    assert!(read.plan.no_categories);
    assert_eq!(read.plan.rows[0].code, "Z");
    assert_eq!(read.plan.rows[0].name, "Lainnya");
    assert_eq!(read.plan.rows[0].orders, 23);
    assert_eq!(read.plan.rows[0].number, Some(PdfNumber::new(1)));

    let save = save_batch_impl(&state, None, false, |_| {}).expect("save");
    assert_eq!(save.batch, BatchNumber::new(1));
    assert_eq!(save.pdfs, 1, "one PDF set for every order");
    let day_state = DayState::load(&dir.join("labels/2026-10-07")).expect("state");
    assert_eq!(day_state.batches[0].pdfs.len(), 1);
    assert_eq!(day_state.batches[0].pdfs[0].orders, 23);
    assert_eq!(day_state.batches[0].pdfs[0].file, "1 Z Lainnya ×23.pdf");
    assert!(
        dir.join("labels/2026-10-07/batch 1/1 Z Lainnya ×23.pdf")
            .is_file()
    );

    std::fs::remove_dir_all(&dir).ok();
}

/// A second set with its own picks: one condition and the last entry taking the rest, so its
/// plan can be told apart from `default`'s (whose file has four entries and an automatic
/// *Uncategorised* row).
const RAMADHAN: &str = "# the ramadhan set\n[[category]]\ncode = \"A\"\nname = \"Sepatu\"\nwhen = 'name contains \"sepatu\"'\n\n[[category]]\ncode = \"Z\"\nname = \"Lainnya\"\n";

/// A batch reads with the set chosen on the New batch screen: that set's own
/// picks, the data folder's `categories.toml` left alone; the set is remembered as the one used
/// last and recorded in the draft, so **Save batch** writes it into
/// `state.json`.
#[test]
fn a_batch_read_uses_the_chosen_set_and_records_it() {
    let Some(_vendor) = vendor_dir() else {
        eprintln!("skipped: run tools/get-pdfium.sh");
        return;
    };
    let dir = temp_dir("read-grouping");
    let groupings_folder = dir.join(crate::GROUPINGS_DIR);
    std::fs::create_dir_all(&groupings_folder).expect("groupings folder");
    std::fs::write(groupings_folder.join("ramadhan.toml"), RAMADHAN).expect("write set");
    let state = app_state(&dir);

    let read = read_draft(
        &state,
        vec![testdata("labels-slip.pdf")],
        &Grouping::new("ramadhan"),
        |_| {},
    )
    .expect("read");
    assert_eq!(read.orders, 23);
    assert!(!read.plan.no_categories);
    assert_eq!(
        read.plan.grouping,
        Grouping::new("ramadhan"),
        "the plan names the chosen set for the header"
    );
    let rows: Vec<(&str, Option<PdfNumber>)> = read
        .plan
        .rows
        .iter()
        .map(|row| (row.code.as_str(), row.number))
        .collect();
    assert_eq!(
        rows,
        vec![
            ("A", Some(PdfNumber::new(1))),
            ("Z", Some(PdfNumber::new(2)))
        ]
    );

    // The requested set is the one the next New batch preselects ...
    assert_eq!(
        settings_load(&dir).expect("settings").grouping,
        Grouping::new("ramadhan")
    );
    // ... the draft carries it, so *Save batch* records it with the batch ...
    state
        .with_draft(|open| assert_eq!(open.grouping, Grouping::new("ramadhan")))
        .expect("draft");
    assert_eq!(
        save_batch_impl(&state, None, false, |_| {})
            .expect("save")
            .batch,
        BatchNumber::new(1)
    );
    let day_state = DayState::load(&dir.join("labels/2026-10-07")).expect("state");
    assert_eq!(day_state.batches[0].grouping, Grouping::new("ramadhan"));
    // ... and the data folder's own `categories.toml` was never written.
    assert!(!dir.join(CATEGORIES_FILE).exists());

    std::fs::remove_dir_all(&dir).ok();
}

/// An **Amend** reads the batch's recorded set again, whatever set was used last: the amend's plan is the recorded set's picks, not the last-used ones.
#[test]
fn an_amend_reads_the_batchs_own_set_whatever_was_used_last() {
    let Some(_vendor) = vendor_dir() else {
        eprintln!("skipped: run tools/get-pdfium.sh");
        return;
    };
    let dir = temp_dir("amend-grouping");
    std::fs::create_dir_all(dir.join(crate::GROUPINGS_DIR)).expect("groupings folder");
    std::fs::write(
        dir.join(crate::GROUPINGS_DIR).join("ramadhan.toml"),
        RAMADHAN,
    )
    .expect("write set");
    std::fs::copy(testdata("categories.toml"), dir.join(CATEGORIES_FILE)).expect("copy rules");
    let state = app_state(&dir);

    // A batch packed with `ramadhan` ...
    read_draft(
        &state,
        vec![testdata("labels-slip.pdf")],
        &Grouping::new("ramadhan"),
        |_| {},
    )
    .expect("read");
    assert_eq!(
        save_batch_impl(&state, None, false, |_| {})
            .expect("save")
            .batch,
        BatchNumber::new(1)
    );

    // ... is redone after another set was used last.
    remember_last_grouping(&dir, &Grouping::default());
    assert_eq!(
        settings_load(&dir).expect("settings").grouping,
        Grouping::default()
    );

    let start = start_amend_impl(&state, "2026-10-07", |_| {}).expect("amend");
    let rows: Vec<&str> = start
        .read
        .plan
        .rows
        .iter()
        .map(|row| row.code.as_str())
        .collect();
    assert_eq!(
        rows,
        vec!["A", "Z"],
        "the amend reads `ramadhan`, the batch's own set"
    );
    assert_eq!(
        start.read.plan.grouping,
        Grouping::new("ramadhan"),
        "the amend's Plan header names the batch's own set"
    );

    std::fs::remove_dir_all(&dir).ok();
}

/// The Plan screen's *Back* is a window move: the back end keeps the draft and its files, and
/// a new *Read labels* replaces the draft from the files, so no plan edit survives.
#[test]
fn back_keeps_the_draft_and_a_new_read_replaces_its_plan_edits() {
    let Some(_vendor) = vendor_dir() else {
        eprintln!("skipped: run tools/get-pdfium.sh");
        return;
    };
    let dir = temp_dir("plan-back");
    std::fs::copy(testdata("categories.toml"), dir.join(CATEGORIES_FILE)).expect("copy rules");
    let state = app_state(&dir);

    let read = read_draft(
        &state,
        vec![testdata("labels-slip.pdf")],
        &Grouping::default(),
        |_| {},
    )
    .expect("read");
    let numbers: Vec<Option<PdfNumber>> = read.plan.rows.iter().map(|row| row.number).collect();
    assert_eq!(
        numbers,
        vec![
            Some(PdfNumber::new(1)),
            Some(PdfNumber::new(2)),
            Some(PdfNumber::new(3)),
            Some(PdfNumber::new(4)),
            Some(PdfNumber::new(5)),
        ]
    );
    assert_eq!(read.plan.rows[0].file_name, "1 A Sepatu ×7.pdf");

    // *Back* changes nothing here: the draft is still open with its one file and its orders.
    state
        .with_draft(|open| {
            assert_eq!(open.draft.files.len(), 1);
            assert_eq!(open.draft.orders.len(), read.orders);
            assert!(open.amend.is_none());
        })
        .expect("the draft is kept when *Back* is pressed");

    // A plan edit on the kept draft: the first saved PDF is renamed for this batch only.
    let edited = state
        .with_draft(|open| {
            apply_edits(
                &mut open.plan,
                &[PlanEdit::Rename {
                    index: 0,
                    name: "renamed.pdf".to_string(),
                }],
            )?;
            let first_number = open.first_number;
            Ok::<_, String>(plan_view(
                &open.plan,
                &open.draft.orders,
                first_number,
                &open.grouping,
            ))
        })
        .expect("draft")
        .expect("rename");
    assert_eq!(edited.rows[0].file_name, "renamed.pdf");

    // A new read replaces the draft whole: the plan is the files' plan again.
    let again = read_draft(
        &state,
        vec![testdata("labels-slip.pdf")],
        &Grouping::default(),
        |_| {},
    )
    .expect("read again");
    let numbers: Vec<Option<PdfNumber>> = again.plan.rows.iter().map(|row| row.number).collect();
    assert_eq!(
        numbers,
        vec![
            Some(PdfNumber::new(1)),
            Some(PdfNumber::new(2)),
            Some(PdfNumber::new(3)),
            Some(PdfNumber::new(4)),
            Some(PdfNumber::new(5)),
        ]
    );
    assert_eq!(
        again.plan.rows[0].file_name, "1 A Sepatu ×7.pdf",
        "no plan edit survives a new read"
    );

    std::fs::remove_dir_all(&dir).ok();
}
