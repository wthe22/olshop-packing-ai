//! The slip table read by position (03 › *Reading the packing slip*; port of
//! `script/packing/slip.py`, validated against Python in the spike on 600 sample and 23 testdata
//! orders).

use std::cmp::Ordering;
use std::sync::OnceLock;

use regex::Regex;

use crate::text::{PageText, Run};

const HEADER_WORDS: [&str; 4] = ["Product Name", "SKU", "Seller SKU", "Qty"];
const COLUMN_TOLERANCE: f64 = 3.0; // a column starts this much left of its header word
const LINE_TOLERANCE: f64 = 0.6; // a run a hair above a row's y still belongs to that row
const EXCLUDED: [&str; 3] = ["Order ID", "Qty Total", "Customer Message"];
const DEFAULT_VARIATION: &str = "Default";

/// One slip row: the Product Name, the variation (``""`` when the slip says `Default`), the
/// Seller SKU and the quantity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlipLine {
    pub name: String,
    pub variation: String,
    pub seller_sku: String,
    pub quantity: i64,
}

/// The slip table of one page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slip {
    pub lines: Vec<SlipLine>,
    pub qty_total: Option<i64>,
    pub customer_message: String,
}

fn int_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"^-?\d+$").expect("built-in slip int regex"))
}

fn is_int(text: &str) -> bool {
    int_re().is_match(text)
}

fn cmp_asc(a: f64, b: f64) -> Ordering {
    a.total_cmp(&b)
}

/// The slip table of one page, or `None` when the page carries no slip header.
/// Mirrors `slip.Slip` / `slip._slip`.
pub fn read_slip(page: &PageText) -> Option<Slip> {
    let runs = &page.runs;
    let (header_y, columns) = find_header(runs, page.width)?;
    let (qty_total_y, qty_total) = qty_total(runs);
    let rows = rows(runs, &columns, header_y, qty_total_y);
    let lower_bound = qty_total_y.unwrap_or(f64::NEG_INFINITY);
    let lines = cells(runs, &columns, &rows, lower_bound)
        .into_iter()
        .map(|(cells, quantity)| SlipLine {
            name: cells[0].clone(),
            variation: if cells[1] == DEFAULT_VARIATION {
                String::new()
            } else {
                cells[1].clone()
            },
            seller_sku: cells[2].clone(),
            quantity,
        })
        .collect();
    Some(Slip {
        lines,
        qty_total,
        customer_message: customer_message(runs),
    })
}

/// A slip column: the header word, the left edge (its x - 3.0) and the right edge.
type Column = (String, f64, f64);

/// (header y, columns) where each column is (word, left edge - 3.0, right edge).
fn find_header(runs: &[Run], page_width: f64) -> Option<(f64, Vec<Column>)> {
    let mut positions: Vec<(&str, Vec<(f64, f64)>)> =
        HEADER_WORDS.iter().map(|&w| (w, Vec::new())).collect();
    for run in runs {
        if let Some(slot) = positions.iter_mut().find(|(w, _)| *w == run.text) {
            slot.1.push((run.x, run.y));
        }
    }
    if positions.iter().any(|(_, v)| v.is_empty()) {
        return None;
    }
    let name_positions = positions
        .iter()
        .find(|(w, _)| *w == "Product Name")
        .map(|(_, v)| v.clone())
        .unwrap_or_default();
    for (_, name_y) in name_positions {
        let mut found: Vec<(&str, f64)> = Vec::new();
        let mut ok = true;
        for (word, xs_and_ys) in positions.iter() {
            let xs: Vec<f64> = xs_and_ys
                .iter()
                .filter(|(_, y)| (y - name_y).abs() < LINE_TOLERANCE)
                .map(|(x, _)| *x)
                .collect();
            if xs.is_empty() {
                ok = false;
                break;
            }
            found.push((word, xs.iter().copied().fold(f64::INFINITY, f64::min)));
        }
        if !ok {
            continue;
        }
        let mut ordered = found;
        ordered.sort_by(|a, b| cmp_asc(a.1, b.1));
        let mut columns = Vec::new();
        for (index, (word, left)) in ordered.iter().enumerate() {
            let right = if index + 1 < ordered.len() {
                ordered[index + 1].1
            } else {
                page_width
            };
            columns.push((word.to_string(), left - COLUMN_TOLERANCE, right));
        }
        return Some((name_y, columns));
    }
    None
}

