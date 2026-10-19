//! The Order ID search, the label fields, the courier deduction and the courier data
//! (03 › *Order ID from the label*, *Courier deduction*; port of `tools/analysis/packing/labels.py`).
//!
//! The courier data lives in the data folder's `couriers.toml` and is the owner's, like
//! `categories.toml`; the bundled file is the default of a fresh data folder and of a folder
//! without the file (07 › *Inputs*, D-LABEL-1).

use std::fmt;
use std::path::Path;
use std::sync::OnceLock;

use jiff::civil::DateTime;
use regex::Regex;

use crate::ids::{OrderId, TrackingId};
use crate::text::LogoImage;

/// The courier data built into the program, written into a new data folder (07 › *Inputs*).
pub const COURIERS_TOML: &str = include_str!("../couriers.toml");

/// The courier file in the data folder (07 › *Inputs*).
pub const COURIERS_FILE_NAME: &str = "couriers.toml";

/// Invalid courier file; the message names the place (`couriers.toml`, its line).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CouriersError {
    pub message: String,
}

impl CouriersError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for CouriersError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for CouriersError {}

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
#[derive(Clone, Debug)]
pub struct Courier {
    pub name: String,
    /// The courier's logo images: XXH3-64 (16 hex) of each one's **encoded bytes**.
    pub logo: Vec<String>,
    /// The courier's logos by **appearance**: XXH3-64 (16 hex) of a 64 × 64 1-bit canvas. The
    /// clue for a **masked** logo, whose encoded bytes name nothing (J&T Cargo, courier.md item 3).
    pub shape: Vec<String>,
    /// Case-insensitive text clues searched in the page text: the logo's stand-in where the logo
    /// cannot be read at all (courier.md item 5).
    pub text: Vec<String>,
    /// The tracking-ID pattern source (as written in `couriers.toml`).
    pub tracking: Option<String>,
    /// `tracking` unanchored (used for the collapsed-text search).
    pub tracking_re: Option<Regex>,
    /// `tracking` anchored, i.e. a full match (Python `fullmatch`).
    pub tracking_full: Option<Regex>,
}

impl Courier {
    /// One courier for a table of the rule's tests; `tracking` is a regex source.
    pub fn new(
        name: &str,
        logo: &[&str],
        shape: &[&str],
        tracking: Option<&str>,
        text: &[&str],
    ) -> Courier {
        Courier::from_entry(
            name.to_string(),
            logo.iter().map(|value| (*value).to_string()).collect(),
            shape.iter().map(|value| (*value).to_string()).collect(),
            text.iter().map(|value| (*value).to_string()).collect(),
            tracking.map(str::to_string),
        )
    }

    fn from_entry(
        name: String,
        logo: Vec<String>,
        shape: Vec<String>,
        text: Vec<String>,
        tracking: Option<String>,
    ) -> Courier {
        let (tracking_re, tracking_full) = match tracking.as_deref() {
            Some(pattern) => (
                Some(Regex::new(pattern).expect("courier tracking pattern is valid")),
                Some(
                    Regex::new(&format!("^(?:{pattern})$"))
                        .expect("courier tracking pattern is valid"),
                ),
            ),
            None => (None, None),
        };
        Courier {
            name,
            logo,
            shape,
            text,
            tracking,
            tracking_re,
            tracking_full,
        }
    }
}

/// The courier data of `couriers.toml`: the couriers and the hashes answered *Not a courier*.
#[derive(Clone, Debug)]
pub struct CourierTable {
    pub couriers: Vec<Courier>,
    /// Hashes answered **Not a courier** — a buying channel's strip, Q-LABEL-5 — and so ignored
    /// wherever a logo is looked up. Written back by the logo prompt (D-LABEL-4).
    pub not_courier: Vec<String>,
}

impl CourierTable {
    /// Whether `hash` is a **byte** hash the data already knows: in a courier's `logo` list or in
    /// `not_courier`. `packing-pdf` computes the appearance hash only for a band image whose byte
    /// hash this does not know (courier.md item 3; Q-LABEL-1 option 1).
    pub fn knows_byte_hash(&self, hash: &str) -> bool {
        self.not_courier.iter().any(|known| known == hash)
            || self
                .couriers
                .iter()
                .any(|courier| courier.logo.iter().any(|known| known == hash))
    }

