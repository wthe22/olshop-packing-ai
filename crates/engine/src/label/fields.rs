//! The label fields read from a page's text: the Order ID, the tracking ID and the
//! `In transit by` deadline.

use std::sync::OnceLock;

use jiff::civil::DateTime;
use regex::Regex;

use crate::ids::{OrderId, TrackingId};

use super::couriers;

/// The characters `str.strip` removes around a tracking token (Python `_TOKEN_TRIM`).
const TOKEN_TRIM: &str = " \t.,;:";

pub(super) fn whitespace_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"\s+").expect("built-in whitespace regex"))
}

fn order_id_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    // Python: (?<!\d)(5\d{17})(?!\d). The `regex` crate has no lookaround, so the digit
    // boundaries are checked by hand from the surrounding characters in `find_order_id`.
    R.get_or_init(|| Regex::new(r"5\d{17}").expect("built-in Order ID regex"))
}

fn labelled_order_id_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    // Python: OrderI[dD][:：](\d{18}); the second colon is fullwidth U+FF1A.
    R.get_or_init(|| {
        Regex::new("OrderI[dD][:\u{FF1A}](\\d{18})").expect("built-in labelled Order ID regex")
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
    labelled_order_id_re()
        .captures(&collapsed)
        .map(|c| OrderId::new(&c[1]))
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

/// `labels.find_tracking_id`: the most frequent tracking token of the text, `None` when none
/// matches a courier form.
pub fn find_tracking_id(text: &str, order_id: &OrderId) -> Option<TrackingId> {
    let mut counts: Vec<(String, usize)> = Vec::new();
    for raw in text.split_whitespace() {
        let token = raw.trim_matches(|c| TOKEN_TRIM.contains(c));
        if token.is_empty() || token == order_id.as_str() {
            continue;
        }
        for courier in couriers() {
            if courier
                .tracking_full
                .as_ref()
                .is_some_and(|re| re.is_match(token))
            {
                bump(&mut counts, token);
                break;
            }
        }
    }
    if counts.is_empty() {
        // Spaced-out IDX/Tokopedia pages: the tracking ID is only whole in the collapsed text.
        let collapsed = whitespace_re().replace_all(text, "");
        for courier in couriers() {
            let (Some(pattern), Some(re)) = (&courier.tracking, &courier.tracking_re) else {
                continue;
            };
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

fn build_datetime(caps: &regex::Captures<'_>) -> Option<DateTime> {
    let part = |i: usize| caps[i].parse::<i64>().ok();
    let (day, month, year, hour, minute) = (part(1)?, part(2)?, part(3)?, part(4)?, part(5)?);
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

/// `labels.find_ship_by`: the label's `In transit by: dd/mm/yyyy hh:mm` deadline, `None` when
/// absent.
pub fn find_ship_by(text: &str) -> Option<DateTime> {
    if let Some(caps) = ship_by_raw_re().captures(text) {
        return build_datetime(&caps);
    }
    let collapsed = whitespace_re().replace_all(text, "");
    ship_by_collapsed_re()
        .captures(&collapsed)
        .and_then(|caps| build_datetime(&caps))
}
