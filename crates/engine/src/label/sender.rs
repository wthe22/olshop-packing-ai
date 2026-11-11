//! The sender a label prints with its `Pengirim` caption (reading.md, *The sender*): the text an
//! account's shop names are looked for in, and the name read, which the shop prompt shows for a
//! sender no account lists. The record keeps the shop name found, not this text.

use crate::text::{PageText, Run};

const CAPTION: &str = "Pengirim";
/// A run this close to the caption's baseline is on its line, in points.
const SAME_LINE: f64 = 3.0;
/// How far under the caption's baseline the name may sit, and how far from the caption's x.
const UNDER: f64 = 12.0;
const UNDER_X: f64 = 20.0;
/// A run of one word with this many digits is a phone or tracking number, which ends the name.
const NUMBER_DIGITS: usize = 6;

/// The sender of a label: the text around its caption and the name read from it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sender {
    pub text: SenderText,
    /// As printed, cut at a phone number or the next caption; its spaces may be wrong.
    pub name: String,
}

/// The text around a page's `Pengirim` caption, its spaces taken out and in lower case: the
/// caption's line of the page text and the runs beside and under the caption. The page text's
/// spaces are not reliable and a digit of the vertical tracking ID may stick to the name, so a name
/// is looked for inside this text, never compared with it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SenderText(String);

impl SenderText {
    /// Whether the label shows `name`, spaces and upper and lower case ignored.
    pub fn shows(&self, name: &str) -> bool {
        let name = squash(name);
        !name.is_empty() && self.0.contains(&name)
    }
}

/// The sender of a page; none when the page has no caption.
pub fn find_sender(page: &PageText) -> Option<Sender> {
    let line = page.text.lines().find(|line| line.contains(CAPTION))?;
    let mut text = line.to_string();
    let mut name = String::new();
    if let Some(caption) = page.runs.iter().find(|run| run.text.contains(CAPTION)) {
        for run in page.runs.iter().filter(|run| beside_or_under(caption, run)) {
            text.push_str(&run.text);
        }
        name = name_beside(caption, &page.runs);
        if name.is_empty() {
            name = name_under(caption, &page.runs);
        }
    }
    if name.is_empty() {
        name = cut(after_caption(line));
    }
    Some(Sender {
        text: SenderText(squash(&text)),
        name,
    })
}

/// `y` grows upwards, so a run under the caption has a lower baseline.
fn beside_or_under(caption: &Run, run: &Run) -> bool {
    beside(caption, run) || under(caption, run)
}

fn beside(caption: &Run, run: &Run) -> bool {
    (run.y - caption.y).abs() <= SAME_LINE && run.x > caption.x
}

fn under(caption: &Run, run: &Run) -> bool {
    (-UNDER..-SAME_LINE).contains(&(run.y - caption.y)) && (run.x - caption.x).abs() <= UNDER_X
}

/// The caption's own run after the caption, then the runs right of it on its baseline up to a
/// number's run (a phone number, or the tracking number the Shopee-style label prints there).
fn name_beside(caption: &Run, runs: &[Run]) -> String {
    let mut beside: Vec<&Run> = runs.iter().filter(|run| beside(caption, run)).collect();
    beside.sort_by(|a, b| a.x.total_cmp(&b.x));
    let mut raw = after_caption(&caption.text).to_string();
    for run in beside.into_iter().take_while(|run| !number_run(&run.text)) {
        raw.push(' ');
        raw.push_str(&run.text);
    }
    cut(&raw)
}

/// The runs under the caption, top to bottom.
fn name_under(caption: &Run, runs: &[Run]) -> String {
    let mut under: Vec<&Run> = runs.iter().filter(|run| under(caption, run)).collect();
    under.sort_by(|a, b| b.y.total_cmp(&a.y).then(a.x.total_cmp(&b.x)));
    let raw: Vec<&str> = under.iter().map(|run| run.text.as_str()).collect();
    cut(&raw.join(" "))
}

fn number_run(text: &str) -> bool {
    let text = text.trim();
    !text.contains(char::is_whitespace)
        && text.chars().filter(char::is_ascii_digit).count() >= NUMBER_DIGITS
}

fn after_caption(text: &str) -> &str {
    text.find(CAPTION)
        .map_or("", |at| &text[at + CAPTION.len()..])
}

/// A name taken up to a phone number (`(+62)`, `+62`, digits with `*`) or the next caption (a word
/// and `:`), with a digit stuck to its end taken off.
fn cut(raw: &str) -> String {
    let text = raw.trim_start_matches(|c: char| c == ':' || c.is_whitespace());
    let mut end = text.len();
    if let Some(at) = text.find("+62") {
        end = end.min(text[..at].strip_suffix('(').map_or(at, str::len));
    }
    if let Some(star) = text.find('*') {
        end = end.min(
            text[..star]
                .trim_end_matches(|c: char| c.is_ascii_digit())
                .len(),
        );
    }
    if let Some(colon) = text.find(':') {
        let word = text[..colon].trim_end();
        end = end.min(word.rfind(char::is_whitespace).map_or(0, |space| space + 1));
    }
    let mut name = text[..end].trim().to_string();
    let mut last = name.chars().rev();
    if let (Some(digit), Some(before)) = (last.next(), last.next())
        && digit.is_ascii_digit()
        && before.is_alphabetic()
    {
        name.pop();
    }
    name
}

