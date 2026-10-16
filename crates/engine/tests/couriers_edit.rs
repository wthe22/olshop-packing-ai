//! couriers_edit.rs: the hash the owner taught is written into `couriers.toml`, comments and
//! order kept (D-LABEL-4, Q-LABEL-7 default).

use std::str::FromStr;

use packing_engine::couriers_edit::CouriersDocument;
use packing_engine::label::load_couriers;

const FILE: &str = "\
# Courier deduction data.
# A hash is 16 hex characters.

not_courier = []          # answered Not a courier

[[courier]]
name = \"J&T Express\"
logo = [\"39b9b7b9f61d0e7f\"]   # 975x377 px, the same asset in every sample
tracking = \"JY\\\\d{10}\"

[[courier]]
name = \"SiCepat\"
logo = []                     # its asset is invisible in those bytes
tracking = \"00\\\\d{10}\"
";

#[test]
fn the_taught_hash_is_appended_and_the_comments_are_kept() {
    let mut document = CouriersDocument::from_str(FILE).expect("parse");
    document
        .add_courier_logo("SiCepat", "cf33ed60daa291b1")
        .expect("remember a logo");
    document
        .add_not_courier("0123456789abcdef")
        .expect("remember Not a courier");
    let out = document.to_string();

    // Comments and the untouched entries are byte-for-byte.
    assert!(out.contains("# Courier deduction data."), "{out}");
    assert!(
        out.contains("# 975x377 px, the same asset in every sample"),
        "{out}"
    );
    assert!(
        out.contains("# its asset is invisible in those bytes"),
        "{out}"
    );
    assert!(out.contains("# answered Not a courier"), "{out}");
    assert!(out.contains("logo = [\"39b9b7b9f61d0e7f\"]"), "{out}");

    // `not_courier` stays a key of the root table, above the first `[[courier]]`, or it would
    // belong to the last courier instead.
    let head = out.split("[[courier]]").next().expect("a head");
    assert!(
        head.contains("not_courier = [\"0123456789abcdef\"]"),
        "not_courier is a root key:\n{out}"
    );

    // The written file loads, and both hashes are now known.
    let table = load_couriers(&out).expect("the written file must load");
    assert!(table.knows_byte_hash("cf33ed60daa291b1"));
    assert!(table.knows_byte_hash("0123456789abcdef"));
    let sicepat = table
        .couriers
        .iter()
        .find(|courier| courier.name == "SiCepat")
        .expect("SiCepat");
    assert_eq!(sicepat.logo, vec!["cf33ed60daa291b1".to_string()]);
    assert_eq!(table.not_courier, vec!["0123456789abcdef".to_string()]);
}

#[test]
fn remembering_the_same_hash_twice_changes_nothing() {
    let mut document = CouriersDocument::from_str(FILE).expect("parse");
    document
        .add_courier_logo("J&T Express", "39b9b7b9f61d0e7f")
        .expect("already there");
    document.add_not_courier("0123456789abcdef").expect("new");
    document
        .add_not_courier("0123456789abcdef")
        .expect("already there");
    let out = document.to_string();
    assert_eq!(out.matches("39b9b7b9f61d0e7f").count(), 1, "{out}");
    assert_eq!(out.matches("0123456789abcdef").count(), 1, "{out}");
}

#[test]
fn a_missing_list_is_created_and_an_unknown_courier_is_an_error() {
    // A courier without a `logo` key gets one.
    let mut document =
        CouriersDocument::from_str("[[courier]]\nname = \"IDX\"\ntracking = \"TKP\\\\d{10}\"\n")
            .expect("parse");
    document
        .add_courier_logo("IDX", "9131c1f94ad363d4")
        .expect("create the list");
    let table = load_couriers(&document.to_string()).expect("load");
    assert_eq!(table.couriers[0].logo, vec!["9131c1f94ad363d4".to_string()]);

    let error = document
        .add_courier_logo("Nope", "9131c1f94ad363d4")
        .expect_err("no such courier");
    assert_eq!(error.message, "couriers.toml: no courier named \"Nope\"");

    // A file without a `[[courier]]` table cannot be edited either.
    let error = CouriersDocument::from_str("not_courier = []\n").expect_err("no couriers");
    assert_eq!(error.message, "couriers.toml: no couriers");
}
