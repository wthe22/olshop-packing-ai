//! The Order ID search, the label fields and the courier deduction (03 › *Order ID from the
//! label*, *Courier deduction*; port of `script/packing/labels.py`).

use std::sync::OnceLock;

use jiff::civil::DateTime;
use regex::Regex;
use serde::Deserialize;

/// The courier data built into the program (07 › *Inputs*).
const COURIERS_TOML: &str = include_str!("../couriers.toml");

/// The characters `str.strip` removes around a tracking token (Python `_TOKEN_TRIM`).
const TOKEN_TRIM: &str = " \t.,;:";

fn whitespace_re() -> &'static Regex {
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

/// One `[[courier]]` entry, its regexes compiled once.
pub struct Courier {
    pub name: String,
    /// Case-insensitive text clues searched in the page text.
    pub text: Vec<String>,
    /// The tracking-ID pattern source (as written in `couriers.toml`).
    pub tracking: Option<String>,
    /// `tracking` unanchored (used for the collapsed-text search).
    pub tracking_re: Option<Regex>,
    /// `tracking` anchored, i.e. a full match (Python `fullmatch`).
    pub tracking_full: Option<Regex>,
    /// Whether the tracking form alone identifies this courier.
    pub tracking_unique: bool,
}

#[derive(Deserialize)]
struct CouriersFile {
    #[serde(default)]
    courier: Vec<CourierEntry>,
}

#[derive(Deserialize)]
struct CourierEntry {
    name: String,
    #[serde(default)]
    text: Vec<String>,
    tracking: Option<String>,
    #[serde(default)]
    tracking_unique: bool,
}

/// The courier data, parsed from the built-in `couriers.toml` once.
pub fn couriers() -> &'static [Courier] {
    static COURIERS: OnceLock<Vec<Courier>> = OnceLock::new();
    COURIERS.get_or_init(|| {
        let file: CouriersFile =
            toml::from_str(COURIERS_TOML).expect("built-in couriers.toml is valid");
        file.courier
            .into_iter()
            .map(|entry| {
                let tracking_re = entry
                    .tracking
                    .as_deref()
                    .map(|p| Regex::new(p).expect("courier tracking pattern is valid"));
                let tracking_full = entry.tracking.as_deref().map(|p| {
                    Regex::new(&format!("^(?:{p})$")).expect("courier tracking pattern is valid")
                });
                Courier {
                    name: entry.name,
                    text: entry.text,
                    tracking: entry.tracking,
                    tracking_re,
                    tracking_full,
                    tracking_unique: entry.tracking_unique,
                }
            })
            .collect()
    })
}

/// The `(?<!\d)` lookbehind of the combined pattern: only needed when the pattern starts with
/// a digit; a left neighbour that is a digit rejects the match.
fn has_digit_left(text: &str, start: usize) -> bool {
    text[..start]
        .chars()
        .next_back()
        .is_some_and(|c| c.is_numeric())
}

/// The `(?!\d)` lookahead of the combined pattern: a right neighbour that is a digit rejects it.
fn has_digit_right(text: &str, end: usize) -> bool {
    text[end..].chars().next().is_some_and(|c| c.is_numeric())
}

/// `labels.find_order_id`: the Order ID on a page, or `None`.
pub fn find_order_id(text: &str) -> Option<String> {
    for m in order_id_re().find_iter(text) {
        let (start, end) = (m.start(), m.end());
        if !has_digit_left(text, start) && !has_digit_right(text, end) {
            return Some(m.as_str().to_string());
        }
    }
    // Whitespace is dropped only in this second pass: on J&T labels the Order ID sits on its own
    // line right under the tracking ID, and removing whitespace first would glue the two into
    // one number. So the raw search must come first.
    let collapsed = whitespace_re().replace_all(text, "");
    labelled_order_id_re()
        .captures(&collapsed)
        .map(|c| c[1].to_string())
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

/// `labels.find_tracking_id`: the most frequent tracking token of the text, `""` when none
/// matches a courier form.
pub fn find_tracking_id(text: &str, order_id: &str) -> String {
    let mut counts: Vec<(String, usize)> = Vec::new();
    for raw in text.split_whitespace() {
        let token = raw.trim_matches(|c| TOKEN_TRIM.contains(c));
        if token.is_empty() || token == order_id {
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
                if m.as_str() != order_id {
                    bump(&mut counts, m.as_str());
                }
            }
        }
    }
    most_frequent(&counts).unwrap_or_default().to_string()
}

/// `labels.deduce_courier`: the courier from the label text clues, else an unambiguous tracking
/// form, else `""`.
pub fn deduce_courier(text: &str, tracking_id: &str) -> String {
    let lowered = text.to_lowercase();
    let collapsed = whitespace_re().replace_all(text, "").to_lowercase();
    for courier in couriers() {
        if courier.text.iter().any(|clue| {
            lowered.contains(&clue.to_lowercase()) || collapsed.contains(&clue.to_lowercase())
        }) {
            return courier.name.clone();
        }
    }
    for courier in couriers() {
        if courier.tracking_unique
            && !tracking_id.is_empty()
            && courier
                .tracking_full
                .as_ref()
                .is_some_and(|re| re.is_match(tracking_id))
        {
            return courier.name.clone();
        }
    }
    String::new()
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