fn squash(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::find_sender;
    use crate::text::{PageText, Run};

    fn run(x: f64, y: f64, text: &str) -> Run {
        Run {
            x,
            y,
            text: text.to_string(),
        }
    }

    fn page(text: &str, runs: Vec<Run>) -> PageText {
        PageText {
            text: text.to_string(),
            runs,
            ..PageText::default()
        }
    }

    #[test]
    fn the_sender_is_found_in_each_place_a_label_prints_it() {
        let cases = [
            (
                "right of the caption, a digit stuck to it",
                page(
                    "Pengirim :Tok o Cont oh\nPenerima :",
                    vec![
                        run(10.0, 300.0, "Pengirim :"),
                        run(45.0, 299.5, "TokoContoh7"),
                    ],
                ),
                true,
            ),
            (
                "in the caption's run",
                page(
                    "Pengirim : Toko Contoh",
                    vec![run(10.0, 300.0, "Pengirim : Toko Contoh")],
                ),
                true,
            ),
            (
                "under the caption",
                page(
                    "Toko Contoh Pengirim",
                    vec![
                        run(10.0, 300.0, "Pengirim"),
                        run(10.0, 289.2, "Toko Contoh"),
                    ],
                ),
                true,
            ),
            (
                "only in the page text's line",
                page(
                    "Pengirim: TokoContoh (+62)8**** Nomor Order:",
                    vec![
                        run(120.0, 300.0, "(+62)8****Pengirim:"),
                        run(9.0, 299.7, "TokoContoh"),
                    ],
                ),
                true,
            ),
            (
                "a run of its own on the Shopee-style label",
                page(
                    "Pengirim:\nNo.Pesanan: 2600000AAAAAAA",
                    vec![
                        run(60.0, 300.0, "Penerima:"),
                        run(214.0, 300.0, "Pengirim:"),
                        run(249.0, 300.0, "Toko Contoh"),
                    ],
                ),
                true,
            ),
            (
                "another shop",
                page(
                    "Pengirim : Toko Lain",
                    vec![run(10.0, 300.0, "Pengirim : Toko Lain")],
                ),
                false,
            ),
            (
                "the name far under the caption",
                page(
                    "Pengirim :",
                    vec![
                        run(10.0, 300.0, "Pengirim :"),
                        run(10.0, 250.0, "Toko Contoh"),
                    ],
                ),
                false,
            ),
        ];
        for (case, page, shows) in cases {
            let sender = find_sender(&page).expect(case);
            assert_eq!(sender.text.shows("toko  CONTOH"), shows, "{case}");
        }
    }

    #[test]
    fn the_name_is_read_up_to_a_phone_number_a_tracking_number_or_the_next_caption() {
        let cases = [
            (
                page(
                    "Pengirim :Tok o Cont oh\nPenerima :",
                    vec![
                        run(10.0, 300.0, "Pengirim :"),
                        run(45.0, 299.5, "TokoContoh7"),
                    ],
                ),
                "TokoContoh",
            ),
            (
                page(
                    "Pengirim : Toko Contoh 99 Penerima : Budi",
                    vec![run(
                        10.0,
                        300.0,
                        "Pengirim : Toko Contoh 99 Penerima : Budi",
                    )],
                ),
                "Toko Contoh 99",
            ),
            (
                page(
                    "Toko Contoh Pengirim",
                    vec![
                        run(10.0, 300.0, "Pengirim"),
                        run(10.0, 289.2, "Toko Contoh"),
                    ],
                ),
                "Toko Contoh",
            ),
            (
                page(
                    "Pengirim: Toko Contoh (+62)8**** Nomor Order:",
                    vec![
                        run(120.0, 300.0, "(+62)8****Pengirim:"),
                        run(9.0, 299.7, "Toko Contoh"),
                    ],
                ),
                "Toko Contoh",
            ),
            (
                page(
                    "Pengirim : Toko Contoh +6281****",
                    vec![run(10.0, 300.0, "Pengirim : Toko Contoh 0812****")],
                ),
                "Toko Contoh",
            ),
            (
                page(
                    "Pengirim:\nSPXID00000000001A",
                    vec![
                        run(166.8, 315.8, "Pengirim:"),
                        run(288.0, 317.3, "SPXID00000000001A"),
                        run(202.2, 315.8, "Toko Contoh"),
                    ],
                ),
                "Toko Contoh",
            ),
            (page("Pengirim :", vec![run(10.0, 300.0, "Pengirim :")]), ""),
        ];
        for (page, name) in cases {
            assert_eq!(find_sender(&page).unwrap().name, name, "{}", page.text);
        }
    }

    #[test]
    fn a_page_with_no_caption_has_no_sender_and_an_empty_name_matches_nothing() {
        assert_eq!(
            find_sender(&page("Penerima : Toko Contoh", Vec::new())),
            None
        );
        let sender = find_sender(&page("Pengirim : Toko Contoh", Vec::new())).unwrap();
        assert!(!sender.text.shows(" "));
    }
}