    /// The courier names in file order: the logo prompt's buttons (D-LABEL-1).
    pub fn names(&self) -> Vec<String> {
        self.couriers
            .iter()
            .map(|courier| courier.name.clone())
            .collect()
    }
}

/// The courier data built into the program, parsed once (07 › *Inputs*): the default of a data
/// folder without a `couriers.toml`.
pub fn courier_data() -> &'static CourierTable {
    static DATA: OnceLock<CourierTable> = OnceLock::new();
    DATA.get_or_init(|| load_couriers(COURIERS_TOML).expect("the built-in couriers.toml is valid"))
}

/// The couriers of the built-in data.
pub fn couriers() -> &'static [Courier] {
    &courier_data().couriers
}

/// The courier data of the data folder's file: its text, or the bundled file when `path` does not
/// exist (07 › *Inputs*, D-LABEL-1). A broken file is an error naming its line.
pub fn load_couriers_file(path: &Path) -> Result<CourierTable, CouriersError> {
    match std::fs::read_to_string(path) {
        Ok(text) => load_couriers(&text),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(courier_data().clone()),
        Err(error) => Err(CouriersError::new(format!(
            "cannot read {}: {error}",
            path.display()
        ))),
    }
}

// --------------------------------------------------------------- couriers file

const COURIERS_KEYS: &[&str] = &["not_courier", "courier"];
const COURIER_KEYS: &[&str] = &["name", "logo", "shape", "text", "tracking"];

fn is_hash(value: &str) -> bool {
    value.len() == 16
        && value
            .chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
}

/// A `[[courier]]` entry's string list (`logo`, `shape`, `text`), empty when the key is absent.
/// `place` names what the list belongs to (`courier "SiCepat": `, empty for a top-level list).
fn string_list(
    value: Option<&toml::Value>,
    place: &str,
    key: &str,
) -> Result<Vec<String>, CouriersError> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let toml::Value::Array(entries) = value else {
        return Err(CouriersError::new(format!(
            "{COURIERS_FILE_NAME}: {place}\"{key}\" must be an array of strings"
        )));
    };
    let mut list = Vec::with_capacity(entries.len());
    for entry in entries {
        let toml::Value::String(text) = entry else {
            return Err(CouriersError::new(format!(
                "{COURIERS_FILE_NAME}: {place}\"{key}\" must be an array of strings"
            )));
        };
        list.push(text.clone());
    }
    Ok(list)
}

/// A list of XXH3-64 hashes (16 lowercase hex characters), empty when the key is absent.
fn hash_list(
    value: Option<&toml::Value>,
    place: &str,
    key: &str,
) -> Result<Vec<String>, CouriersError> {
    let list = string_list(value, place, key)?;
    for hash in &list {
        if !is_hash(hash) {
            return Err(CouriersError::new(format!(
                "{COURIERS_FILE_NAME}: {place}\"{key}\" entry \"{hash}\" is not 16 hex characters \
                 (an XXH3-64 hash)"
            )));
        }
    }
    Ok(list)
}

/// One `[[courier]]` entry; errors name the courier by its `name`, else by its position.
fn courier_of(entry: &toml::Value, index: usize) -> Result<Courier, CouriersError> {
    let toml::Value::Table(entry) = entry else {
        return Err(CouriersError::new(format!(
            "{COURIERS_FILE_NAME}: courier #{}: must be a table",
            index + 1
        )));
    };
    let cid = match entry.get("name") {
        Some(toml::Value::String(name)) if !name.is_empty() => format!("\"{name}\""),
        _ => format!("#{}", index + 1),
    };
    for key in entry.keys() {
        if !COURIER_KEYS.contains(&key.as_str()) {
            return Err(CouriersError::new(format!(
                "{COURIERS_FILE_NAME}: courier {cid}: unknown key \"{key}\""
            )));
        }
    }

    let Some(name_value) = entry.get("name") else {
        return Err(CouriersError::new(format!(
            "{COURIERS_FILE_NAME}: courier {cid}: missing \"name\""
        )));
    };
    let toml::Value::String(name) = name_value else {
        return Err(CouriersError::new(format!(
            "{COURIERS_FILE_NAME}: courier {cid}: \"name\" must be a string"
        )));
    };
    if name.is_empty() {
        return Err(CouriersError::new(format!(
            "{COURIERS_FILE_NAME}: courier {cid}: \"name\" must not be empty"
        )));
    }

    let place = format!("courier {cid}: ");
    let logo = hash_list(entry.get("logo"), &place, "logo")?;
    let shape = hash_list(entry.get("shape"), &place, "shape")?;
    let text = string_list(entry.get("text"), &place, "text")?;
    let tracking = match entry.get("tracking") {
        None => None,
        Some(toml::Value::String(pattern)) => Some(pattern.clone()),
        Some(_) => {
            return Err(CouriersError::new(format!(
                "{COURIERS_FILE_NAME}: courier {cid}: \"tracking\" must be a string"
            )));
        }
    };
    if let Some(pattern) = &tracking
        && let Err(error) = Regex::new(pattern)
    {
        return Err(CouriersError::new(format!(
            "{COURIERS_FILE_NAME}: courier {cid}: \"tracking\" is not a valid pattern: {error}"
        )));
    }

    Ok(Courier::from_entry(
        name.clone(),
        logo,
        shape,
        text,
        tracking,
    ))
}