fn qty_total(runs: &[Run]) -> (Option<f64>, Option<i64>) {
    for run in runs {
        if run.text.starts_with("Qty Total") {
            for other in runs {
                if other.x > run.x
                    && (other.y - run.y).abs() < LINE_TOLERANCE
                    && is_int(&other.text)
                {
                    return (Some(run.y), other.text.parse::<i64>().ok());
                }
            }
            return (Some(run.y), None);
        }
    }
    (None, None)
}

/// (y, quantity) per row, top first; a row starts at each Qty value.
fn rows(
    runs: &[Run],
    columns: &[Column],
    header_y: f64,
    qty_total_y: Option<f64>,
) -> Vec<(f64, i64)> {
    let (left, right) = columns
        .iter()
        .find(|(word, _, _)| word == "Qty")
        .map(|(_, l, r)| (*l, *r))
        .expect("Qty column");
    let mut result = Vec::new();
    for run in runs {
        if !(left <= run.x && run.x < right) {
            continue;
        }
        if run.y >= header_y - LINE_TOLERANCE {
            continue;
        }
        if let Some(ty) = qty_total_y
            && run.y <= ty + LINE_TOLERANCE
        {
            continue;
        }
        if is_int(&run.text) {
            result.push((run.y, run.text.parse::<i64>().unwrap()));
        }
    }
    result.sort_by(|a, b| cmp_asc(b.0, a.0));
    result
}

fn cells(
    runs: &[Run],
    columns: &[Column],
    rows: &[(f64, i64)],
    lower_bound: f64,
) -> Vec<(Vec<String>, i64)> {
    let excluded = |text: &str| EXCLUDED.iter().any(|prefix| text.starts_with(prefix));
    let mut out = Vec::new();
    for (index, (row_y, quantity)) in rows.iter().enumerate() {
        let low = if index + 1 < rows.len() {
            rows[index + 1].0
        } else {
            lower_bound
        };
        let mut result = Vec::new();
        for (_, left, right) in columns {
            let mut pieces: Vec<&Run> = runs
                .iter()
                .filter(|r| {
                    *left <= r.x
                        && r.x < *right
                        && low < r.y
                        && r.y <= row_y + LINE_TOLERANCE
                        && !excluded(&r.text)
                })
                .collect();
            result.push(join(&mut pieces));
        }
        out.push((result, *quantity));
    }
    out
}

/// Top-to-bottom, left-to-right; a wrap right after '-' joins without a space.
fn join(pieces: &mut [&Run]) -> String {
    pieces.sort_by(|a, b| cmp_asc(b.y, a.y).then_with(|| cmp_asc(a.x, b.x)));
    let mut out = String::new();
    for run in pieces.iter() {
        if !out.is_empty() && !out.ends_with('-') {
            out.push(' ');
        }
        out.push_str(&run.text);
    }
    out
}

fn customer_message(runs: &[Run]) -> String {
    for run in runs {
        if run.text == "Customer Message" {
            let mut pieces: Vec<&Run> = runs
                .iter()
                .filter(|p| {
                    (p.y - run.y).abs() < LINE_TOLERANCE
                        && p.x > run.x
                        && p.text != ":"
                        && p.text != "\u{FF1A}"
                })
                .collect();
            // Python `sorted(...)` over (x, y, text) tuples -> lexicographic on x first.
            pieces.sort_by(|a, b| {
                cmp_asc(a.x, b.x)
                    .then_with(|| cmp_asc(a.y, b.y))
                    .then_with(|| a.text.cmp(&b.text))
            });
            return pieces
                .iter()
                .map(|p| p.text.clone())
                .collect::<Vec<_>>()
                .join(" ");
        }
    }
    String::new()
}
