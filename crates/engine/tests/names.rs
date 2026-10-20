//! `names.rs`: saved-PDF file names, the user's whole name, the Windows-forbidden characters,
//! duplicate names and the folder / file names (mirrors the file-name cases of
//! tools/analysis/tests/test_picks.py).

use packing_engine::ids::{BatchNumber, PdfNumber};
use packing_engine::names::{
    NAME_LIMIT, NameError, batch_folder, check_unique_names, day_folder, normalize_saved_pdf_name,
    packing_list_file_name, sanitize_file_name, suggested_saved_pdf_name, temp_batch_folder,
};

#[test]
fn a_suggested_saved_pdf_name_follows_the_01_format() {
    let cases = [
        ((3, "Z", "Lainnya", 2), "3 Z Lainnya ×2.pdf"),
        ((1, "A", "Sepatu", 172), "1 A Sepatu ×172.pdf"),
        // The rest code `?` becomes a dash.
        ((5, "?", "Uncategorised", 2), "5 - Uncategorised ×2.pdf"),
        // Every Windows-forbidden character becomes a dash.
        (
            (1, "A", "A\\B/C:D*E?F\"G<H>I|J", 1),
            "1 A A-B-C-D-E-F-G-H-I-J ×1.pdf",
        ),
    ];
    for ((number, code, name, count), expected) in cases {
        assert_eq!(
            suggested_saved_pdf_name(PdfNumber::new(number), code, name, count),
            expected
        );
    }

    // A long name part is cut at the limit, ending with an ellipsis.
    let long_name = "Produk ".repeat(40);
    let file_name = suggested_saved_pdf_name(PdfNumber::new(1), "A", &long_name, 3);
    assert!(file_name.starts_with("1 A "));
    assert!(file_name.ends_with(" ×3.pdf"));
    let name_part = &file_name["1 A ".len()..file_name.len() - " ×3.pdf".len()];
    assert_eq!(name_part.chars().count(), NAME_LIMIT);
    assert!(name_part.ends_with('…'));
}

#[test]
fn a_name_is_made_safe_for_windows_only_where_it_must_be() {
    let typed: Vec<(&str, Result<&str, NameError>)> = vec![
        ("1 A Sepatu sendiri", Ok("1 A Sepatu sendiri.pdf")),
        ("1 A Sepatu sendiri.pdf", Ok("1 A Sepatu sendiri.pdf")),
        ("  x.PDF  ", Ok("x.pdf")),
        ("a/b:c.pdf", Ok("a-b-c.pdf")),
        (".pdf", Err(NameError::Empty)),
        ("   ", Err(NameError::Empty)),
    ];
    for (name, expected) in typed {
        match expected {
            Ok(want) => assert_eq!(normalize_saved_pdf_name(name).unwrap(), want, "{name}"),
            Err(want) => assert_eq!(normalize_saved_pdf_name(name).unwrap_err(), want, "{name}"),
        }
    }

    // Only the forbidden characters are touched; the rest of the name stays as it is.
    let built = [
        ("3 Z Lainnya ×393.pdf", "3 Z Lainnya ×393.pdf"),
        ("a<b>c", "a-b-c"),
    ];
    for (name, expected) in built {
        assert_eq!(sanitize_file_name(name), expected);
    }
}

#[test]
fn two_file_names_that_differ_only_in_case_are_a_duplicate() {
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
fn the_folder_and_packing_list_file_names_are_fixed() {
    assert_eq!(batch_folder(BatchNumber::new(1)), "batch 1");
    assert_eq!(temp_batch_folder(BatchNumber::new(2)), "batch 2.partial");
    let date: jiff::civil::Date = "2026-10-07".parse().expect("date");
    assert_eq!(day_folder(date), "2026-10-07");

    // Scope whole with one layout is the plain name; a second layout is added; a per-PDF scope
    // puts the number, and the layout only where one layout is not enough.
    let names = [
        ((None, None), "packing-list.pdf"),
        ((None, Some("full")), "packing-list-full.pdf"),
        (
            (Some(PdfNumber::new(3)), Some("pick")),
            "packing-list-3-pick.pdf",
        ),
        ((Some(PdfNumber::new(1)), None), "packing-list-1.pdf"),
    ];
    for ((pdf, layout), expected) in names {
        assert_eq!(packing_list_file_name(pdf, layout), expected);
    }
}
