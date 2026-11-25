use std::collections::BTreeMap;
use std::fs;
use std::time::SystemTime;

use jiff::civil::date;
use packing_engine::record::RecordBody;
use packing_store::{SESSION_TOML, open_date};
use testkit::TempDir;

use super::*;
use crate::commands::day::overview;
use crate::commands::start::{start_information, use_folder};
use crate::{READ_ONLY, settings};

const DAY: &str = "2026-10-07";

/// A program folder naming a data folder of an older version: one date with a batch saved before
/// the area was read, its label PDF not kept, and views of schema 2.1.
fn older_folder(tag: &str) -> (TempDir, AppState, PathBuf) {
    let program = TempDir::new(tag);
    let dir = settings::set_data_dir(program.path(), &program.path().join("data")).unwrap();
    let folder = DateFolder::new(&dir, date(2026, 10, 7));
    let mut saved = testkit::batch(1, false, Vec::new());
    for order in &mut saved.orders {
        order.area = None;
    }
    let mut journal = Journal::open(&folder).unwrap();
    journal
        .commit(vec![RecordBody::BatchSaved(saved)], APP_VERSION)
        .unwrap();
    open_date(&folder).unwrap();
    let session = folder.dir().join(SESSION_TOML);
    let text = fs::read_to_string(&session).unwrap();
    fs::write(
        &session,
        text.replace("schema = \"2.2\"", "schema = \"2.1\""),
    )
    .unwrap();
    fs::remove_file(folder.dir().join("orders.csv")).unwrap();
    let state = AppState::new(program.path().to_path_buf());
    (program, state, dir)
}

/// Every file under `dir` with its bytes and modify time.
fn files(dir: &Path) -> BTreeMap<PathBuf, (Vec<u8>, SystemTime)> {
    let mut found = BTreeMap::new();
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            found.extend(files(&path));
        } else {
            let modified = fs::metadata(&path).unwrap().modified().unwrap();
            found.insert(path.clone(), (fs::read(&path).unwrap(), modified));
        }
    }
    found
}

#[test]
fn the_start_asks_first_and_read_only_writes_nothing() {
    let (_program, state, dir) = older_folder("app-read-only");
    let before = files(&dir);
    let info = start_information(&state).unwrap();
    assert_eq!(
        info.migration,
        [MigrationView {
            date: DAY.into(),
            older_views: true,
            batches: 0,
            orders: 0,
        }],
        "a batch whose label PDF is gone cannot be read"
    );
    assert!(!state.has_data(), "no date opens before the owner answers");
    let chosen = use_folder(&state, &dir).unwrap();
    assert_eq!(
        chosen.migration, info.migration,
        "choosing the folder asks too"
    );
    assert!(!state.has_data());

    open_read_only(&state).unwrap();
    assert!(start_information(&state).unwrap().read_only);
    let day = serde_json::to_value(overview(&state, DAY).unwrap()).unwrap();
    assert_eq!(day["totals"]["batches"], 1);
    assert_eq!(state.write_data(|_, _| Ok(())), Err(READ_ONLY.to_string()));
    assert_eq!(
        files(&dir),
        before,
        "no lock, no view, no parser.toml or config.toml"
    );
}

#[test]
fn update_writes_the_views_again_and_opens_the_folder() {
    let _guard = testkit::pdfium_lock();
    let (_program, state, dir) = older_folder("app-update");
    assert!(state.ensure_worker() || testkit::vendor_dir().is_none());
    let (steps, sent) = mpsc::channel();
    let updated = update(&state, steps).unwrap();
    assert_eq!(updated.len(), 1);
    assert_eq!(updated[0].date, DAY);
    assert!(matches!(
        sent.try_iter().last(),
        Some(MigrationStep::Updated(_))
    ));
    let session = fs::read_to_string(dir.join(DAY).join(SESSION_TOML)).unwrap();
    assert!(session.starts_with("schema = \"2.2\""), "{session}");
    assert!(dir.join(DAY).join("orders.csv").is_file());
    assert!(pending(&dir).unwrap().is_empty());
    assert!(!start_information(&state).unwrap().read_only);
    assert!(state.write_data(|_, _| Ok(())).is_ok());
}