/// Load the couriers from the text of `couriers.toml` (the app reads the file). A TOML error and
/// every field error name the file and, where the TOML parser gives it, the line (07 › *Inputs*).
pub fn load_couriers(text: &str) -> Result<CourierTable, CouriersError> {
    let table: toml::Table = toml::from_str(text)
        .map_err(|error| CouriersError::new(format!("{COURIERS_FILE_NAME}: {error}")))?;

    for key in table.keys() {
        if !COURIERS_KEYS.contains(&key.as_str()) {
            return Err(CouriersError::new(format!(
                "{COURIERS_FILE_NAME}: unknown key \"{key}\""
            )));
        }
    }

    let not_courier = match table.get("not_courier") {
        None => Vec::new(),
        Some(value) => hash_list(Some(value), "", "not_courier")?,
    };

    let raw = match table.get("courier") {
        None => {
            return Err(CouriersError::new(format!(
                "{COURIERS_FILE_NAME}: no couriers"
            )));
        }
        Some(toml::Value::Array(entries)) => entries,
        Some(_) => {
            return Err(CouriersError::new(format!(
                "{COURIERS_FILE_NAME}: \"courier\" must be an array of tables"
            )));
        }
    };
    if raw.is_empty() {
        return Err(CouriersError::new(format!(
            "{COURIERS_FILE_NAME}: no couriers"
        )));
    }

    let mut couriers = Vec::with_capacity(raw.len());
    for (index, entry) in raw.iter().enumerate() {
        couriers.push(courier_of(entry, index)?);
    }
    Ok(CourierTable {
        couriers,
        not_courier,
    })
}

/// The candidate couriers of a tracking-ID form (courier.md item 2): every courier whose
/// `tracking` matches the whole of `tracking_id`.
pub fn candidates<'a>(table: &'a CourierTable, tracking_id: &TrackingId) -> Vec<&'a Courier> {
    table
        .couriers
        .iter()
        .filter(|courier| form_matches(courier, tracking_id))
        .collect()
}

/// Whether the tracking-ID form names `name` among its candidates (the two-clue rule's second
/// half; the order's `courier` is filled only where this holds — D-LABEL-2).
pub fn form_names_courier_in(table: &CourierTable, tracking_id: &TrackingId, name: &str) -> bool {
    candidates(table, tracking_id)
        .iter()
        .any(|courier| courier.name == name)
}

fn form_matches(courier: &Courier, tracking_id: &TrackingId) -> bool {
    !tracking_id.as_str().is_empty()
        && courier
            .tracking_full
            .as_ref()
            .is_some_and(|re| re.is_match(tracking_id.as_str()))
}

/// The courier one header-band image names: its byte hash in a courier's `logo`, else its
/// appearance hash in a courier's `shape`. `None` for a hash answered *Not a courier* (a buying
/// channel's strip) and for an image the data does not know.
pub fn logo_courier<'a>(table: &'a CourierTable, image: &LogoImage) -> Option<&'a Courier> {
    if is_not_courier(
        table,
        Some(image.hash.as_str()),
        image.appearance.as_deref(),
    ) {
        return None;
    }
    if let Some(courier) = table
        .couriers
        .iter()
        .find(|courier| courier.logo.iter().any(|known| known == &image.hash))
    {
        return Some(courier);
    }
    let appearance = image.appearance.as_deref()?;
    table
        .couriers
        .iter()
        .find(|courier| courier.shape.iter().any(|known| known == appearance))
}

