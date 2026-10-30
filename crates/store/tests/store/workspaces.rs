//! Workspaces: which names are names, the list, the dates, the one written last.

use std::fs;

use jiff::civil::date;
use packing_engine::record::RecordBody;
use packing_store::{
    DateFolder, Journal, StoreError, WorkspaceNameError, check_workspace_name, create_workspace,
    dates, delete_workspace, last_written, newest_batch_rule, open_date, rename_workspace,
    workspaces,
};
use testkit::TempDir;

use crate::common::{APP, batch};

#[test]
fn a_workspace_name_follows_the_file_name_rule_and_is_unique() {
    let existing = vec!["Toko Contoh".to_string()];
    let cases = [
        ("Toko Lain", Ok(())),
        ("Toko_2-B", Ok(())),
        ("", Err(WorkspaceNameError::NotAName)),
        (" Toko", Err(WorkspaceNameError::NotAName)),
        ("Toko & Co", Err(WorkspaceNameError::NotAName)),
        ("Toko  Dua", Err(WorkspaceNameError::NotAName)),
        (&"x".repeat(41), Err(WorkspaceNameError::NotAName)),
        ("Labels", Err(WorkspaceNameError::Reserved)),
        ("categories", Err(WorkspaceNameError::Reserved)),
        ("TOKO CONTOH", Err(WorkspaceNameError::Taken)),
    ];
    for (name, want) in cases {
        let got = check_workspace_name(name, &existing);
        assert_eq!(got, want, "{name:?}");
        if let Err(error) = got {
            assert!(!error.to_string().is_empty());
        }
    }
    assert!(check_workspace_name(&"x".repeat(40), &existing).is_ok());
}

#[test]
fn the_list_holds_the_workspace_folders_and_the_last_written_one_opens() {
    let data = TempDir::new("workspaces");
    assert!(workspaces(data.path()).unwrap().is_empty());
    assert_eq!(last_written(data.path()).unwrap(), None);
    create_workspace(data.path(), "Toko Contoh").unwrap();
    create_workspace(data.path(), "toko lain").unwrap();
    fs::create_dir_all(data.path().join("labels")).unwrap();
    fs::create_dir_all(data.path().join("not & a name")).unwrap();
    assert_eq!(
        workspaces(data.path()).unwrap(),
        ["Toko Contoh", "toko lain"]
    );
    match create_workspace(data.path(), "TOKO LAIN") {
        Err(StoreError::BadName { reason, .. }) => assert!(reason.contains("another workspace")),
        other => panic!("expected the name taken, got {:?}", other.err()),
    }

    for (workspace, day) in [("toko lain", 7), ("Toko Contoh", 8)] {
        let folder = DateFolder::new(data.path(), workspace, date(2026, 10, day));
        let mut journal = Journal::open(&folder).unwrap();
        journal
            .commit(
                vec![RecordBody::BatchSaved(batch(1, false, Vec::new()))],
                APP,
            )
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    fs::create_dir_all(data.path().join("Toko Contoh").join("not a date")).unwrap();
    assert_eq!(
        dates(data.path(), "Toko Contoh").unwrap(),
        [date(2026, 10, 8)]
    );
    assert_eq!(
        last_written(data.path()).unwrap().as_deref(),
        Some("Toko Contoh")
    );
}

#[test]
fn the_newest_batchs_rule_is_found_on_whichever_date_it_was_saved() {
    let data = TempDir::new("newest-rule");
    create_workspace(data.path(), "Toko Contoh").unwrap();
    assert_eq!(newest_batch_rule(data.path(), "Toko Contoh").unwrap(), None);
    let save = |workspace: &str, day: i8, number: u8, rule: &str| {
        let mut saved = batch(number, false, Vec::new());
        saved.sort_rule.name = rule.into();
        Journal::open(&DateFolder::new(
            data.path(),
            workspace,
            date(2026, 10, day),
        ))
        .unwrap()
        .commit(vec![RecordBody::BatchSaved(saved)], APP)
        .unwrap();
    };
    save("Toko Contoh", 7, 1, "default");
    save("Toko Contoh", 7, 2, "harian");
    fs::create_dir_all(data.path().join("Toko Contoh").join("2026-10-08")).unwrap();
    let damaged = data.path().join("Toko Contoh").join("2026-10-09");
    fs::create_dir_all(&damaged).unwrap();
    fs::write(damaged.join("journal.jsonl"), b"not a record\n").unwrap();
    assert_eq!(
        newest_batch_rule(data.path(), "Toko Contoh")
            .unwrap()
            .as_deref(),
        Some("harian"),
        "a later date with no batch, or a damaged journal, is passed over"
    );

    save("Toko Lain", 10, 1, "promo");
    save("Toko Contoh", 6, 1, "lama");
    assert_eq!(
        newest_batch_rule(data.path(), "Toko Contoh")
            .unwrap()
            .as_deref(),
        Some("harian"),
        "another workspace and an older date do not count"
    );
}

#[test]
fn a_renamed_workspace_keeps_its_dates_and_their_batches() {
    let data = TempDir::new("rename");
    create_workspace(data.path(), "Toko Contoh").unwrap();
    create_workspace(data.path(), "Toko Lain").unwrap();
    let folder = DateFolder::new(data.path(), "Toko Contoh", date(2026, 10, 7));
    Journal::open(&folder)
        .unwrap()
        .commit(
            vec![RecordBody::BatchSaved(batch(1, false, Vec::new()))],
            APP,
        )
        .unwrap();

    for (new_name, reason) in [
        ("toko lain", "another workspace"),
        ("Toko & Co", "1 to 40"),
        ("labels", "Packing v0.1"),
    ] {
        match rename_workspace(data.path(), "Toko Contoh", new_name) {
            Err(StoreError::BadName { reason: why, .. }) => assert!(why.contains(reason), "{why}"),
            other => panic!("{new_name}: expected a refused name, got {other:?}"),
        }
    }
    rename_workspace(data.path(), "Toko Contoh", "toko contoh").unwrap();
    rename_workspace(data.path(), "toko contoh", "Toko Baru").unwrap();
    assert_eq!(workspaces(data.path()).unwrap(), ["Toko Baru", "Toko Lain"]);
    assert_eq!(
        dates(data.path(), "Toko Baru").unwrap(),
        [date(2026, 10, 7)]
    );

    let opened = open_date(&DateFolder::new(
        data.path(),
        "Toko Baru",
        date(2026, 10, 7),
    ))
    .unwrap();
    assert_eq!(
        opened.journal.damage(),
        None,
        "its journal names the old name"
    );
    assert_eq!(opened.journal.fold().live.len(), 1);
}

#[test]
fn a_deleted_workspace_leaves_the_list() {
    let data = TempDir::new("delete");
    create_workspace(data.path(), "Toko Contoh").unwrap();
    create_workspace(data.path(), "Toko Lain").unwrap();
    assert!(delete_workspace(data.path(), "Toko Tiada").is_err());
    delete_workspace(data.path(), "Toko Lain").unwrap();
    assert_eq!(workspaces(data.path()).unwrap(), ["Toko Contoh"]);
    assert!(!data.path().join("Toko Lain").exists());
}
