//! The arithmetic of the check before saving (07 › *Check before saving*): compare the plan with
//! a read-back description of the written files.
//!
//! Pure — no PDF code — so every rule is tested on hand-made descriptions. A save with an empty
//! [`Check`] renames straight away; any finding, error or warning, stops it until *Save anyway*
//! (Q-APP-1).

use std::collections::BTreeMap;
use std::fmt;

use crate::orders::ReadOrder;
use crate::plan::PlanResult;

/// One saved PDF as the plan wants it: `pages` holds the Order ID of every page, in run order
/// (an order of two pages repeats its ID once per page).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExpectedPdf {
    pub number: i64,
    pub file: String,
    pub pages: Vec<String>,
}

/// One packing list as the plan wants it: the counts it must print.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExpectedList {
    pub file: String,
    pub orders: usize,
    pub units: i64,
    pub runs: usize,
}

/// The plan side of the check.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Expected {
    pub pdfs: Vec<ExpectedPdf>,
    pub lists: Vec<ExpectedList>,
}

/// One saved PDF as it was read back: the Order ID found on every page (`None` for a page that
/// carries none and could not continue a previous order).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WrittenPdf {
    pub file: String,
    pub pages: Vec<Option<String>>,
}

/// One packing list as it was read back; a count is `None` when its line could not be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WrittenList {
    pub file: String,
    pub orders: Option<usize>,
    pub units: Option<i64>,
    pub runs: Option<usize>,
}

/// The read-back side of the check.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Written {
    pub pdfs: Vec<WrittenPdf>,
    pub lists: Vec<WrittenList>,
}

/// A finding of the check; the severities are [`Check::errors`] and [`Check::warnings`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Finding {
    /// A page of an order is missing from a saved PDF.
    MissingPage {
        pdf: i64,
        file: String,
        order_id: String,
        expected: usize,
        found: usize,
    },
    /// A page is twice in one saved PDF.
    DoubledPageInOnePdf {
        pdf: i64,
        file: String,
        order_id: String,
        expected: usize,
        found: usize,
    },
    /// An order is in two saved PDFs.
    OrderInTwoPdfs { order_id: String, pdfs: Vec<i64> },
    /// An order is in no saved PDF.
    OrderInNoPdf { order_id: String },
    /// A page holds another pick's label.
    WrongOrderLabelOnPage {
        pdf: i64,
        file: String,
        page: usize,
        order_id: String,
    },
    /// A written page count differs from the plan.
    PageCountDiffersFromPlan {
        pdf: i64,
        file: String,
        expected: usize,
        found: usize,
    },
    /// A packing list's printed counts differ from the plan.
    PackingListCountsDiffer {
        file: String,
        expected_orders: usize,
        expected_units: i64,
        expected_runs: usize,
        found_orders: Option<usize>,
        found_units: Option<i64>,
        found_runs: Option<usize>,
    },
    /// Every page is there exactly once, but not in the run order (a warning).
    PageOrderDiffers { pdf: i64, file: String },
}

impl Finding {
    /// Whether the finding is a warning; every other finding is an error
    /// (07 › *Check before saving*).
    pub fn is_warning(&self) -> bool {
        matches!(self, Finding::PageOrderDiffers { .. })
    }
}

fn maybe<T: fmt::Display>(value: Option<T>) -> String {
    match value {
        Some(value) => value.to_string(),
        None => "?".to_string(),
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Finding::MissingPage {
                pdf,
                file,
                order_id,
                expected,
                found,
            } => write!(
                f,
                "PDF {pdf} ({file}): the page of order {order_id} is missing \
                 ({found} of {expected} written)."
            ),
            Finding::DoubledPageInOnePdf {
                pdf,
                file,
                order_id,
                expected,
                found,
            } => write!(
                f,
                "PDF {pdf} ({file}): the page of order {order_id} is written {found} times, \
                 the plan has {expected}."
            ),
            Finding::OrderInTwoPdfs { order_id, pdfs } => {
                let list: Vec<String> = pdfs.iter().map(i64::to_string).collect();
                write!(
                    f,
                    "order {order_id} is in {} saved PDFs ({}); it must be in one.",
                    pdfs.len(),
                    list.join(", ")
                )
            }
            Finding::OrderInNoPdf { order_id } => {
                write!(f, "order {order_id} is in no saved PDF.")
            }
            Finding::WrongOrderLabelOnPage {
                pdf,
                file,
                page,
                order_id,
            } => write!(
                f,
                "PDF {pdf} ({file}): page {page} holds order {order_id}'s label, \
                 which belongs to another pick."
            ),
            Finding::PageCountDiffersFromPlan {
                pdf,
                file,
                expected,
                found,
            } => write!(
                f,
                "PDF {pdf} ({file}) has {found} pages; the plan has {expected}."
            ),
            Finding::PackingListCountsDiffer {
                file,
                expected_orders,
                expected_units,
                expected_runs,
                found_orders,
                found_units,
                found_runs,
            } => write!(
                f,
                "{file}: the printed counts differ from the plan (printed {} orders · {} units · \
                 {} runs; the plan has {expected_orders} orders · {expected_units} units · \
                 {expected_runs} runs).",
                maybe(*found_orders),
                maybe(*found_units),
                maybe(*found_runs)
            ),
            Finding::PageOrderDiffers { pdf, file } => write!(
                f,
                "PDF {pdf} ({file}): every page is there, but not in the run order."
            ),
        }
    }
}

