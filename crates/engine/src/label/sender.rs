//! The sender a label prints with its `Pengirim` caption (reading.md, *The sender*). It is read only
//! to check that a label is one of the workspace's shops; the record does not keep it.

use crate::text::{PageText, Run};

const CAPTION: &str = "Pengirim";
/// A run this close to the caption's baseline is on its line, in points.
const SAME_LINE: f64 = 3.0;
/// How far under the caption's baseline the name may sit, and how far from the caption's x.
const UNDER: f64 = 12.0;
const UNDER_X: f64 = 20.0;

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

/// The sender text of a page; none when the page has no caption.
pub fn find_sender(page: &PageText) -> Option<SenderText> {
    let line = page.text.lines().find(|line| line.contains(CAPTION))?;
    let mut text = line.to_string();
    if let Some(caption) = page.runs.iter().find(|run| run.text.contains(CAPTION)) {
        for run in page.runs.iter().filter(|run| beside_or_under(caption, run)) {
            text.push_str(&run.text);
        }
    }
    Some(SenderText(squash(&text)))
}

/// `y` grows upwards, so a run under the caption has a lower baseline.
fn beside_or_under(caption: &Run, run: &Run) -> bool {
    let rise = run.y - caption.y;
    let beside = rise.abs() <= SAME_LINE && run.x > caption.x;
    let under = (-UNDER..-SAME_LINE).contains(&rise) && (run.x - caption.x).abs() <= UNDER_X;
    beside || under
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
            assert_eq!(sender.shows("toko  CONTOH"), shows, "{case}");
        }
    }

    #[test]
    fn a_page_with_no_caption_has_no_sender_and_an_empty_name_matches_nothing() {
        assert_eq!(
            find_sender(&page("Penerima : Toko Contoh", Vec::new())),
            None
        );
        let sender = find_sender(&page("Pengirim : Toko Contoh", Vec::new())).unwrap();
        assert!(!sender.shows(" "));
    }
}
