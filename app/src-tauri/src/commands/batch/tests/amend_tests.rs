//! Revert, amend and the amend's selection page on `testdata/`.

use super::*;

/// The orders a plan view puts in its rows, as the Plan screen's counts show them.
fn plan_orders(view: &PlanView) -> usize {
    view.rows.iter().map(|row| row.orders).sum()
}

/// Split `labels-slip.pdf` into three files — one order in the first, one in the second, the
/// rest in the third — so an amendment's selection page has files of its own to take off the
/// list and to add ([`the_amend_read_takes_the_selection_as_it_is`]). A split never falls
/// inside a multi-page order, so each part reads cleanly on its own.
fn split_testdata_labels(worker: &PdfWorker, dir: &Path) -> Vec<PathBuf> {
    let source = testdata("labels-slip.pdf");
    worker
        .open(std::slice::from_ref(&source))
        .expect("open the fixture");
    let cancel = Arc::new(AtomicBool::new(false));
    let (progress, _rx) = mpsc::channel();
    let items = worker
        .read(
            cancel,
            progress,
            packing_engine::label::courier_data().clone(),
        )
        .expect("read the fixture");
    // The first page of every order; a page without an Order ID continues the previous one.
    let starts: Vec<usize> = items
        .iter()
        .enumerate()
        .filter(|(_, page)| packing_engine::label::find_order_id(&page.2.text).is_some())
        .map(|(page, _)| page)
        .collect();
    assert!(starts.len() > 2, "the fixture carries more than two orders");

    let bounds = [0usize, starts[1], starts[2], items.len()];
    let mut parts = Vec::with_capacity(3);
    for part in 0..3 {
        let path = dir.join(format!("part{}.pdf", part + 1));
        let pages: Vec<(usize, usize)> = (bounds[part]..bounds[part + 1])
            .map(|page| (0, page))
            .collect();
        worker.write_pages(&pages, &path).expect("write a part");
        parts.push(path);
    }
    parts
}

