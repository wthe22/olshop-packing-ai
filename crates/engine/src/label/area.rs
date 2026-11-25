//! The receiver's area a label prints (reading.md, *The receiver's area*): province, city and
//! district as printed, found by the captions around them. A part the layout does not print is
//! none; nothing is guessed.

use crate::record::Area;
use crate::text::{PageText, Run};

/// The receiver's caption of every layout (`Penerima :`, `Penerima:` or a bare `Penerima`).
const RECEIVER: &str = "Penerima";
/// The instant label's address caption.
const ADDRESS: &str = "Alamat :";
/// The Shopee-style label's weight caption, under its city line.
const WEIGHT: &str = "Berat:";
const SENDER: &str = "Pengirim";
/// The Shopee-style label's note under its city line on a label the seller pays no postage for.
const CASHLESS: &str = "CASHLESS";

/// A run this close to a caption's x is in the caption's column, in points (a tag such as
/// `HOME` under the Shopee-style caption starts 3 pt right of it).
const SAME_X: f64 = 2.0;
/// Runs this close to one baseline are on one line.
const SAME_LINE: f64 = 1.5;
/// How far under the receiver's caption the area line may sit (the boxed layout prints the
/// receiver's name between them).
const AREA_UNDER: f64 = 25.0;
/// The instant label's address lines are 11.5 pt apart, the first as far under its caption.
const INSTANT_GAP: f64 = 14.0;
/// The Shopee-style address starts up to 26 pt under its caption (a tag may come between), its
/// lines 7.5 to 8.2 pt apart; the city line is 10.5 pt or more under the last.
const SHOPEE_FIRST_UNDER: f64 = 30.0;
const SHOPEE_GAP: f64 = 9.5;

/// The provinces, for the spaces of a name a line break cut. Compared without spaces, upper case.
const PROVINCES: [&str; 41] = [
    "ACEH",
    "SUMATERA UTARA",
    "SUMATERA BARAT",
    "RIAU",
    "KEPULAUAN RIAU",
    "JAMBI",
    "SUMATERA SELATAN",
    "KEPULAUAN BANGKA BELITUNG",
    "BANGKA BELITUNG",
    "BENGKULU",
    "LAMPUNG",
    "DKI JAKARTA",
    "BANTEN",
    "JAWA BARAT",
    "JAWA TENGAH",
    "DI YOGYAKARTA",
    "DAERAH ISTIMEWA YOGYAKARTA",
    "JAWA TIMUR",
    "BALI",
    "NUSA TENGGARA BARAT",
    "NUSA TENGGARA TIMUR",
    "KALIMANTAN BARAT",
    "KALIMANTAN TENGAH",
    "KALIMANTAN SELATAN",
    "KALIMANTAN TIMUR",
    "KALIMANTAN UTARA",
    "SULAWESI UTARA",
    "GORONTALO",
    "SULAWESI TENGAH",
    "SULAWESI BARAT",
    "SULAWESI SELATAN",
    "SULAWESI TENGGARA",
    "MALUKU",
    "MALUKU UTARA",
    "PAPUA",
    "PAPUA BARAT",
    "PAPUA BARAT DAYA",
    "PAPUA SELATAN",
    "PAPUA TENGAH",
    "PAPUA PEGUNUNGAN",
    "NANGGROE ACEH DARUSSALAM",
];

/// The receiver's area of a label's first page.
pub fn find_area(page: &PageText) -> Area {
    if let Some(address) = caption(page, ADDRESS) {
        return instant(page, address);
    }
    let Some(receiver) = caption(page, RECEIVER) else {
        return Area::default();
    };
    match caption(page, WEIGHT) {
        Some(weight) => shopee(page, receiver, weight),
        None => standard(page, receiver),
    }
}

/// The TikTok standard and the boxed layout: the first line under the caption that holds two
/// commas, `province,city,district` (the boxed layout goes on with the street).
fn standard(page: &PageText, receiver: &Run) -> Area {
    let line = column_under(page, receiver)
        .into_iter()
        .take_while(|run| receiver.y - run.y <= AREA_UNDER)
        .find(|run| run.text.matches(',').count() >= 2);
    let Some(line) = line else {
        return Area::default();
    };
    let mut parts = line.text.split(',').map(part);
    Area {
        province: parts.next().flatten(),
        city: parts.next().flatten(),
        district: parts.next().flatten(),
    }
}

/// The instant label: the address under its caption ends with `district, city, province`.
fn instant(page: &PageText, address: &Run) -> Area {
    let column = Column {
        caption: address,
        first_under: INSTANT_GAP,
        gap: INSTANT_GAP,
        right_end: f64::MAX,
    };
    let lines = column.block(page);
    let joined = join_lines(&lines);
    let mut parts = joined.rsplit(',').map(part);
    let province = parts.next().flatten().map(|name| respaced(&name));
    Area {
        province,
        city: parts.next().flatten(),
        district: parts.next().flatten(),
    }
}