/// Whether the image is a band image the prompt must ask the owner about (D-LABEL-4): a byte
/// hash not in `logo`/`not_courier` whose appearance is not in `shape` either.
pub fn is_unseen_logo(table: &CourierTable, image: &LogoImage) -> bool {
    if is_not_courier(
        table,
        Some(image.hash.as_str()),
        image.appearance.as_deref(),
    ) {
        return false;
    }
    if table
        .couriers
        .iter()
        .any(|courier| courier.logo.iter().any(|known| known == &image.hash))
    {
        return false;
    }
    !table.couriers.iter().any(|courier| {
        image
            .appearance
            .as_deref()
            .is_some_and(|appearance| courier.shape.iter().any(|known| known == appearance))
    })
}

fn is_not_courier(table: &CourierTable, hash: Option<&str>, appearance: Option<&str>) -> bool {
    table.not_courier.iter().any(|known| {
        hash.is_some_and(|hash| known == hash)
            || appearance.is_some_and(|appearance| known == appearance)
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

/// The two-clue courier rule (D-LABEL-2; courier.md items 1, 2, 4, 5): the tracking-ID form gives
/// the **candidate** couriers, and the courier is named only when the page's **logo** names one
/// of them (its byte hash, else its appearance hash). A logo whose courier and form disagree, a
/// logo with no form, a form with no logo, or neither → no name.
///
/// The printed text clue stands in for the logo only where the logo cannot be read at all — a
/// page with no header-band image (Q-LABEL-5) — and must be a candidate too. On such a page a
/// form that carries the vendor's **own marker** (a lettered prefix: `JY…`, `TKP…`, `SPXID…`)
/// also names the courier on its own, when it names only one: that is the old `tracking_unique`
/// behaviour derived from the table instead of a flag, and it keeps a label whose image is
/// missing altogether (the fpdf2 fixtures hold no images; a downloaded label always carries a
/// logo) readable. A **bare number** form (`00…`, `57…`) never names alone — the page is full of
/// numbers, so a number on its own is no clue, which is exactly what marking the `00` form
/// shared meant.
pub fn deduce_courier_in(
    table: &CourierTable,
    text: &str,
    tracking_id: Option<&TrackingId>,
    logos: &[LogoImage],
) -> Option<String> {
    let tracking_id = tracking_id?;
    let candidates = candidates(table, tracking_id);
    if candidates.is_empty() {
        return None;
    }

    if !logos.is_empty() {
        // Which courier the known band images name; two images naming different couriers cannot
        // be told apart, so nothing is named.
        let mut named: Option<&Courier> = None;
        for image in logos {
            let Some(courier) = logo_courier(table, image) else {
                continue;
            };
            match named {
                Some(other) if other.name != courier.name => return None,
                _ => named = Some(courier),
            }
        }
        return match named {
            Some(courier) if candidates.iter().any(|c| c.name == courier.name) => {
                Some(courier.name.clone())
            }
            _ => None,
        };
    }

    // No band image at all: the printed text clue stands in for the logo.
    let lowered = text.to_lowercase();
    let collapsed = whitespace_re().replace_all(text, "").to_lowercase();
    for courier in &candidates {
        if courier.text.iter().any(|clue| {
            let clue = clue.to_lowercase();
            lowered.contains(&clue) || collapsed.contains(&clue)
        }) {
            return Some(courier.name.clone());
        }
    }
    // A label whose image is missing altogether, and the form carries the vendor's own marker.
    if let [courier] = candidates.as_slice()
        && has_marker_prefix(courier)
    {
        return Some(courier.name.clone());
    }
    None
}

/// Whether the tracking form carries the vendor's own marker: its pattern begins with a letter
/// (`JY…`, `TKP…`, `SPXID…`) rather than a bare number (`00…`, `57…`).
fn has_marker_prefix(courier: &Courier) -> bool {
    courier
        .tracking
        .as_deref()
        .is_some_and(|pattern| pattern.starts_with(|c: char| c.is_ascii_alphabetic()))
}

/// The two-clue rule on the built-in `couriers.toml`.
pub fn deduce_courier(
    text: &str,
    tracking_id: Option<&TrackingId>,
    logos: &[LogoImage],
) -> Option<String> {
    deduce_courier_in(courier_data(), text, tracking_id, logos)
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