/// The check's two lists, kept apart (07 › *Check before saving*).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Check {
    pub errors: Vec<Finding>,
    pub warnings: Vec<Finding>,
}

impl Check {
    /// Whether nothing was found; a save with no findings renames straight away.
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty() && self.warnings.is_empty()
    }

    /// The errors as text, for `state.json` and `summary.txt`.
    pub fn error_texts(&self) -> Vec<String> {
        self.errors.iter().map(Finding::to_string).collect()
    }

    /// The warnings as text, for `state.json` and `summary.txt`.
    pub fn warning_texts(&self) -> Vec<String> {
        self.warnings.iter().map(Finding::to_string).collect()
    }
}

/// The page sequence of every saved PDF: the pick's runs, their orders, each repeated once per
/// page (07 › *Check before saving*).
pub fn expected_pdfs(plan: &PlanResult, orders: &[ReadOrder]) -> Vec<ExpectedPdf> {
    let index: BTreeMap<&str, &ReadOrder> = orders
        .iter()
        .map(|order| (order.order.order_id.as_str(), order))
        .collect();
    plan.picks
        .iter()
        .map(|pick| {
            let mut pages = Vec::new();
            for run in &pick.runs {
                for id in &run.orders {
                    let count = index.get(id.as_str()).map_or(0, |order| order.pages.len());
                    for _ in 0..count {
                        pages.push(id.clone());
                    }
                }
            }
            ExpectedPdf {
                number: pick.number,
                file: pick.file_name.clone(),
                pages,
            }
        })
        .collect()
}

/// Compare the plan with the read-back description (07 › *Check before saving*): every order
/// exactly once, in one saved PDF, page for page and in order; every packing list's counts equal
/// the plan's.
pub fn compare(expected: &Expected, written: &Written) -> Check {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    for pdf in &expected.pdfs {
        let actual: &[Option<String>] = written
            .pdfs
            .iter()
            .find(|read| read.file == pdf.file)
            .map_or(&[], |read| read.pages.as_slice());
        if actual.len() != pdf.pages.len() {
            errors.push(Finding::PageCountDiffersFromPlan {
                pdf: pdf.number,
                file: pdf.file.clone(),
                expected: pdf.pages.len(),
                found: actual.len(),
            });
        }
        let want = order_counts(pdf.pages.iter().map(String::as_str));
        let have = order_counts(actual.iter().flatten().map(String::as_str));

        // A page holding another pick's label.
        for order in have.keys() {
            if want.contains_key(order) {
                continue;
            }
            for (index, id) in actual.iter().enumerate() {
                if id.as_deref() == Some(order.as_str()) {
                    errors.push(Finding::WrongOrderLabelOnPage {
                        pdf: pdf.number,
                        file: pdf.file.clone(),
                        page: index + 1,
                        order_id: order.clone(),
                    });
                }
            }
        }
        // A page missing, or twice, in this saved PDF.
        for (order, expected_count) in &want {
            let found = have.get(order).copied().unwrap_or(0);
            if found < *expected_count {
                errors.push(Finding::MissingPage {
                    pdf: pdf.number,
                    file: pdf.file.clone(),
                    order_id: order.clone(),
                    expected: *expected_count,
                    found,
                });
            } else if found > *expected_count {
                errors.push(Finding::DoubledPageInOnePdf {
                    pdf: pdf.number,
                    file: pdf.file.clone(),
                    order_id: order.clone(),
                    expected: *expected_count,
                    found,
                });
            }
        }
        // The warning: all pages are there exactly once, in another order.
        if actual.iter().all(Option::is_some) && actual.len() == pdf.pages.len() && have == want {
            let sequence: Vec<&str> = actual.iter().filter_map(|id| id.as_deref()).collect();
            let wanted: Vec<&str> = pdf.pages.iter().map(String::as_str).collect();
            if sequence != wanted {
                warnings.push(Finding::PageOrderDiffers {
                    pdf: pdf.number,
                    file: pdf.file.clone(),
                });
            }
        }
    }

    // An order in no saved PDF, or in two: over the whole batch.
    let mut places: BTreeMap<String, Vec<i64>> = BTreeMap::new();
    for pdf in &expected.pdfs {
        for order in &pdf.pages {
            places.entry(order.clone()).or_default();
        }
    }
    for read in &written.pdfs {
        let Some(pdf) = expected.pdfs.iter().find(|p| p.file == read.file) else {
            continue;
        };
        let mut seen: Vec<&str> = Vec::new();
        for order in read.pages.iter().flatten() {
            if seen.contains(&order.as_str()) {
                continue;
            }
            seen.push(order.as_str());
            places.entry(order.clone()).or_default().push(pdf.number);
        }
    }
    for (order, pdfs) in &places {
        if pdfs.is_empty() {
            errors.push(Finding::OrderInNoPdf {
                order_id: order.clone(),
            });
        } else if pdfs.len() > 1 {
            errors.push(Finding::OrderInTwoPdfs {
                order_id: order.clone(),
                pdfs: pdfs.clone(),
            });
        }
    }

    // The packing lists' printed counts.
    for list in &expected.lists {
        let read = written.lists.iter().find(|read| read.file == list.file);
        let (orders, units, runs) = match read {
            Some(read) => (read.orders, read.units, read.runs),
            None => (None, None, None),
        };
        if orders != Some(list.orders) || units != Some(list.units) || runs != Some(list.runs) {
            errors.push(Finding::PackingListCountsDiffer {
                file: list.file.clone(),
                expected_orders: list.orders,
                expected_units: list.units,
                expected_runs: list.runs,
                found_orders: orders,
                found_units: units,
                found_runs: runs,
            });
        }
    }

    Check { errors, warnings }
}

