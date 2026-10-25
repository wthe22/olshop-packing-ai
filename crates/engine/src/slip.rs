//! The slip table read by position: the columns of a page's packing slip, and the rows between
//! them.

use std::cmp::Ordering;
use std::sync::OnceLock;

use regex::Regex;

use crate::text::{PageText, Run};

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

/// The job of one slip column: which value of a `SlipLine` its cell holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ColumnRole {
    Name,
    Variation,
    SellerSku,
    Qty,
}

/// A slip's table header: its four words left to right, each with the role of its column.
struct SlipHeader {
    words: [(&'static str, ColumnRole); 4],
}

/// The TikTok and Tokopedia slip; its `SKU` column holds the variation.
const ENGLISH_SLIP: SlipHeader = SlipHeader {
    words: [
        ("Product Name", ColumnRole::Name),
        ("SKU", ColumnRole::Variation),
        ("Seller SKU", ColumnRole::SellerSku),
        ("Qty", ColumnRole::Qty),
    ],
};

/// The Shopee-style slip: the same three cells under Indonesian names, and its `SKU` column
/// holds the seller SKU where the English slip's holds the variation.
const SHOPEE_SLIP: SlipHeader = SlipHeader {
    words: [
        ("Nama Produk", ColumnRole::Name),
        ("SKU", ColumnRole::SellerSku),
        ("Variasi", ColumnRole::Variation),
        ("Qty", ColumnRole::Qty),
    ],
};

const SLIP_HEADERS: [SlipHeader; 2] = [ENGLISH_SLIP, SHOPEE_SLIP];

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
    let table_end = table_end(runs);
    let rows = rows(runs, &columns, header_y, table_end);
    let lower_bound = table_end.unwrap_or(f64::NEG_INFINITY);
    let lines = cells(runs, &columns, &rows, lower_bound)
        .into_iter()
        .map(|(cells, quantity)| slip_line(&columns, &cells, quantity))
        .collect();
    Some(Slip {
        lines,
        qty_total: qty_total(runs),
        customer_message: customer_message(runs),
    })
}

/// A slip column: its role, its left edge (the header word's x - `COLUMN_TOLERANCE`) and its
/// right edge (the next column's word x, the page width after the last).
type Column = (ColumnRole, f64, f64);

/// The header line's y and its columns: the first line that holds one of the slip headers.
fn find_header(runs: &[Run], page_width: f64) -> Option<(f64, Vec<Column>)> {
    for line in lines(runs) {
        let (text, starts) = joined(&line);
        for header in &SLIP_HEADERS {
            if let Some(columns) = columns_of(&text, &starts, header, page_width) {
                return Some((line[0].y, columns));
            }
        }
    }
    None
}

/// The page's runs grouped into lines: a run joins a line while its y is within
/// `LINE_TOLERANCE` of the line's first run. Each line's runs are in x order.
fn lines(runs: &[Run]) -> Vec<Vec<&Run>> {
    let mut sorted: Vec<&Run> = runs.iter().collect();
    sorted.sort_by(|a, b| cmp_asc(b.y, a.y).then_with(|| cmp_asc(a.x, b.x)));
    let mut lines: Vec<Vec<&Run>> = Vec::new();
    for run in sorted {
        match lines.last_mut() {
            Some(line) if (line[0].y - run.y).abs() < LINE_TOLERANCE => line.push(run),
            _ => lines.push(vec![run]),
        }
    }
    lines
}

/// The line's text — its runs' texts joined with no separator — and each run's start offset.
/// Joining without a separator is what lets a header word the style splits across two runs
/// (`V` + `ariasi`) be found.
fn joined<'a>(line: &[&'a Run]) -> (String, Vec<(&'a Run, usize)>) {
    let mut text = String::new();
    let mut starts = Vec::with_capacity(line.len());
    for run in line {
        starts.push((*run, text.len()));
        text.push_str(&run.text);
    }
    (text, starts)
}

/// The columns of a header whose four words sit in this line's text at strictly increasing
/// offsets; `None` when the line does not hold the header.
fn columns_of(
    text: &str,
    starts: &[(&Run, usize)],
    header: &SlipHeader,
    page_width: f64,
) -> Option<Vec<Column>> {
    let xs = word_xs(text, starts, header)?;
    let mut columns = Vec::with_capacity(xs.len());
    for (index, (_, role)) in header.words.iter().enumerate() {
        let right = xs.get(index + 1).copied().unwrap_or(page_width);
        columns.push((*role, xs[index] - COLUMN_TOLERANCE, right));
    }
    Some(columns)
}

/// The x of the run holding each of the header's words, in header order; `None` when a word is
/// missing or comes before an earlier one.
fn word_xs(text: &str, starts: &[(&Run, usize)], header: &SlipHeader) -> Option<Vec<f64>> {
    let mut xs = Vec::with_capacity(header.words.len());
    let mut from = 0usize;
    for (word, _) in header.words.iter() {
        let at = text[from..].find(*word)? + from;
        xs.push(run_at(starts, at).x);
        from = at + word.len();
    }
    Some(xs)
}

/// The run whose text holds the character at byte `offset`.
fn run_at<'a>(starts: &[(&'a Run, usize)], offset: usize) -> &'a Run {
    starts
        .iter()
        .rev()
        .find(|(_, start)| *start <= offset)
        .map(|(run, _)| *run)
        .expect("the line's first run holds the first character")
}

