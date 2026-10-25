//! The label fields read from a page's text: the Order ID, the tracking ID and the
//! `In transit by` deadline.

use std::sync::OnceLock;

use jiff::civil::DateTime;
use regex::Regex;

use crate::ids::{OrderId, TrackingId};

use super::CourierTable;

/// The characters trimmed from both ends of a tracking token.
const TOKEN_TRIM: &str = " \t.,;:";

pub(crate) fn whitespace_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"\s+").expect("built-in whitespace regex"))
}

fn order_id_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    // The pattern is (?<!\d)(5\d{17})(?!\d). The `regex` crate has no lookaround, so the digit
    // boundaries are checked by hand from the surrounding characters in `find_order_id`.
    R.get_or_init(|| Regex::new(r"5\d{17}").expect("built-in Order ID regex"))
}

fn labelled_order_id_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    // The second colon is fullwidth U+FF1A.
    R.get_or_init(|| {
        Regex::new("OrderI[dD][:\u{FF1A}](\\d{18})").expect("built-in labelled Order ID regex")
    })
}

fn shopee_order_id_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    // The Shopee-style label's own order number: six digits then eight letters or digits. The
    // `regex` crate has no lookaround, so `find_order_id` checks by hand that a longer token is
    // not cut down to its first fourteen characters.
    R.get_or_init(|| {
        Regex::new(r"No\.Pesanan:\s*(\d{6}[0-9A-Z]{8})").expect("built-in Shopee Order ID regex")
    })
}

fn ship_by_raw_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"In\s+transit\s+by\s*:\s*(\d{2})/(\d{2})/(\d{4})\s+(\d{2}):(\d{2})")
            .expect("built-in ship_by regex")
    })
}

fn ship_by_collapsed_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"Intransitby:(\d{2})/(\d{2})/(\d{4})(\d{2}):(\d{2})")
            .expect("built-in collapsed ship_by regex")
    })
}

fn ship_by_day_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    // The Shopee-style label's deadline: a date with no time. The `\s*` lets the one pattern read
    // both the raw text and a spaced-out page's collapsed text.
    R.get_or_init(|| {
        Regex::new(r"Batas\s*Kirim:\s*(\d{2})-(\d{2})-(\d{4})").expect("built-in ship_by day regex")
    })
}

/// The `(?<!\d)` lookbehind of the combined pattern: only needed when the pattern starts with
/// a digit; a left neighbour that is a digit rejects the match.
pub(super) fn has_digit_left(text: &str, start: usize) -> bool {
    text[..start]
        .chars()
        .next_back()
        .is_some_and(|c| c.is_numeric())
}

/// The `(?!\d)` lookahead of the combined pattern: a right neighbour that is a digit rejects it.
pub(super) fn has_digit_right(text: &str, end: usize) -> bool {
    text[end..].chars().next().is_some_and(|c| c.is_numeric())
}

/// `labels.find_order_id`: the Order ID on a page, or `None`.
pub fn find_order_id(text: &str) -> Option<OrderId> {
    for m in order_id_re().find_iter(text) {
        let (start, end) = (m.start(), m.end());
        if !has_digit_left(text, start) && !has_digit_right(text, end) {
            return Some(OrderId::new(m.as_str()));
        }
    }
    // Whitespace is dropped only in this second pass: on J&T labels the Order ID sits on its own
    // line right under the tracking ID, and removing whitespace first would glue the two into
    // one number. So the raw search must come first.
    let collapsed = whitespace_re().replace_all(text, "");
    if let Some(caps) = labelled_order_id_re().captures(&collapsed) {
        return Some(OrderId::new(&caps[1]));
    }
    // The Shopee-style label prints no 18-digit Order ID; it keys the order by this caption.
    // Last, so no other style's read changes. The raw text is searched first: dropping the
    // whitespace would glue the caption's number to the word printed below it, and the guard
    // that rejects a longer token would then reject every page.
    shopee_order_id(text).or_else(|| shopee_order_id(&collapsed))
}

/// The number of the Shopee-style label's `No.Pesanan:` caption in `text`, or `None` when the
/// caption is absent or the token is longer than its fourteen characters.
fn shopee_order_id(text: &str) -> Option<OrderId> {
    let caps = shopee_order_id_re().captures(text)?;
    let end = caps.get(1).expect("the Shopee caption's number").end();
    let longer = text[end..]
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_digit() || c.is_ascii_uppercase());
    if longer {
        return None;
    }
    Some(OrderId::new(&caps[1]))
}