#[test]
fn revert_last_deletes_the_folder_and_the_state() {
    let Some(_vendor) = vendor_dir() else {
        eprintln!("skipped: run tools/get-pdfium.sh");
        return;
    };
    let dir = temp_dir("revert");
    std::fs::copy(testdata("categories.toml"), dir.join(CATEGORIES_FILE)).expect("copy rules");
    let (state, _guard) = app_state(&dir);

    read_draft(
        &state,
        vec![testdata("labels-slip.pdf")],
        &Grouping::default(),
        |_| {},
    )
    .expect("read");
    let save = save_batch_impl(&state, None, false, |_| {}).expect("save");
    assert_eq!(save.batch, BatchNumber::new(1));
    let folder = dir.join("labels/2026-10-07/batch 1");
    assert!(folder.is_dir());

    revert_last_impl(&state, "2026-10-07").expect("revert");
    assert!(!folder.exists(), "the batch folder is gone");
    let day_state = DayState::load(&dir.join("labels/2026-10-07")).expect("state");
    assert!(day_state.batches.is_empty());
    assert!(day_state.saved.is_empty());

    // The numbers are reused: the next batch is batch 1 again.
    read_draft(
        &state,
        vec![testdata("labels-slip.pdf")],
        &Grouping::default(),
        |_| {},
    )
    .expect("read again");
    let save = save_batch_impl(&state, None, false, |_| {}).expect("save again");
    assert_eq!(save.batch, BatchNumber::new(1));

    // No batch to undo on an untouched day.
    assert!(revert_last_impl(&state, "2026-10-08").is_err());

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn amend_reads_the_download_copy_and_replaces_the_batch() {
    let Some(_vendor) = vendor_dir() else {
        eprintln!("skipped: run tools/get-pdfium.sh");
        return;
    };
    let dir = temp_dir("amend");
    std::fs::copy(testdata("categories.toml"), dir.join(CATEGORIES_FILE)).expect("copy rules");
    let (state, _guard) = app_state(&dir);

    read_draft(
        &state,
        vec![testdata("labels-slip.pdf")],
        &Grouping::default(),
        |_| {},
    )
    .expect("read");
    assert_eq!(
        save_batch_impl(&state, None, false, |_| {})
            .expect("save")
            .batch,
        BatchNumber::new(1)
    );
    let day_dir = dir.join("labels").join("2026-10-07");
    let folder = day_dir.join("batch 1");
    assert!(folder.join("download").is_dir());

    // *Amend* reads the batch's own download copy at once and opens the Plan screen: its file
    // list comes back for the selection page, the read holds the batch's own orders (none seen
    // as already saved) and the draft is open.
    let start = start_amend_impl(&state, "2026-10-07", |_| {}).expect("amend");
    assert_eq!(start.batch, BatchNumber::new(1));
    assert_eq!(start.files.len(), 1);
    assert_eq!(start.files[0].name, "labels-slip.pdf");
    assert_eq!(start.read.amend, Some(BatchNumber::new(1)));
    assert_eq!(start.read.orders, 23);
    assert_eq!(start.read.already_saved, 0);
    state.with_draft(|_| ()).expect("the amend draft is open");

    // Saving again replaces batch 1 (same number, one batch in the state).
    assert_eq!(
        save_batch_impl(&state, None, false, |_| {})
            .expect("save again")
            .batch,
        BatchNumber::new(1)
    );
    let day_state = DayState::load(&day_dir).expect("state");
    assert_eq!(day_state.batches.len(), 1);
    assert_eq!(day_state.batches[0].batch, BatchNumber::new(1));
    assert!(folder.is_dir());

    // Cancel leaves the old batch untouched.
    start_amend_impl(&state, "2026-10-07", |_| {}).expect("amend again");
    state.clear_draft();
    assert!(folder.is_dir());
    assert_eq!(DayState::load(&day_dir).expect("state").batches.len(), 1);

    std::fs::remove_dir_all(&dir).ok();
}

/// **Amend** on the newest batch reads the batch's own `download/` files in
/// the `state.json` order and opens the Plan screen at once — the picks first, not the PDF
/// selection page — with the draft open, leaving `state.json` byte-identical until **Save
/// batch 1 again**. The batch's file list comes back for the *Back* selection page, listed
/// already selected in that order.
#[test]
fn amend_starts_on_the_plan_reads_the_batchs_files_and_writes_nothing() {
    let Some(_vendor) = vendor_dir() else {
        eprintln!("skipped: run tools/get-pdfium.sh");
        return;
    };
    let dir = temp_dir("amend-start");
    std::fs::copy(testdata("categories.toml"), dir.join(CATEGORIES_FILE)).expect("copy rules");
    let (state, _guard) = app_state(&dir);
    let parts = state
        .with_worker(|worker| split_testdata_labels(worker, &dir))
        .expect("worker");

    read_draft(&state, parts[..2].to_vec(), &Grouping::default(), |_| {}).expect("read");
    assert_eq!(
        save_batch_impl(&state, None, false, |_| {})
            .expect("save")
            .batch,
        BatchNumber::new(1)
    );
    let day_dir = dir.join("labels").join("2026-10-07");
    let download = day_dir.join("batch 1").join("download");
    let before = std::fs::read(day_dir.join("state.json")).expect("state.json");

    // *Amend* opens the Plan screen: the batch's own orders read again, none seen as already
    // saved, and the draft open.
    let start = start_amend_impl(&state, "2026-10-07", |_| {}).expect("amend");
    assert_eq!(start.batch, BatchNumber::new(1));
    assert_eq!(start.read.amend, Some(BatchNumber::new(1)));
    assert_eq!(start.read.files, 2);
    assert_eq!(start.read.orders, 2);
    assert_eq!(
        start.read.already_saved, 0,
        "the batch's own orders are re-read"
    );
    assert_eq!(plan_orders(&start.read.plan), 2);
    state
        .with_draft(|open| {
            assert_eq!(
                open.amend.as_ref().map(|amend| amend.batch),
                Some(BatchNumber::new(1))
            )
        })
        .expect("the amend draft is open");

    // The *Back* selection page's list: the batch's `download/` copies, already selected in
    // `state.json` order, with their page counts and no *used in batch 1* mark of their own.
    let listed: Vec<&str> = start.files.iter().map(|file| file.name.as_str()).collect();
    assert_eq!(listed, vec!["part1.pdf", "part2.pdf"]);
    assert!(start.files.iter().all(|file| file.pages > 0));
    assert!(
        start
            .files
            .iter()
            .all(|file| file.path == download.join(&file.name).display().to_string()),
        "the files are the batch's `download/` copies"
    );
    assert!(
        start.files.iter().all(|file| file.used_in_batch.is_none()),
        "the batch's own files are not marked *used in batch*"
    );
    assert_eq!(
        std::fs::read(day_dir.join("state.json")).expect("state.json"),
        before,
        "state.json is byte-identical until Save"
    );

    // *Save batch 1 again* replaces the batch: one batch, the same number.
    assert_eq!(
        save_batch_impl(&state, None, false, |_| {})
            .expect("save again")
            .batch,
        BatchNumber::new(1)
    );
    let day_state = DayState::load(&day_dir).expect("state");
    assert_eq!(day_state.batches.len(), 1);
    assert_eq!(day_state.batches[0].batch, BatchNumber::new(1));
    assert!(
        day_state
            .saved
            .iter()
            .all(|order| order.batch == BatchNumber::new(1))
    );
    assert_eq!(day_state.batches[0].files.len(), 2);
    assert!(day_dir.join("batch 1").is_dir());

    std::fs::remove_dir_all(&dir).ok();
}

/// **Read labels** on the selection page reads the
/// selection as it stands — a file taken off the list leaves its orders out of the plan (they
/// stay unsaved and the next batch reads them again), and an added file joins with its orders.
#[test]
fn the_amend_read_takes_the_selection_as_it_is() {
    let Some(_vendor) = vendor_dir() else {
        eprintln!("skipped: run tools/get-pdfium.sh");
        return;
    };
    let dir = temp_dir("amend-selection-read");
    std::fs::copy(testdata("categories.toml"), dir.join(CATEGORIES_FILE)).expect("copy rules");
    let (state, _guard) = app_state(&dir);
    let parts = state
        .with_worker(|worker| split_testdata_labels(worker, &dir))
        .expect("worker");

    // Batch 1 holds the first two files: one order each.
    let read = read_draft(&state, parts[..2].to_vec(), &Grouping::default(), |_| {}).expect("read");
    assert_eq!(read.orders, 2);
    assert_eq!(plan_orders(&read.plan), 2);
    assert_eq!(
        save_batch_impl(&state, None, false, |_| {})
            .expect("save")
            .batch,
        BatchNumber::new(1)
    );
    let day_dir = dir.join("labels/2026-10-07");
    let download = day_dir.join("batch 1").join("download");

    // The batch's own two files again: both orders, none of them 'already saved'.
    let whole = read_amend_draft(
        &state,
        vec![download.join("part1.pdf"), download.join("part2.pdf")],
        BatchNumber::new(1),
        |_| {},
    )
    .expect("amend read");
    assert_eq!(whole.files, 2);
    assert_eq!(whole.orders, 2);
    assert_eq!(whole.already_saved, 0);
    assert_eq!(plan_orders(&whole.plan), 2);

    // **×** on the second file: its order is out of the plan and is not
    // saved by the amendment.
    let kept = read_amend_draft(
        &state,
        vec![download.join("part1.pdf")],
        BatchNumber::new(1),
        |_| {},
    )
    .expect("amend read");
    assert_eq!(kept.files, 1);
    assert_eq!(kept.orders, 1, "the taken-off file's order is gone");
    assert_eq!(plan_orders(&kept.plan), 1);

    // **Add files…** (: any PDF on the PC): the added file joins with its
    // orders.
    let added = read_amend_draft(
        &state,
        vec![
            download.join("part1.pdf"),
            download.join("part2.pdf"),
            parts[2].clone(),
        ],
        BatchNumber::new(1),
        |_| {},
    )
    .expect("amend read");
    assert_eq!(added.files, 3);
    assert_eq!(added.orders, 23, "the added file's orders join the plan");
    assert_eq!(plan_orders(&added.plan), 23);

    std::fs::remove_dir_all(&dir).ok();
}

/// *Cancel* on the selection page and *Cancel amend* discard the amendment and
/// nothing else: the batch folder and `state.json` are left exactly as they were, byte for
/// byte, and neither asks.
#[test]
fn cancel_amend_leaves_the_batch_and_the_state_untouched() {
    let Some(_vendor) = vendor_dir() else {
        eprintln!("skipped: run tools/get-pdfium.sh");
        return;
    };
    let dir = temp_dir("cancel-amend");
    std::fs::copy(testdata("categories.toml"), dir.join(CATEGORIES_FILE)).expect("copy rules");
    let (state, _guard) = app_state(&dir);

    read_draft(
        &state,
        vec![testdata("labels-slip.pdf")],
        &Grouping::default(),
        |_| {},
    )
    .expect("read");
    assert_eq!(
        save_batch_impl(&state, None, false, |_| {})
            .expect("save")
            .batch,
        BatchNumber::new(1)
    );
    let day_dir = dir.join("labels/2026-10-07");
    let folder = day_dir.join("batch 1");
    let before = std::fs::read(day_dir.join("state.json")).expect("state.json");

    // The Plan screen's *Cancel amend*: *Amend* opened the amend draft at once, and the
    // `discard_draft` command's body drops it without writing anything.
    let start = start_amend_impl(&state, "2026-10-07", |_| {}).expect("amend");
    assert_eq!(start.batch, BatchNumber::new(1));
    state.with_draft(|_| ()).expect("the amend draft is open");
    state.clear_draft();
    assert!(state.with_draft(|_| ()).is_err(), "the draft is gone");
    assert!(folder.is_dir(), "the batch folder is untouched");
    assert_eq!(
        std::fs::read(day_dir.join("state.json")).expect("state.json"),
        before,
        "state.json is untouched"
    );

    // After *Back* and the selection page's *Read labels*, *Cancel* there too discards the
    // amendment's draft and leaves the old batch as it was.
    let amend = read_amend_draft(
        &state,
        vec![folder.join("download").join("labels-slip.pdf")],
        BatchNumber::new(1),
        |_| {},
    )
    .expect("amend read");
    assert_eq!(amend.amend, Some(BatchNumber::new(1)));
    state.with_draft(|_| ()).expect("the amend draft is open");
    state.clear_draft();

    assert!(state.with_draft(|_| ()).is_err(), "the draft is gone");
    assert!(folder.is_dir(), "the batch folder is untouched");
    assert_eq!(
        std::fs::read(day_dir.join("state.json")).expect("state.json"),
        before,
        "state.json is untouched"
    );
    assert_eq!(
        DayState::load(&day_dir).expect("state").batches.len(),
        1,
        "the batch is still saved"
    );

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn the_testdata_labels_show_no_logo_prompt_and_list_the_couriers() {
    let Some(_vendor) = vendor_dir() else {
        eprintln!("skipped: run tools/get-pdfium.sh");
        return;
    };
    let dir = temp_dir("logo-prompt");
    std::fs::copy(testdata("categories.toml"), dir.join(CATEGORIES_FILE)).expect("copy rules");
    let (state, _guard) = app_state(&dir);

    // The testdata labels carry no images, so every logo is unknown-and-absent: the prompt
    // never opens, and the buttons still carry `couriers.toml`'s names in file order.
    let read = read_draft(
        &state,
        vec![testdata("labels-slip.pdf")],
        &Grouping::default(),
        |_| {},
    )
    .expect("read");
    assert!(read.logos.is_empty(), "the fixtures have no images");
    assert_eq!(
        read.couriers,
        vec!["J&T Express", "SiCepat", "J&T Cargo", "IDX", "SPX"]
    );

    // An amend read goes through the same path: *Amend* reads the batch's own files and ends
    // with no prompt and the same couriers.
    assert_eq!(
        save_batch_impl(&state, None, false, |_| {})
            .expect("save")
            .batch,
        BatchNumber::new(1)
    );
    let start = start_amend_impl(&state, "2026-10-07", |_| {}).expect("amend");
    assert_eq!(start.batch, BatchNumber::new(1));
    assert_eq!(start.read.amend, Some(BatchNumber::new(1)));
    assert!(start.read.logos.is_empty());
    assert_eq!(start.read.couriers, read.couriers);

    // Answering a hash no logo carries changes nothing and still returns a full result.
    let again = answer_logo_impl(&state, "ffffffffffffffff", &LogoAnswer::Skip).expect("answer");
    assert_eq!(again.orders, start.read.orders);
    assert!(again.logos.is_empty());

    std::fs::remove_dir_all(&dir).ok();
}
