//! The check before saving's read-back: read the written saved PDFs
//! and packing lists back and build the description the check compares with the plan.
//!
//! The pages come from the same page reader as the reading step ([`crate::worker`]'s
//! `read_back`, no new parser); the packing lists' counts are read from the header line they
//! print.

use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;

use packing_engine::ids::OrderId;
use packing_engine::label::find_order_id;
use packing_engine::orders::PageItem;
use packing_engine::text::PageText;
use packing_engine::verify::{Expected, Written, WrittenList, WrittenPdf};

use crate::PdfError;
use crate::worker::{PdfWorker, SaveProgress};

/// Read every written saved PDF page by page and every packing list's printed counts back.
/// `expected.pdfs` come first, then `expected.lists`.
pub fn read_back(
    worker: &PdfWorker,
    folder: &Path,
    expected: &Expected,
    progress: Sender<SaveProgress>,
) -> Result<Written, PdfError> {
    let mut paths: Vec<PathBuf> = Vec::with_capacity(expected.pdfs.len() + expected.lists.len());
    for pdf in &expected.pdfs {
        paths.push(folder.join(&pdf.file));
    }
    let pdf_count = paths.len();
    for list in &expected.lists {
        paths.push(folder.join(&list.file));
    }
    if paths.is_empty() {
        return Ok(Written::default());
    }
    let items = worker.read_back(&paths, progress)?;

    let mut pdfs = Vec::with_capacity(expected.pdfs.len());
    for (index, pdf) in expected.pdfs.iter().enumerate() {
        pdfs.push(WrittenPdf {
            file: pdf.file.clone(),
            pages: order_ids(&pages_of(&items, index)),
        });
    }

    let mut lists = Vec::with_capacity(expected.lists.len());
    for (offset, list) in expected.lists.iter().enumerate() {
        let counts = pages_of(&items, pdf_count + offset)
            .first()
            .and_then(printed_counts);
        lists.push(WrittenList {
            file: list.file.clone(),
            orders: counts.map(|counts| counts.0),
            units: counts.map(|counts| counts.1),
            runs: counts.map(|counts| counts.2),
        });
    }

    Ok(Written { pdfs, lists })
}

fn pages_of(items: &[PageItem], file: usize) -> Vec<PageText> {
    items
        .iter()
        .filter(|(index, _, _)| *index == file)
        .map(|(_, _, text)| text.clone())
        .collect()
}

/// The Order ID of every page, in order: a page with none continues the previous order (the
/// reading step groups pages the same way).
pub fn order_ids(pages: &[PageText]) -> Vec<Option<OrderId>> {
    let mut ids = Vec::with_capacity(pages.len());
    let mut previous: Option<OrderId> = None;
    for page in pages {
        let found = find_order_id(&page.text);
        if let Some(id) = &found {
            previous = Some(id.clone());
        }
        ids.push(found.or_else(|| previous.clone()));
    }
    ids
}

/// The counts printed on a packing list's header line (`54 orders · 77 units · 9 runs · printed
/// 13:15`), `None` when that line cannot be read. A section bar's shorter `N orders · M units`
/// line is skipped.
pub fn printed_counts(page: &PageText) -> Option<(usize, i64, usize)> {
    for run in &page.runs {
        let text = run.text.as_str();
        if !text.contains(" orders · ") {
            continue;
        }
        let parts: Vec<&str> = text.split(" · ").collect();
        let number = |part: Option<&&str>| -> Option<i64> {
            part?
                .split_whitespace()
                .next()?
                .replace(',', "")
                .parse()
                .ok()
        };
        let (Some(orders), Some(units), Some(runs)) = (
            number(parts.first()),
            number(parts.get(1)),
            number(parts.get(2)),
        ) else {
            continue;
        };
        // The third part must be the `runs` count, not another line's units.
        if !parts.get(2).is_some_and(|part| part.ends_with("runs")) {
            continue;
        }
        return Some((
            usize::try_from(orders).ok()?,
            units,
            usize::try_from(runs).ok()?,
        ));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use packing_engine::text::{PageText, Run};

    fn page(runs: &[&str]) -> PageText {
        PageText {
            text: runs.join("\n"),
            runs: runs
                .iter()
                .map(|text| Run {
                    x: 0.0,
                    y: 0.0,
                    text: (*text).to_string(),
                })
                .collect(),
            width: 595.0,
            height: 842.0,
            logos: Vec::new(),
        }
    }

    #[test]
    fn the_header_line_gives_the_three_counts() {
        let header = "Packing list · 2026-10-06 · batch 1 · saved PDFs 1-2";
        let counts = "54 orders · 77 units · 9 runs · printed 13:15";
        assert_eq!(printed_counts(&page(&[header, counts])), Some((54, 77, 9)));
        // Thousands separators are read too.
        assert_eq!(
            printed_counts(&page(&[
                "1,234 orders · 2,345 units · 12 runs · printed 09:07"
            ])),
            Some((1234, 2345, 12))
        );
        // A page without the line, and a header line that is not it.
        assert_eq!(printed_counts(&page(&[header])), None);
        assert_eq!(printed_counts(&page(&[])), None);
        // A section bar's shorter line is not the header's line.
        assert_eq!(
            printed_counts(&page(&["7 orders · 8 units", header, counts])),
            Some((54, 77, 9))
        );
    }

    #[test]
    fn a_page_with_no_order_id_continues_the_previous_one() {
        let pages = vec![
            page(&["OrderId:580000000000000001"]),
            page(&["a long label, no Order ID"]),
            page(&["OrderId:580000000000000002"]),
        ];
        assert_eq!(
            order_ids(&pages),
            vec![
                Some(OrderId::new("580000000000000001")),
                Some(OrderId::new("580000000000000001")),
                Some(OrderId::new("580000000000000002")),
            ]
        );
        // A first page with nothing to continue stays unknown.
        assert_eq!(order_ids(&[page(&["nothing"])]), vec![None]);
    }
}
