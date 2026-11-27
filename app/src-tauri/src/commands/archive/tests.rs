use std::fs;

use jiff::ToSpan;
use packing_engine::ids::{BatchId, PdfId};
use packing_engine::record::{OutputFile, RecordBody};
use packing_store::{DateFolder, Journal, input_path, sha256_of};
use serde_json::Value;
use testkit::{TempDir, batch, sha};

use super::*;
use crate::ARCHIVED;
use crate::commands::day::{overview, revert};
use crate::commands::panel::panel;
use crate::commands::start::use_folder;
use crate::settings::{self, DEFAULT_DATA_DIR, Theme};

#[test]
fn the_dates_older_than_a_number_of_days_are_counted_from_today() {
    let today = date(2026, 11, 27);
    let all = [
        date(2026, 9, 27),
        date(2026, 9, 28),
        date(2026, 9, 29),
        date(2026, 11, 26),
        today,
    ];
    for (days, older) in [
        (60, vec![date(2026, 9, 27)]),
        (59, vec![date(2026, 9, 27), date(2026, 9, 28)]),
        (1, all[..3].to_vec()),
        (0, all[..4].to_vec()),
        (400, Vec::new()),
    ] {
        assert_eq!(older_than(&all, today, days), older, "{days} days");
    }
}

fn date(year: i16, month: i8, day: i8) -> Date {
    jiff::civil::date(year, month, day)
}

/// Batch 1 saved `days_ago` days before today, with its input and its two outputs in its folder.
fn saved_date(data_dir: &std::path::Path, days_ago: i64) -> Date {
    let date = today().saturating_sub(days_ago.days());
    let folder = DateFolder::new(data_dir, date);
    fs::create_dir_all(folder.dir()).unwrap();
    let output = |file: &str, group: Option<&str>| {
        let path = folder.dir().join(file);
        fs::write(&path, format!("%PDF {file}")).unwrap();
        OutputFile {
            file: file.into(),
            group: group.map(str::to_string),
            pages: 1,
            size: fs::metadata(&path).unwrap().len(),
            sha256: sha256_of(&path).unwrap(),
        }
    };
    let outputs = vec![
        output("1-A Sepatu x2.pdf", Some("A")),
        output("1 Summary.pdf", None),
    ];
    let input = input_path(&folder, BatchId::new(1), PdfId::new(1), &sha('a'));
    fs::write(input, b"%PDF made up").unwrap();
    Journal::open(&folder)
        .unwrap()
        .commit(
            vec![RecordBody::BatchSaved(batch(1, false, outputs))],
            "0.3.3",
        )
        .unwrap();
    date
}

/// A data folder with a date saved 100 days ago, one 10 days ago and today, opened.
fn three_dates(tag: &str) -> (TempDir, AppState, [Date; 3]) {
    let program = TempDir::new(tag);
    let data_dir = program.path().join(DEFAULT_DATA_DIR);
    let dates = [100, 10, 0].map(|days_ago| saved_date(&data_dir, days_ago));
    let state = AppState::new(program.path().to_path_buf());
    use_folder(&state, &data_dir).unwrap();
    state.index().wait();
    (program, state, dates)
}

fn panel_row(state: &AppState, date: Date) -> Value {
    let shown = serde_json::to_value(panel(state).unwrap()).unwrap();
    shown["dates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["date"] == date.to_string())
        .unwrap()
        .clone()
}

fn range(date: Date) -> ArchiveScope {
    ArchiveScope::Range {
        from: date.to_string(),
        to: date.to_string(),
    }
}

#[test]
fn an_old_date_is_archived_and_shown_without_its_pdfs() {
    let (_program, state, [old, recent, today]) = three_dates("app-archive");
    overview(&state, &old.to_string()).unwrap();
    let before = panel_row(&state, old);
    assert_eq!(before["archived"], false);
    assert!(before["size_bytes"].as_u64().unwrap() > 0);

    let asked = preview(&state, &ArchiveScope::OlderThan { days: 60 }).unwrap();
    assert_eq!(asked.dates, [old.to_string()]);
    assert_eq!(asked.pdfs, 3, "the input and two outputs");
    let range_of_all = ArchiveScope::Range {
        from: old.to_string(),
        to: today.to_string(),
    };
    let all = preview(&state, &range_of_all).unwrap();
    assert_eq!(
        all.dates,
        [old.to_string(), recent.to_string()],
        "today is passed over"
    );

    let steps = std::sync::Mutex::new(Vec::new());
    let done = archive_dates(&state, &range(old), &|step| {
        steps
            .lock()
            .unwrap()
            .push((step.date, step.number, step.of));
    })
    .unwrap();
    assert_eq!(done.archived, [old.to_string()]);
    assert!(done.left.is_empty());
    assert_eq!(*steps.lock().unwrap(), [(old.to_string(), 1, 1)]);

    let after = panel_row(&state, old);
    assert_eq!(after["archived"], true);
    assert!(after["size_bytes"].as_u64() < before["size_bytes"].as_u64());
    let shown = serde_json::to_value(overview(&state, &old.to_string()).unwrap()).unwrap();
    assert_eq!(shown["archived"], true);
    assert_eq!(
        shown["problems"],
        serde_json::json!([]),
        "no missing input is named"
    );
    assert_eq!(shown["batches"][0]["pdfs"][0]["problem"], Value::Null);
    assert_eq!(
        revert(&state, &old.to_string(), BatchId::new(1), None),
        Err(ARCHIVED.to_string())
    );
    assert!(
        preview(&state, &range(old)).unwrap().dates.is_empty(),
        "archived already"
    );

    let used = storage_used(&state).unwrap();
    assert_eq!((used.dates, used.archived), (3, Some(1)));
}

#[test]
fn read_only_archives_nothing() {
    let (_program, state, [old, ..]) = three_dates("app-archive-read-only");
    state
        .with_data(|data| {
            data.read_only = true;
            Ok(())
        })
        .unwrap();
    let refused = archive_dates(&state, &range(old), &|_| {}).unwrap_err();
    assert_eq!(refused, crate::READ_ONLY);
    assert_eq!(panel_row(&state, old)["archived"], false);
}

#[test]
fn archive_after_days_is_written_and_removed_with_the_other_settings_kept() {
    let program = TempDir::new("app-archive-setting");
    let dir = program.path().join(DEFAULT_DATA_DIR);
    settings::set_data_dir(program.path(), &dir).unwrap();
    settings::set_theme(program.path(), Theme::Dark).unwrap();
    assert_eq!(settings::archive_after_days(program.path()).unwrap(), None);
    for days in [Some(60), Some(1), None] {
        settings::set_archive_after_days(program.path(), days).unwrap();
        assert_eq!(settings::archive_after_days(program.path()).unwrap(), days);
        assert_eq!(settings::theme(program.path()).unwrap(), Theme::Dark);
        assert_eq!(
            settings::data_dir(program.path()).unwrap(),
            Some(dir.clone())
        );
    }
    let written = fs::read_to_string(program.path().join(settings::APP_FILE)).unwrap();
    assert!(!written.contains("archive_after_days"), "{written}");
}
