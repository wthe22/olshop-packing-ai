//! The check before saving's arithmetic on made-up plan and read-back descriptions
//! (07 › *Check before saving*): a doubled page, a dropped page, an order in two picks and in
//! none, a wrong label and a swapped page order — each reported, each in its own list.
//!
//! Every defect test also asks the same description with the check switched off: the result is
//! empty, so removing the check makes these tests fail, exactly as the 2.15 display tests fail
//! when their fix is reverted.

use packing_engine::ids::{OrderId, PdfNumber};
use packing_engine::verify::{
    Check, Expected, ExpectedList, ExpectedPdf, Finding, Written, WrittenList, WrittenPdf, compare,
};

/// Two picks, three orders (one of them on two pages) and one packing list, as made-up data.
fn plan() -> Expected {
    Expected {
        pdfs: vec![
            ExpectedPdf {
                number: PdfNumber::new(1),
                file: "1 A Sepatu ×2.pdf".to_string(),
                pages: order_ids(&[
                    "580000000000000001",
                    "580000000000000001",
                    "580000000000000002",
                ]),
            },
            ExpectedPdf {
                number: PdfNumber::new(2),
                file: "2 B Spion ×1.pdf".to_string(),
                pages: order_ids(&["580000000000000003"]),
            },
        ],
        lists: vec![ExpectedList {
            file: "packing-list.pdf".to_string(),
            orders: 3,
            units: 4,
            runs: 2,
        }],
    }
}

/// The plan's read-back, faithful unless a test changes one pdf.
fn written(pdfs: Vec<WrittenPdf>) -> Written {
    Written {
        pdfs,
        lists: vec![WrittenList {
            file: "packing-list.pdf".to_string(),
            orders: Some(3),
            units: Some(4),
            runs: Some(2),
        }],
    }
}

fn pages(ids: &[&str]) -> Vec<Option<OrderId>> {
    ids.iter().map(|id| Some(OrderId::new(*id))).collect()
}

fn order_ids(ids: &[&str]) -> Vec<OrderId> {
    ids.iter().map(|id| OrderId::new(*id)).collect()
}

fn pdf1(ids: &[&str]) -> WrittenPdf {
    WrittenPdf {
        file: "1 A Sepatu ×2.pdf".to_string(),
        pages: pages(ids),
    }
}

fn pdf2(ids: &[&str]) -> WrittenPdf {
    WrittenPdf {
        file: "2 B Spion ×1.pdf".to_string(),
        pages: pages(ids),
    }
}

/// The comparison, or — with `check_on` false — the result of a save whose check is switched off
/// (nothing read back and compared).
fn check(expected: &Expected, written: &Written, check_on: bool) -> Check {
    if check_on {
        compare(expected, written)
    } else {
        Check::default()
    }
}

fn has(findings: &[Finding], wanted: &Finding, file: &str) -> bool {
    if findings.contains(wanted) {
        return true;
    }
    panic!("{file}: {wanted:?} not found in {findings:#?}");
}

#[test]
fn a_faithful_read_back_reports_nothing() {
    let written = written(vec![
        pdf1(&[
            "580000000000000001",
            "580000000000000001",
            "580000000000000002",
        ]),
        pdf2(&["580000000000000003"]),
    ]);
    let result = compare(&plan(), &written);
    assert!(result.is_empty(), "{result:#?}");
}

#[test]
fn a_doubled_page_is_an_error_in_its_own_list() {
    // Order 1's page is written twice.
    let written = written(vec![
        pdf1(&[
            "580000000000000001",
            "580000000000000001",
            "580000000000000001",
            "580000000000000002",
        ]),
        pdf2(&["580000000000000003"]),
    ]);
    let result = check(&plan(), &written, true);
    assert!(
        result.warnings.is_empty(),
        "a doubled page is an error only"
    );
    assert!(has(
        &result.errors,
        &Finding::DoubledPageInOnePdf {
            pdf: PdfNumber::new(1),
            file: "1 A Sepatu ×2.pdf".to_string(),
            order_id: OrderId::new("580000000000000001"),
            expected: 2,
            found: 3,
        },
        "doubled page",
    ));
    // Switched off: the same save reports nothing.
    assert!(check(&plan(), &written, false).is_empty());
}

#[test]
fn a_dropped_page_is_an_error_in_its_own_list() {
    // Order 2's only page is gone.
    let written = written(vec![
        pdf1(&["580000000000000001", "580000000000000001"]),
        pdf2(&["580000000000000003"]),
    ]);
    let result = check(&plan(), &written, true);
    assert!(result.warnings.is_empty());
    assert!(has(
        &result.errors,
        &Finding::MissingPage {
            pdf: PdfNumber::new(1),
            file: "1 A Sepatu ×2.pdf".to_string(),
            order_id: OrderId::new("580000000000000002"),
            expected: 1,
            found: 0,
        },
        "dropped page",
    ));
    assert!(check(&plan(), &written, false).is_empty());
}

