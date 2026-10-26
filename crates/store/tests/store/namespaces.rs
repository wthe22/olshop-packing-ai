//! Namespaces: which names are names, the list, the dates, the one written last.

use std::fs;

use jiff::civil::date;
use packing_engine::record::RecordBody;
use packing_store::{
    DateFolder, Journal, NamespaceNameError, StoreError, check_namespace_name, create_namespace,
    dates, last_written, namespaces,
};
use testkit::TempDir;

use crate::common::{APP, batch};

#[test]
fn a_namespace_name_follows_the_file_name_rule_and_is_unique() {
    let existing = vec!["Toko Contoh".to_string()];
    let cases = [
        ("Toko Lain", Ok(())),
        ("Toko_2-B", Ok(())),
        ("", Err(NamespaceNameError::NotAName)),
        (" Toko", Err(NamespaceNameError::NotAName)),
        ("Toko & Co", Err(NamespaceNameError::NotAName)),
        ("Toko  Dua", Err(NamespaceNameError::NotAName)),
        (&"x".repeat(41), Err(NamespaceNameError::NotAName)),
        ("Labels", Err(NamespaceNameError::Reserved)),
        ("categories", Err(NamespaceNameError::Reserved)),
        ("TOKO CONTOH", Err(NamespaceNameError::Taken)),
    ];
    for (name, want) in cases {
        let got = check_namespace_name(name, &existing);
        assert_eq!(got, want, "{name:?}");
        if let Err(error) = got {
            assert!(!error.to_string().is_empty());
        }
    }
    assert!(check_namespace_name(&"x".repeat(40), &existing).is_ok());
}

#[test]
fn the_list_holds_the_namespace_folders_and_the_last_written_one_opens() {
    let data = TempDir::new("namespaces");
    assert!(namespaces(data.path()).unwrap().is_empty());
    assert_eq!(last_written(data.path()).unwrap(), None);
    create_namespace(data.path(), "Toko Contoh").unwrap();
    create_namespace(data.path(), "toko lain").unwrap();
    fs::create_dir_all(data.path().join("labels")).unwrap();
    fs::create_dir_all(data.path().join("not & a name")).unwrap();
    assert_eq!(
        namespaces(data.path()).unwrap(),
        ["Toko Contoh", "toko lain"]
    );
    match create_namespace(data.path(), "TOKO LAIN") {
        Err(StoreError::BadName { reason, .. }) => assert!(reason.contains("another namespace")),
        other => panic!("expected the name taken, got {:?}", other.err()),
    }

    for (namespace, day) in [("toko lain", 7), ("Toko Contoh", 8)] {
        let folder = DateFolder::new(data.path(), namespace, date(2026, 10, day));
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