fn bump(counts: &mut Vec<(String, usize)>, token: &str) {
    if let Some(entry) = counts.iter_mut().find(|(t, _)| t == token) {
        entry.1 += 1;
    } else {
        counts.push((token.to_string(), 1));
    }
}

fn most_frequent(counts: &[(String, usize)]) -> Option<&str> {
    let mut best: Option<&(String, usize)> = None;
    for entry in counts {
        if best.is_none_or(|b| entry.1 > b.1) {
            best = Some(entry);
        }
    }
    best.map(|(token, _)| token.as_str())
}

/// One tracking-ID form: its pattern as written, matched fully against a token and unanchored
/// against the whitespace-collapsed text.
pub struct Form<'a> {
    pub source: &'a str,
    pub full: &'a Regex,
    pub unanchored: &'a Regex,
}

/// The most frequent tracking token of the text, `None` when none matches a courier form of
/// `table`, the data folder's courier data.
pub fn find_tracking_id(
    text: &str,
    order_id: &OrderId,
    table: &CourierTable,
) -> Option<TrackingId> {
    let forms: Vec<Form<'_>> = table
        .couriers
        .iter()
        .filter_map(|courier| {
            Some(Form {
                source: courier.tracking.as_deref()?,
                full: courier.tracking_full.as_ref()?,
                unanchored: courier.tracking_re.as_ref()?,
            })
        })
        .collect();
    find_tracking_id_in(text, order_id, &forms)
}

/// The most frequent token of the text that one of `forms` matches fully, `None` when none does.
/// The owner's file decides which forms name a courier, so a form added there is read without a
/// program change.
pub fn find_tracking_id_in(
    text: &str,
    order_id: &OrderId,
    forms: &[Form<'_>],
) -> Option<TrackingId> {
    let mut counts: Vec<(String, usize)> = Vec::new();
    for raw in text.split_whitespace() {
        let token = raw.trim_matches(|c| TOKEN_TRIM.contains(c));
        if token.is_empty() || token == order_id.as_str() {
            continue;
        }
        if forms.iter().any(|form| form.full.is_match(token)) {
            bump(&mut counts, token);
        }
    }
    if counts.is_empty() {
        // Spaced-out IDX/Tokopedia pages: the tracking ID is only whole in the collapsed text.
        let collapsed = whitespace_re().replace_all(text, "");
        for form in forms {
            let (pattern, re) = (form.source, form.unanchored);
            let left_boundary = pattern.starts_with(|c: char| c.is_ascii_digit());
            for m in re.find_iter(&collapsed) {
                if left_boundary && has_digit_left(&collapsed, m.start()) {
                    continue;
                }
                if has_digit_right(&collapsed, m.end()) {
                    continue;
                }
                if m.as_str() != order_id.as_str() {
                    bump(&mut counts, m.as_str());
                }
            }
        }
    }
    most_frequent(&counts).map(TrackingId::new)
}

/// A label deadline from its parts; an impossible date or time is `None`.
fn datetime(year: i64, month: i64, day: i64, hour: i64, minute: i64) -> Option<DateTime> {
    DateTime::new(
        i16::try_from(year).ok()?,
        i8::try_from(month).ok()?,
        i8::try_from(day).ok()?,
        i8::try_from(hour).ok()?,
        i8::try_from(minute).ok()?,
        0,
        0,
    )
    .ok()
}

fn build_datetime(caps: &regex::Captures<'_>) -> Option<DateTime> {
    let part = |i: usize| caps[i].parse::<i64>().ok();
    datetime(part(3)?, part(2)?, part(1)?, part(4)?, part(5)?)
}

/// The deadline a bare date gives: that day at 00:00.
fn build_day(caps: &regex::Captures<'_>) -> Option<DateTime> {
    let part = |i: usize| caps[i].parse::<i64>().ok();
    datetime(part(3)?, part(2)?, part(1)?, 0, 0)
}

/// `labels.find_ship_by`: the label's `In transit by: dd/mm/yyyy hh:mm` deadline, or the
/// Shopee-style label's `Batas Kirim: dd-mm-yyyy` day; `None` when absent.
pub fn find_ship_by(text: &str) -> Option<DateTime> {
    if let Some(caps) = ship_by_raw_re().captures(text) {
        return build_datetime(&caps);
    }
    let collapsed = whitespace_re().replace_all(text, "");
    if let Some(caps) = ship_by_collapsed_re().captures(&collapsed) {
        return build_datetime(&caps);
    }
    ship_by_day_re()
        .captures(text)
        .or_else(|| ship_by_day_re().captures(&collapsed))
        .and_then(|caps| build_day(&caps))
}
