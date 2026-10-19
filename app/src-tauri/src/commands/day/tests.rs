use super::*;
use packing_engine::day::{BatchEntry, FileEntry, PackingListChoice, PdfEntry};

fn temp_dir(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("packing-app-test-{}-{label}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}

fn file_entry(name: &str) -> FileEntry {
    FileEntry {
        name: name.to_string(),
        size: 1,
        xxh3: "00".to_string(),
        pages: 1,
        legacy_sha256: None,
    }
}

fn batch_entry(
    batch: i64,
    time: &str,
    pdf: PdfEntry,
    warnings: Vec<String>,
    errors: Vec<String>,
) -> BatchEntry {
    BatchEntry {
        batch,
        save_time: time.to_string(),
        set: "default".to_string(),
        files: vec![file_entry("a.pdf")],
        pdfs: vec![pdf],
        packing_list: PackingListChoice::default(),
        printed: Vec::new(),
        warnings,
        errors,
    }
}

#[test]
fn settings_default_and_round_trip() {
    let dir = temp_dir("settings");
    assert_eq!(settings_load(&dir).expect("default"), Settings::default());

    let custom = Settings {
        scope: Scope::PerPdf,
        layouts: vec![Layout::Full, Layout::Summary],
        set: "ramadhan".to_string(),
    };
    settings_save(&dir, &custom).expect("save");
    assert_eq!(settings_load(&dir).expect("load"), custom);
    assert_eq!(custom.packing_choice().scope, Scope::PerPdf);

    // A missing key takes its default: an older file without the set loads as `default`
    // (07 › *Settings*; 04 › *Sets*).
    std::fs::write(dir.join(SETTINGS_FILE), "{\n  \"scope\": \"none\"\n}\n").expect("write");
    let partial = settings_load(&dir).expect("partial");
    assert_eq!(partial.scope, Scope::None);
    assert_eq!(partial.layouts, vec![Layout::Pick]);
    assert_eq!(partial.set, packing_engine::sets::DEFAULT_SET);

    std::fs::remove_dir_all(&dir).ok();
}

/// The picks set used last is remembered in `settings.json` (07 › *Settings*; 04 › *Sets*), so
/// the next New batch preselects it; a read with the same set writes nothing, and a folder
/// whose settings cannot be read is left alone.
#[test]
fn the_last_used_set_is_remembered() {
    let dir = temp_dir("settings-set");
    // No file yet: the default, and remembering a set writes it.
    assert_eq!(settings_load(&dir).expect("default").set, "default");
    remember_last_set(&dir, "ramadhan");
    assert_eq!(settings_load(&dir).expect("load").set, "ramadhan");

    // The packing-list keys are kept (the file is written whole).
    remember_last_set(&dir, "default");
    let settings = settings_load(&dir).expect("load again");
    assert_eq!(settings.set, "default");
    assert_eq!(settings.scope, Scope::Whole);
    assert_eq!(settings.layouts, vec![Layout::Pick]);

    // A broken settings file is not touched.
    std::fs::write(dir.join(SETTINGS_FILE), "not json").expect("write broken");
    remember_last_set(&dir, "ramadhan");
    assert_eq!(
        std::fs::read_to_string(dir.join(SETTINGS_FILE)).expect("read"),
        "not json"
    );

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn overview_reads_state_newest_first_totals_and_warnings() {
    let dir = temp_dir("overview");
    let day_dir = dir.join(LABELS_DIR).join("2026-10-07");
    let mut state = DayState::new("2026-10-07");
    state.batches.push(batch_entry(
        1,
        "2026-10-07T07:40:12+07:00",
        PdfEntry {
            number: 1,
            code: "A".to_string(),
            name: "Sepatu".to_string(),
            file: "1 A Sepatu ×172.pdf".to_string(),
            orders: 172,
            runs: 4,
        },
        vec!["no tracking ID for: 580000000000000004".to_string()],
        Vec::new(),
    ));
    state.batches.push(batch_entry(
        2,
        "2026-10-07T09:12:00+07:00",
        PdfEntry {
            number: 2,
            code: "Z".to_string(),
            name: "Lainnya".to_string(),
            file: "2 Z Lainnya ×10.pdf".to_string(),
            orders: 10,
            runs: 2,
        },
        Vec::new(),
        vec!["PDF 2 is missing a page".to_string()],
    ));
    state.save(&day_dir).expect("save state");

    let overview = day_overview_at(&dir, "2026-10-07").expect("overview");
    assert_eq!(overview.day, "2026-10-07");
    assert_eq!(overview.batches.len(), 2);
    assert_eq!(overview.batches[0].batch, 2); // newest on top
    assert_eq!(overview.batches[0].time, "09:12");
    assert_eq!(overview.batches[0].orders, 10);
    assert!(overview.batches[0].warnings.is_empty());
    // The check before saving's errors reach the Day screen's view.
    assert_eq!(
        overview.batches[0].errors,
        vec!["PDF 2 is missing a page".to_string()]
    );
    assert_eq!(overview.batches[0].packing_lists, vec!["packing-list.pdf"]);
    assert!(overview.batches[0].folder.ends_with("batch 2"));
    assert_eq!(
        overview.batches[1].warnings,
        vec!["no tracking ID for: 580000000000000004".to_string()]
    );
    assert!(overview.batches[1].errors.is_empty(), "this batch had none");
    assert_eq!(overview.totals.batches, 2);
    assert_eq!(overview.totals.orders, 182);
    assert_eq!(overview.totals.pdfs, 2);

    // A missing day folder is an empty overview.
    let empty = day_overview_at(&dir, "2026-10-08").expect("empty");
    assert!(empty.batches.is_empty());
    assert_eq!(empty.totals.orders, 0);

    std::fs::remove_dir_all(&dir).ok();
}

/// The Day screen's **Printed** tick: the write is read back by a second `day_overview` (the
/// tick survives closing and reopening), and an unknown batch or saved-PDF number changes
/// nothing (08 › *1. Day*; 07 › *`state.json`*).
#[test]
fn set_printed_writes_the_tick_and_the_overview_reports_it() {
    let dir = temp_dir("printed");
    let day_dir = dir.join(LABELS_DIR).join("2026-10-07");
    let mut state = DayState::new("2026-10-07");
    state.batches.push(batch_entry(
        1,
        "2026-10-07T07:40:12+07:00",
        PdfEntry {
            number: 1,
            code: "A".to_string(),
            name: "Sepatu".to_string(),
            file: "1 A Sepatu ×172.pdf".to_string(),
            orders: 172,
            runs: 4,
        },
        Vec::new(),
        Vec::new(),
    ));
    state.save(&day_dir).expect("save state");

    // Before: the row is unticked.
    let before = day_overview_at(&dir, "2026-10-07").expect("overview");
    assert!(!before.batches[0].pdfs[0].printed);

    // Ticking writes the file; a fresh `day_overview` reports the tick.
    set_printed_at(&dir, "2026-10-07", 1, 1, true).expect("tick");
    let after = day_overview_at(&dir, "2026-10-07").expect("overview again");
    assert!(after.batches[0].pdfs[0].printed);

    // Unticking is read back too.
    set_printed_at(&dir, "2026-10-07", 1, 1, false).expect("untick");
    let after = day_overview_at(&dir, "2026-10-07").expect("overview again");
    assert!(!after.batches[0].pdfs[0].printed);

    // An unknown batch or saved-PDF number changes nothing.
    set_printed_at(&dir, "2026-10-07", 9, 1, true).expect("unknown batch");
    set_printed_at(&dir, "2026-10-07", 1, 9, true).expect("unknown pdf");
    let after = day_overview_at(&dir, "2026-10-07").expect("overview again");
    assert!(!after.batches[0].pdfs[0].printed);

    std::fs::remove_dir_all(&dir).ok();
}

/// The Day screen's view carries each batch's own picks set (04 › *Sets*), so **Amend**'s
/// confirmation can name it and an amend reads that set again; an older `state.json` without
/// the key reads as `default`.
#[test]
fn the_overview_reports_each_batchs_own_set() {
    let dir = temp_dir("overview-set");
    let day_dir = dir.join(LABELS_DIR).join("2026-10-07");
    let mut state = DayState::new("2026-10-07");
    let mut first = batch_entry(
        1,
        "2026-10-07T07:40:12+07:00",
        PdfEntry {
            number: 1,
            code: "Z".to_string(),
            name: "Lainnya".to_string(),
            file: "1 Z Lainnya ×10.pdf".to_string(),
            orders: 10,
            runs: 2,
        },
        Vec::new(),
        Vec::new(),
    );
    first.set = "ramadhan".to_string();
    state.batches.push(first);
    state.save(&day_dir).expect("save state");

    let overview = day_overview_at(&dir, "2026-10-07").expect("overview");
    assert_eq!(overview.batches[0].set, "ramadhan");

    // The key is defaulted: a `state.json` entry without it reads as `default`.
    let text = std::fs::read_to_string(day_dir.join("state.json")).expect("read");
    let without: String = text
        .lines()
        .filter(|line| !line.contains("\"set\""))
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(day_dir.join("state.json"), without).expect("write");
    let overview = day_overview_at(&dir, "2026-10-07").expect("overview again");
    assert_eq!(overview.batches[0].set, "default");

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn days_lists_folders_newest_first() {
    let dir = temp_dir("days");
    let labels = dir.join(LABELS_DIR);
    std::fs::create_dir_all(labels.join("2026-10-06")).expect("day 1");
    std::fs::create_dir_all(labels.join("2026-10-07")).expect("day 2");
    std::fs::write(labels.join("stray.txt"), "x").expect("stray file");

    assert_eq!(
        days_in(&dir),
        vec!["2026-10-07".to_string(), "2026-10-06".to_string()]
    );
    // No labels folder yet: no days.
    assert!(days_in(&temp_dir("days-empty")).is_empty());

    std::fs::remove_dir_all(&dir).ok();
}

/// A first start writes `couriers.toml` and no `categories.toml`; the folder is a data folder
/// from then on, and the owner's own `couriers.toml` (or one the logo prompt wrote to) is
/// never overwritten (08 › *Start*, 08 › *6. Settings*).
#[test]
fn a_first_start_writes_couriers_only_and_marks_the_folder_with_it() {
    let dir = temp_dir("first-start");
    assert!(
        !is_data_folder(&dir),
        "an empty folder is not a data folder"
    );

    // The picks file is not a marker: a folder with only `categories.toml` still asks
    // (08 › *Start* step 1).
    std::fs::write(
        dir.join(crate::CATEGORIES_FILE),
        "[[category]]\ncode = \"Z\"\nname = \"Rest\"\n",
    )
    .expect("write");
    assert!(
        !is_data_folder(&dir),
        "categories.toml is not the marker of a data folder"
    );
    std::fs::remove_file(dir.join(crate::CATEGORIES_FILE)).expect("remove");

    ensure_builtin_couriers(&dir).expect("couriers");
    assert!(is_data_folder(&dir), "couriers.toml marks the folder");
    assert_eq!(
        std::fs::read_to_string(dir.join(COURIERS_FILE)).expect("read"),
        BUILTIN_COURIERS
    );
    assert!(
        !dir.join(crate::CATEGORIES_FILE).exists(),
        "a first start writes no categories.toml"
    );

    let mine = "# mine\n[[courier]]\nname = \"X\"\nlogo = [\"0123456789abcdef\"]\n";
    std::fs::write(dir.join(COURIERS_FILE), mine).expect("write");
    ensure_builtin_couriers(&dir).expect("keep");
    assert_eq!(
        std::fs::read_to_string(dir.join(COURIERS_FILE)).expect("read"),
        mine
    );

    std::fs::remove_dir_all(&dir).ok();
}

/// *Choose a folder…* / Settings *Change…*: a folder without a `couriers.toml` gets the
/// built-in one and no picks; a folder with its own files is used as it is (08 › *Start*,
/// 08 › *6. Settings*).
#[test]
fn choosing_a_folder_writes_only_a_missing_couriers() {
    let state = AppState::new();
    let mine = "# mine\n[[courier]]\nname = \"X\"\nlogo = [\"0123456789abcdef\"]\n";

    let empty = temp_dir("chosen-empty");
    use_chosen_folder(&state, empty.to_str().expect("utf8")).expect("choose");
    assert!(empty.join(COURIERS_FILE).is_file());
    assert!(
        !empty.join(crate::CATEGORIES_FILE).exists(),
        "no picks are written"
    );

    let own = temp_dir("chosen-own");
    let rules = "[[category]]\ncode = \"Z\"\nname = \"Rest\"\n";
    std::fs::write(own.join(crate::CATEGORIES_FILE), rules).expect("write");
    std::fs::write(own.join(COURIERS_FILE), mine).expect("write");
    use_chosen_folder(&state, own.to_str().expect("utf8")).expect("choose");
    assert_eq!(
        std::fs::read_to_string(own.join(crate::CATEGORIES_FILE)).expect("read"),
        rules,
        "the folder's own picks are kept"
    );
    assert_eq!(
        std::fs::read_to_string(own.join(COURIERS_FILE)).expect("read"),
        mine
    );

    std::fs::remove_dir_all(&empty).ok();
    std::fs::remove_dir_all(&own).ok();
}

#[test]
fn about_reports_the_app_and_pdfium_versions() {
    let about = about();
    assert_eq!(about.version, version_text());
    assert_eq!(about.pdfium, PDFIUM_VERSION);
}

/// The shown version is `v` + the first two numbers of the workspace `version`, the one place
/// (08 › *6. Settings* › *The version*). The expected text is derived from the same
/// `CARGO_PKG_VERSION` here, so a hard-coded `v0.1` in the app would pass today and fail as
/// soon as the workspace `version` changes — which is exactly the property the owner asked
/// for: changing the number in `Cargo.toml` moves every spot.
#[test]
fn version_text_is_v_and_the_first_two_numbers_of_the_workspace_version() {
    let expected = {
        let mut numbers = env!("CARGO_PKG_VERSION").split('.');
        let major = numbers.next().expect("a major number");
        let minor = numbers.next().expect("a minor number");
        format!("v{major}.{minor}")
    };
    assert_eq!(version_text(), expected);
    // Only the first two numbers are shown (`0.1.0` → `v0.1`), never the third.
    assert_eq!(version_text().matches('.').count(), 1);
}

/// The window title and *About* both take the version from [`version_text`], the one place, so
/// they cannot differ (08 › *Window frame*, 08 › *6. Settings*). A literal in either spot
/// passes today but fails as soon as the workspace `version` changes.
#[test]
fn the_window_title_and_about_take_the_version_from_that_one_function() {
    let shown = version_text();
    assert_eq!(window_title(), format!("{PROGRAM_NAME} {shown}"));
    assert_eq!(about().version, shown);
}

/// `app/src-tauri/tauri.conf.json` carries no second copy of the version (08 › *6. Settings* ›
/// *The version*): no `version` key — Tauri then takes it from `app/src-tauri/Cargo.toml`,
/// which the app inherits from the workspace — and the window `title` holds no number, because
/// [`crate::run`] sets the real title at start.
#[test]
fn tauri_conf_has_no_version_key_and_no_number_in_the_window_title() {
    let conf: serde_json::Value = serde_json::from_str(include_str!("../../../tauri.conf.json"))
        .expect("tauri.conf.json is JSON");
    assert!(
        conf.get("version").is_none(),
        "tauri.conf.json must not carry a `version` key"
    );
    let title = conf["app"]["windows"][0]["title"]
        .as_str()
        .expect("the window title is a string");
    assert!(
        !title.chars().any(|character| character.is_ascii_digit()),
        "the window title must carry no version number: {title}"
    );
}

/// The shell calls hand Explorer the path and nothing else. `/select,<path>` is not parsed by
/// Explorer when the path holds a space — it opened *Documents* (or *Home*) instead of the
/// folder — and the batch folder always has a space in its path (review round 1).
#[test]
fn explorer_gets_the_path_as_its_only_argument() {
    let path = r"C:\packing data\labels\2026-10-11\batch 1";
    let command = explorer_command(path);
    assert_eq!(command.get_program().to_string_lossy(), "explorer");
    let arguments: Vec<String> = command
        .get_args()
        .map(|argument| argument.to_string_lossy().into_owned())
        .collect();
    assert_eq!(arguments, vec![path.to_string()]);
}