/// The Shopee-style label: the province ends the address under the receiver's caption, left of
/// the sender's column; the city and the district are the first two runs, left to right, of the
/// line right above the weight's (or above its `CASHLESS` note). A street's hint may come between
/// the address and that line; the tracking ID printed upright at the page's edge is left of both
/// columns.
fn shopee(page: &PageText, receiver: &Run, weight: &Run) -> Area {
    let column = Column {
        caption: receiver,
        first_under: SHOPEE_FIRST_UNDER,
        gap: SHOPEE_GAP,
        right_end: caption(page, SENDER).map_or(f64::MAX, |sender| sender.x),
    };
    let lines = column.block(page);
    let Some(last) = lines.last() else {
        return Area::default();
    };
    let province = join_lines(&lines)
        .rsplit(',')
        .next()
        .and_then(part)
        .map(|name| respaced(&name));
    let mut between: Vec<&Run> = page
        .runs
        .iter()
        .filter(|run| {
            (receiver.x - SAME_X..column.right_end).contains(&run.x)
                && run.y < last.y - SAME_LINE
                && run.y > weight.y + SAME_LINE
        })
        .collect();
    between.sort_by(|a, b| a.y.total_cmp(&b.y));
    let city_line = between
        .iter()
        .map(|run| on_line(page, run.y))
        .find(|line| !line.iter().any(|run| run.text.starts_with(CASHLESS)))
        .unwrap_or_default();
    let mut city_line = city_line.into_iter().map(|run| part(&run.text));
    Area {
        province,
        city: city_line.next().flatten(),
        district: city_line.next().flatten(),
    }
}

/// The first run whose text starts with `text`.
fn caption<'a>(page: &'a PageText, text: &str) -> Option<&'a Run> {
    page.runs.iter().find(|run| run.text.starts_with(text))
}

/// The runs in the caption's column under it, top to bottom.
fn column_under<'a>(page: &'a PageText, caption: &Run) -> Vec<&'a Run> {
    let mut under: Vec<&Run> = page
        .runs
        .iter()
        .filter(|run| (run.x - caption.x).abs() <= SAME_X && run.y < caption.y - SAME_LINE)
        .collect();
    under.sort_by(|a, b| b.y.total_cmp(&a.y));
    under
}

/// The runs on the baseline `y`, left to right.
fn on_line(page: &PageText, y: f64) -> Vec<&Run> {
    let mut line: Vec<&Run> = page
        .runs
        .iter()
        .filter(|run| (run.y - y).abs() <= SAME_LINE)
        .collect();
    line.sort_by(|a, b| a.x.total_cmp(&b.x));
    line
}

/// `first` and the runs right of it on its baseline up to `right_end`.
fn line_from<'a>(page: &'a PageText, first: &'a Run, right_end: f64) -> Vec<&'a Run> {
    page.runs
        .iter()
        .filter(|run| (run.y - first.y).abs() <= SAME_LINE && run.x >= first.x && run.x < right_end)
        .collect()
}

/// A wrapped address under its caption.
struct Column<'a> {
    caption: &'a Run,
    /// How far under the caption its first line may start.
    first_under: f64,
    /// How far apart two of its lines may be.
    gap: f64,
    /// Where the column to its right starts.
    right_end: f64,
}

impl Column<'_> {
    /// Its lines, top to bottom, each line's runs joined with a space (a justified line is
    /// printed word by word).
    fn block(&self, page: &PageText) -> Vec<Line> {
        let mut lines: Vec<Line> = Vec::new();
        for run in column_under(page, self.caption) {
            let (above, limit) = match lines.last() {
                Some(line) => (line.y, self.gap),
                None => (self.caption.y, self.first_under),
            };
            if above - run.y > limit {
                break;
            }
            let mut runs = line_from(page, run, self.right_end);
            runs.sort_by(|a, b| a.x.total_cmp(&b.x));
            let words: Vec<&str> = runs.iter().map(|run| run.text.as_str()).collect();
            lines.push(Line {
                y: run.y,
                text: words.join(" "),
            });
        }
        lines
    }
}

struct Line {
    y: f64,
    text: String,
}

/// The lines of a wrapped address as one text. A label breaks a line inside a word too and drops
/// the space where it breaks between words, so a line that starts with a capital letter starts
/// a word.
fn join_lines(lines: &[Line]) -> String {
    let mut joined = String::new();
    for line in lines {
        let starts_word = line.text.chars().next().is_some_and(char::is_uppercase);
        if !joined.is_empty() && starts_word {
            joined.push(' ');
        }
        joined.push_str(&line.text);
    }
    joined
}

/// A comma-separated part, trimmed; none when empty.
fn part(text: &str) -> Option<String> {
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_string())
}

/// A province with the spaces of its listed name, its letters as printed: a line break may have
/// cut it (`JA` / `WA TENGAH`, `DKI` / `JAKARTA`). A name not listed stays as read.
fn respaced(name: &str) -> String {
    let squashed: String = name
        .chars()
        .filter(|c| !c.is_whitespace())
        .flat_map(char::to_uppercase)
        .collect();
    let Some(listed) = PROVINCES
        .iter()
        .find(|listed| listed.replace(' ', "") == squashed)
    else {
        return name.to_string();
    };
    let mut printed = name.chars().filter(|c| !c.is_whitespace());
    listed
        .chars()
        .map(|c| match c {
            ' ' => Some(' '),
            _ => printed.next(),
        })
        .collect::<Option<String>>()
        .unwrap_or_else(|| name.to_string())
}

#[cfg(test)]
mod tests;
