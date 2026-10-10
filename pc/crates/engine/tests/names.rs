//! `names.rs`: saved-PDF file names, the user's whole name, the Windows-forbidden characters,
//! duplicate names and the folder / file names (07 › *Day folder*; 03 › *Numbering and file
//! names*; mirrors the file-name cases of script/tests/test_picks.py).

use packing_engine::names::{
    NAME_LIMIT, NameError, batch_folder, check_unique_names, day_folder, normalize_saved_pdf_name,
    packing_list_file_name, sanitize_file_name, suggested_saved_pdf_name, temp_batch_folder,
};

#[test]
fn suggested_name_matches_the_spec_example() {
    assert_eq!(
        suggested_saved_pdf_name(3, "Z", "Lainnya", 2),
        "3 Z Lainnya ×2.pdf"
    );
    assert_eq!(
        suggested_saved_pdf_name(1, "A", "Sepatu", 172),
        "1 A Sepatu ×172.pdf"
    );
}

#[test]
fn the_rest_code_becomes_a_dash() {
    assert_eq!(
        suggested_saved_pdf_name(5, "?", "Uncategorised", 2),
        "5 - Uncategorised ×2.pdf"
    );
}

#[test]
fn illegal_characters_become_a_dash() {
    let name = "A\\B/C:D*E?F\"G<H>I|J";
    assert_eq!(
        suggested_saved_pdf_name(1, "A", name, 1),
        "1 A A-B-C-D-E-F-G-H-I-J ×1.pdf"
    );
}

#[test]
fn the_name_part_is_cut_at_100_characters() {
    let long_name = "Produk ".repeat(40);
    let file_name = suggested_saved_pdf_name(1, "A", &long_name, 3);
    assert!(file_name.starts_with("1 A "));
    assert!(file_name.ends_with(" ×3.pdf"));
    let name_part = &file_name["1 A ".len()..file_name.len() - " ×3.pdf".len()];
    assert_eq!(name_part.chars().count(), NAME_LIMIT);
    assert!(name_part.ends_with('…'));
}

#[test]
fn a_user_name_gets_its_pdf_extension() {
    assert_eq!(
        normalize_saved_pdf_name("1 A Sepatu sendiri").unwrap(),
        "1 A Sepatu sendiri.pdf"
    );
    assert_eq!(
        normalize_saved_pdf_name("1 A Sepatu sendiri.pdf").unwrap(),
        "1 A Sepatu sendiri.pdf"
    );
    assert_eq!(normalize_saved_pdf_name("  x.PDF  ").unwrap(), "x.pdf");
}

#[test]
fn a_user_name_is_sanitized_and_an_empty_one_refused() {
    assert_eq!(normalize_saved_pdf_name("a/b:c.pdf").unwrap(), "a-b-c.pdf");
    assert_eq!(normalize_saved_pdf_name(".pdf"), Err(NameError::Empty));
    assert_eq!(normalize_saved_pdf_name("   "), Err(NameError::Empty));
}

#[test]
fn duplicate_names_are_refused() {
    let names = vec!["1 A.pdf".to_string(), "2 B.pdf".to_string()];
    assert_eq!(check_unique_names(&names), Ok(()));

    let clash = vec!["1 A.pdf".to_string(), "1 a.PDF".to_string()];
    assert_eq!(
        check_unique_names(&clash),
        Err(NameError::Duplicate {
            name: "1 a.PDF".to_string(),
            first: 0,
            second: 1,
        })
    );
}

#[test]
fn sanitize_only_touches_the_forbidden_characters() {
    assert_eq!(
        sanitize_file_name("3 Z Lainnya ×393.pdf"),
        "3 Z Lainnya ×393.pdf"
    );
    assert_eq!(sanitize_file_name("a<b>c"), "a-b-c");
}

#[test]
fn folder_and_file_names() {
    assert_eq!(batch_folder(1), "batch 1");
    assert_eq!(temp_batch_folder(2), "batch 2.partial");
    let date: jiff::civil::Date = "2026-10-07".parse().expect("date");
    assert_eq!(day_folder(date), "2026-10-07");
}

#[test]
fn packing_list_file_names() {
    // Scope whole, one layout: the plain name.
    assert_eq!(packing_list_file_name(None, None), "packing-list.pdf");
    // Scope whole, several layouts: the layout is added.
    assert_eq!(
        packing_list_file_name(None, Some("full")),
        "packing-list-full.pdf"
    );
    // Scope per-pdf, several layouts: number and layout.
    assert_eq!(
        packing_list_file_name(Some(3), Some("pick")),
        "packing-list-3-pick.pdf"
    );
    // Scope per-pdf, one layout: the number only.
    assert_eq!(packing_list_file_name(Some(1), None), "packing-list-1.pdf");
}