fn order_counts<'a>(ids: impl Iterator<Item = &'a str>) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for id in ids {
        *counts.entry(id.to_string()).or_default() += 1;
    }
    counts
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orders::{Order, PageRef};

    fn read_order(id: &str, pages: usize) -> ReadOrder {
        ReadOrder {
            order: Order {
                order_id: id.to_string(),
                tracking_id: format!("JY{id}"),
                courier: "J&T Express".to_string(),
                ship_by: None,
                lines: Vec::new(),
                customer_message: String::new(),
            },
            pages: (0..pages).map(|page| PageRef { file: 0, page }).collect(),
            position: 1,
            has_slip: true,
            qty_total: None,
        }
    }

    fn plan() -> PlanResult {
        use crate::plan::{PickResult, RunResult};
        PlanResult {
            picks: vec![PickResult {
                number: 1,
                code: "A".to_string(),
                name: "Sepatu".to_string(),
                orders: vec![
                    "580000000000000001".to_string(),
                    "580000000000000002".to_string(),
                ],
                runs: vec![RunResult {
                    number: 1,
                    orders: vec![
                        "580000000000000001".to_string(),
                        "580000000000000002".to_string(),
                    ],
                }],
                pages: 3,
                file_name: "1 A Sepatu ×2.pdf".to_string(),
            }],
        }
    }

    #[test]
    fn expected_pages_repeat_a_multi_page_order_once_per_page() {
        let orders = vec![
            read_order("580000000000000001", 2),
            read_order("580000000000000002", 1),
        ];
        let pdfs = expected_pdfs(&plan(), &orders);
        assert_eq!(pdfs.len(), 1);
        assert_eq!(pdfs[0].number, 1);
        assert_eq!(pdfs[0].file, "1 A Sepatu ×2.pdf");
        assert_eq!(
            pdfs[0].pages,
            vec![
                "580000000000000001".to_string(),
                "580000000000000001".to_string(),
                "580000000000000002".to_string(),
            ]
        );
    }

    #[test]
    fn a_finding_names_itself_in_plain_words_and_keeps_the_severity() {
        let finding = Finding::MissingPage {
            pdf: 1,
            file: "1 A Sepatu ×2.pdf".to_string(),
            order_id: "580000000000000001".to_string(),
            expected: 1,
            found: 0,
        };
        assert_eq!(
            finding.to_string(),
            "PDF 1 (1 A Sepatu ×2.pdf): the page of order 580000000000000001 is missing \
             (0 of 1 written)."
        );
        assert!(!finding.is_warning());

        let warning = Finding::PageOrderDiffers {
            pdf: 1,
            file: "1 A Sepatu ×2.pdf".to_string(),
        };
        assert!(warning.is_warning());
        assert!(warning.to_string().contains("not in the run order"));

        let check = Check {
            errors: vec![finding],
            warnings: vec![warning],
        };
        assert!(!check.is_empty());
        assert_eq!(check.error_texts().len(), 1);
        assert_eq!(check.warning_texts().len(), 1);
        assert!(Check::default().is_empty());
    }
}