#[test]
fn an_order_in_two_pdfs_is_an_error_in_its_own_list() {
    // Order 1 is also written into pick 2.
    let written = written(vec![
        pdf1(&[
            "580000000000000001",
            "580000000000000001",
            "580000000000000002",
        ]),
        pdf2(&["580000000000000001", "580000000000000003"]),
    ]);
    let result = check(&plan(), &written, true);
    assert!(result.warnings.is_empty());
    assert!(has(
        &result.errors,
        &Finding::OrderInTwoPdfs {
            order_id: OrderId::new("580000000000000001"),
            pdfs: vec![PdfNumber::new(1), PdfNumber::new(2)],
        },
        "order in two picks",
    ));
    assert!(check(&plan(), &written, false).is_empty());
}

#[test]
fn an_order_in_no_pdf_is_an_error_in_its_own_list() {
    // Orders 2 and 3 are nowhere.
    let written = written(vec![
        pdf1(&["580000000000000001", "580000000000000001"]),
        pdf2(&[]),
    ]);
    let result = check(&plan(), &written, true);
    assert!(result.warnings.is_empty());
    assert!(has(
        &result.errors,
        &Finding::OrderInNoPdf {
            order_id: OrderId::new("580000000000000002"),
        },
        "order in no pick",
    ));
    assert!(has(
        &result.errors,
        &Finding::OrderInNoPdf {
            order_id: OrderId::new("580000000000000003"),
        },
        "order in no pick",
    ));
    assert!(check(&plan(), &written, false).is_empty());
}

#[test]
fn a_wrong_label_on_a_page_is_an_error() {
    // Page 1 of pick 1 holds order 3's label.
    let written = written(vec![
        pdf1(&[
            "580000000000000003",
            "580000000000000001",
            "580000000000000002",
        ]),
        pdf2(&["580000000000000003"]),
    ]);
    let result = check(&plan(), &written, true);
    assert!(has(
        &result.errors,
        &Finding::WrongOrderLabelOnPage {
            pdf: PdfNumber::new(1),
            file: "1 A Sepatu ×2.pdf".to_string(),
            page: 1,
            order_id: OrderId::new("580000000000000003"),
        },
        "wrong label",
    ));
    assert!(check(&plan(), &written, false).is_empty());
}

#[test]
fn a_page_count_that_is_not_the_plans_is_an_error() {
    // An extra page no order owns is written into pick 1.
    let mut first = pdf1(&[
        "580000000000000001",
        "580000000000000001",
        "580000000000000002",
    ]);
    first.pages.push(None);
    let written = written(vec![first, pdf2(&["580000000000000003"])]);
    let result = check(&plan(), &written, true);
    assert!(has(
        &result.errors,
        &Finding::PageCountDiffersFromPlan {
            pdf: PdfNumber::new(1),
            file: "1 A Sepatu ×2.pdf".to_string(),
            expected: 3,
            found: 4,
        },
        "page count",
    ));
    assert!(check(&plan(), &written, false).is_empty());
}

#[test]
fn a_swapped_page_order_is_a_warning_only_in_its_own_list() {
    // Both pages of order 1 are there, but order 2's comes first.
    let written = written(vec![
        pdf1(&[
            "580000000000000002",
            "580000000000000001",
            "580000000000000001",
        ]),
        pdf2(&["580000000000000003"]),
    ]);
    let result = check(&plan(), &written, true);
    assert!(result.errors.is_empty(), "a swapped order is no error");
    assert!(has(
        &result.warnings,
        &Finding::PageOrderDiffers {
            pdf: PdfNumber::new(1),
            file: "1 A Sepatu ×2.pdf".to_string(),
        },
        "swapped page order",
    ));
    assert!(check(&plan(), &written, false).is_empty());
}

#[test]
fn packing_list_counts_that_are_not_the_plans_are_an_error() {
    let mut written = written(vec![
        pdf1(&[
            "580000000000000001",
            "580000000000000001",
            "580000000000000002",
        ]),
        pdf2(&["580000000000000003"]),
    ]);
    written.lists[0].orders = Some(2);
    let result = check(&plan(), &written, true);
    assert!(has(
        &result.errors,
        &Finding::PackingListCountsDiffer {
            file: "packing-list.pdf".to_string(),
            expected_orders: 3,
            expected_units: 4,
            expected_runs: 2,
            found_orders: Some(2),
            found_units: Some(4),
            found_runs: Some(2),
        },
        "packing-list count",
    ));
    assert!(check(&plan(), &written, false).is_empty());
}
