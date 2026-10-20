//! label.rs: the courier data of the data folder — loading `couriers.toml`, its errors with the
//! line, and the bundled file as the default.

use std::path::PathBuf;

use packing_engine::label::{
    COURIERS_TOML, CourierTable, courier_data, load_couriers, load_couriers_file,
};

fn temp_path(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "packing-engine-couriers-{}-{label}.toml",
        std::process::id()
    ))
}

fn names(table: &CourierTable) -> Vec<String> {
    table.names()
}

#[test]
fn the_bundled_file_loads_with_the_measured_couriers() {
    let table = load_couriers(COURIERS_TOML).expect("the bundled file must load");
    assert_eq!(
        names(&table),
        vec!["J&T Express", "SiCepat", "J&T Cargo", "IDX", "SPX"]
    );
    // The two-clue data of the measured couriers (courier.md item 1) comes through.
    let cargo = table
        .couriers
        .iter()
        .find(|courier| courier.name == "J&T Cargo")
        .expect("J&T Cargo");
    assert!(cargo.logo.is_empty());
    assert_eq!(cargo.shape, vec!["753ccbd14749befa".to_string()]);
    assert_eq!(cargo.tracking.as_deref(), Some("57\\d{10}"));
    assert!(table.not_courier.is_empty());
    // A byte hash of the file is known; an unknown one is not.
    assert!(table.knows_byte_hash("39b9b7b9f61d0e7f"));
    assert!(!table.knows_byte_hash("0123456789abcdef"));
}

#[test]
fn a_missing_file_is_the_bundled_default() {
    let path = temp_path("missing");
    let _ = std::fs::remove_file(&path);
    let table = load_couriers_file(&path).expect("a missing file falls back to the bundled list");
    assert_eq!(names(&table), names(courier_data()));
}

#[test]
fn a_broken_file_reports_its_line() {
    // A TOML syntax error: the message names the file and the line the parser stopped at.
    let error = load_couriers("not_courier = [\"aa11\"\n[[courier]]\nname = \"X\"\n")
        .expect_err("unterminated array");
    assert!(
        error.message.starts_with("couriers.toml: "),
        "{}",
        error.message
    );
    assert!(error.message.contains("line"), "{}", error.message);

    // A field error names the courier and what is wrong.
    let error = load_couriers("[[courier]]\nlogo = [\"aa11\"]\ntracking = \"JY\\\\d{10}\"\n")
        .expect_err("missing name");
    assert_eq!(error.message, "couriers.toml: courier #1: missing \"name\"");

    let error =
        load_couriers("[[courier]]\nname = \"X\"\nlogo = [\"ZZ11\"]\ntracking = \"JY\\\\d{10}\"\n")
            .expect_err("a hash is 16 lowercase hex characters");
    assert_eq!(
        error.message,
        "couriers.toml: courier \"X\": \"logo\" entry \"ZZ11\" is not 16 hex characters (an \
         XXH3-64 hash)"
    );

    let error = load_couriers("[[courier]]\nname = \"X\"\ntracking = \"JY(\\\\d{10}\"\n")
        .expect_err("a bad pattern");
    assert!(
        error
            .message
            .starts_with("couriers.toml: courier \"X\": \"tracking\" is not a valid pattern:"),
        "{}",
        error.message
    );

    let error = load_couriers("[[courier]]\nname = \"X\"\nnope = 1\n").expect_err("unknown key");
    assert_eq!(
        error.message,
        "couriers.toml: courier \"X\": unknown key \"nope\""
    );

    let error = load_couriers("courier = []\n").expect_err("no couriers");
    assert_eq!(error.message, "couriers.toml: no couriers");
}

#[test]
fn a_file_that_reads_as_the_bundled_list_still_loads() {
    // The round trip the app does: read the file, then load it.
    let path = temp_path("roundtrip");
    std::fs::write(&path, COURIERS_TOML).expect("write");
    let table = load_couriers_file(&path).expect("the bundled text must load from a file");
    assert_eq!(names(&table), names(courier_data()));
    let _ = std::fs::remove_file(&path);
}
