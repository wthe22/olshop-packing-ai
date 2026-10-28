use jiff::civil::date;
use packing_engine::record::RecordBody;
use packing_store::{DateFolder, Journal};
use testkit::{TempDir, batch};

use super::*;
use crate::commands::start::{new_workspace, use_folder};

fn names(state: &AppState) -> Vec<(String, usize, bool)> {
    overview(state)
        .unwrap()
        .into_iter()
        .map(|view| (view.name, view.dates, view.open))
        .collect()
}

fn open(state: &AppState) -> Option<String> {
    state.with_data(|data| Ok(data.workspace.clone())).unwrap()
}

// The deleted folders go to this PC's Recycle Bin: two empty test folders per run.
#[test]
fn a_workspace_is_renamed_or_deleted_and_the_open_one_follows() {
    let program = TempDir::new("app-workspaces");
    let state = AppState::new(program.path().to_path_buf());
    let dir = program.path().join("data");
    use_folder(&state, &dir).unwrap();
    new_workspace(&state, "Toko Contoh").unwrap();
    Journal::open(&DateFolder::new(&dir, "Toko Contoh", date(2026, 10, 7)))
        .unwrap()
        .commit(
            vec![RecordBody::BatchSaved(batch(1, false, Vec::new()))],
            "0.2.0",
        )
        .unwrap();
    new_workspace(&state, "Toko Lain").unwrap();
    let both = [("Toko Contoh", 1, false), ("Toko Lain", 0, true)];
    assert_eq!(names(&state), both.map(|(n, d, o)| (n.to_string(), d, o)));

    assert!(rename(&state, "Toko Lain", "toko contoh").is_err());
    rename(&state, "Toko Lain", "Toko Baru").unwrap();
    rename(&state, "Toko Contoh", "Toko Satu").unwrap();
    assert_eq!(open(&state).as_deref(), Some("Toko Baru"));
    assert_eq!(
        names(&state),
        [
            ("Toko Baru".to_string(), 0, true),
            ("Toko Satu".to_string(), 1, false)
        ]
    );

    assert_eq!(
        delete(&state, "Toko Baru").unwrap().as_deref(),
        Some("Toko Satu"),
        "the workspace written last opens"
    );
    assert_eq!(open(&state).as_deref(), Some("Toko Satu"));
    assert!(delete(&state, "Toko Tiada").is_err());
    assert_eq!(delete(&state, "Toko Satu").unwrap(), None);
    assert!(names(&state).is_empty());
}