/// The caption that ends the slip's table and its y: `Qty Total:` on the TikTok and Tokopedia
/// slips, `Pesan:` on the Shopee-style one, whose table prints no total. The runs below it — the
/// caption's value and the footer's tracking IDs — are not part of the last row.
fn table_end(runs: &[Run]) -> Option<f64> {
    runs.iter()
        .filter(|run| run.text.starts_with("Qty Total") || run.text.starts_with("Pesan:"))
        .map(|run| run.y)
        .reduce(f64::max)
}

/// The number the slip prints beside its `Qty Total:` caption; the cross-check against the sum of
/// the rows' quantities.
fn qty_total(runs: &[Run]) -> Option<i64> {
    let caption = runs.iter().find(|run| run.text.starts_with("Qty Total"))?;
    runs.iter()
        .find(|other| {
            other.x > caption.x
                && (other.y - caption.y).abs() < LINE_TOLERANCE
                && is_int(&other.text)
        })
        .and_then(|other| other.text.parse::<i64>().ok())
}

/// (y, quantity) per row, top first; a row starts at each Qty value.
fn rows(
    runs: &[Run],
    columns: &[Column],
    header_y: f64,
    table_end: Option<f64>,
) -> Vec<(f64, i64)> {
    let (left, right) = columns
        .iter()
        .find(|(role, _, _)| *role == ColumnRole::Qty)
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
        if let Some(end) = table_end
            && run.y <= end + LINE_TOLERANCE
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

/// One row's cells placed by their columns' roles; a variation cell of `Default` is stored as
/// no variation.
fn slip_line(columns: &[Column], cells: &[String], quantity: i64) -> SlipLine {
    let mut line = SlipLine {
        name: String::new(),
        variation: String::new(),
        seller_sku: String::new(),
        quantity,
    };
    for ((role, _, _), cell) in columns.iter().zip(cells) {
        match role {
            ColumnRole::Name => line.name = cell.clone(),
            ColumnRole::Variation => {
                if cell != DEFAULT_VARIATION {
                    line.variation = cell.clone();
                }
            }
            ColumnRole::SellerSku => line.seller_sku = cell.clone(),
            // The quantity is the row's Qty cell, which `rows` already read as the row's key.
            ColumnRole::Qty => {}
        }
    }
    line
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
            // By x first, then y, then the text.
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
