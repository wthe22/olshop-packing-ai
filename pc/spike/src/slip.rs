//! Pure port of script/packing/slip.py (slip table read by text position) and of
//! script/packing/labels.py's Order-ID two-pass search, over "runs" of (x, y, text).
//!
//! No PDF code here: it takes the runs rebuilt from Pdfium characters (see main.rs) and
//! returns the same slip lines and Order ID the Python reference returns.

use regex::Regex;
use std::cmp::Ordering;
use std::sync::OnceLock;

/// One text run: x is the left edge, y is the baseline (page co-ordinates, y grows upwards).
#[derive(Clone, Debug)]
pub struct Run {
    pub x: f64,
    pub y: f64,
    pub text: String,
}

const HEADER_WORDS: [&str; 4] = ["Product Name", "SKU", "Seller SKU", "Qty"];
const COLUMN_TOLERANCE: f64 = 3.0;
const LINE_TOLERANCE: f64 = 0.6;
const DEFAULT_VARIATION: &str = "Default";

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SlipLine {
    pub name: String,
    pub variation: String,
    pub seller_sku: String,
    pub quantity: i64,
}

#[derive(Clone, Debug)]
pub struct Slip {
    pub lines: Vec<SlipLine>,
    pub qty_total: Option<i64>,
    pub customer_message: String,
}

fn int_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"^-?\d+$").unwrap())
}

fn order_id_core_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    // Python: (?<!\d)(5\d{17})(?!\d). The `regex` crate has no lookaround, so the
    // digit boundaries are checked by hand from the surrounding characters below.
    R.get_or_init(|| Regex::new(r"5\d{17}").unwrap())
}

fn labelled_order_id_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    // Python: OrderI[dD][:：](\d{18})  (second colon U+FF1A fullwidth)
    R.get_or_init(|| Regex::new("OrderI[dD][:\u{FF1A}](\\d{18})").unwrap())
}

fn whitespace_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"\s+").unwrap())
}

fn is_int(text: &str) -> bool {
    int_re().is_match(text)
}

/// Which pass found the Order ID (1 = raw search, 2 = labelled search after collapsing
/// whitespace), or None when no Order ID is on the page. Mirrors labels.find_order_id.
pub fn find_order_id(text: &str) -> Option<(u8, String)> {
    for m in order_id_core_re().find_iter(text) {
        let start = m.start();
        let end = m.end();
        let before_ok = text[..start]
            .chars()
            .next_back()
            .map_or(true, |c| !c.is_numeric());
        let after_ok = text[end..].chars().next().map_or(true, |c| !c.is_numeric());
        if before_ok && after_ok {
            return Some((1, m.as_str().to_string()));
        }
    }
    let collapsed = whitespace_re().replace_all(text, "");
    if let Some(c) = labelled_order_id_re().captures(&collapsed) {
        return Some((2, c[1].to_string()));
    }
    None
}

/// The slip table of one page, or None when the page carries no slip header.
/// Mirrors slip.Slip / slip._slip.
pub fn slip(runs: &[Run], page_width: f64) -> Option<Slip> {
    let (header_y, columns) = find_header(runs, page_width)?;
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

/// (header y, columns) where each column is (word, left edge - 3.0, right edge).
fn find_header(runs: &[Run], page_width: f64) -> Option<(f64, Vec<(String, f64, f64)>)> {
    let mut positions: Vec<(&str, Vec<(f64, f64)>)> = HEADER_WORDS
        .iter()
        .map(|&w| (w, Vec::new()))
        .collect();
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
            found.push((word, xs.iter().cloned().fold(f64::INFINITY, f64::min)));
        }
        if !ok {
            continue;
        }
        let mut ordered = found.clone();
        ordered.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));
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
    columns: &[(String, f64, f64)],
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
        if let Some(ty) = qty_total_y {
            if run.y <= ty + LINE_TOLERANCE {
                continue;
            }
        }
        if is_int(&run.text) {
            result.push((run.y, run.text.parse::<i64>().unwrap()));
        }
    }
    result.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(Ordering::Equal));
    result
}

fn cells(
    runs: &[Run],
    columns: &[(String, f64, f64)],
    rows: &[(f64, i64)],
    lower_bound: f64,
) -> Vec<(Vec<String>, i64)> {
    let excluded = |text: &str| {
        text.starts_with("Order ID")
            || text.starts_with("Qty Total")
            || text.starts_with("Customer Message")
    };
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
    pieces.sort_by(|a, b| {
        b.y.partial_cmp(&a.y)
            .unwrap_or(Ordering::Equal)
            .then(a.x.partial_cmp(&b.x).unwrap_or(Ordering::Equal))
    });
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
                a.x.partial_cmp(&b.x)
                    .unwrap_or(Ordering::Equal)
                    .then(a.y.partial_cmp(&b.y).unwrap_or(Ordering::Equal))
                    .then(a.text.cmp(&b.text))
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
