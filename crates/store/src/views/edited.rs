//! A view changed by hand: `day.toml` holds the SHA-256 of the two other views and, on its last
//! line, of its own text before it. A view whose hash differs is named, with the lines where it
//! differs from what the journal gives; in `orders.csv`, the cells, by their column.

use std::fs;
use std::path::Path;

use packing_engine::record::Sha256;
use serde::Deserialize;

use super::{DAY_TOML, ORDER_CONTENTS_TOML, ORDERS_CSV, orders_csv, texts};
use crate::files::{sha256_of, sha256_of_bytes};
use crate::fold::Fold;

const SEAL: &str = "# sha256: ";

/// The changes shown of a view; the rest are counted.
const SHOWN: usize = 5;

/// A view whose hash differs from the one `day.toml` states.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditedView {
    pub file: &'static str,
    /// The first lines or cells that differ from what the journal gives; none without a journal.
    pub lines: Vec<ChangedLine>,
    /// How many more differ.
    pub more: usize,
}

/// One line of a view, or one cell of a line of `orders.csv`, as the journal gives it and as the
/// file holds it; empty past a text's end.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChangedLine {
    /// From 1.
    pub number: usize,
    /// The cell's column header; none for a whole line.
    pub column: Option<String>,
    pub written: String,
    pub found: String,
}

/// `day.toml`'s text with its last line: the SHA-256 of the text before it.
pub(super) fn sealed(text: String) -> String {
    let hash = sha256_of_bytes(text.as_bytes());
    format!("{text}{SEAL}{hash}\n")
}

/// The views of the date that differ from their hashes in `day.toml`, each with the lines where it
/// differs from the views of `fold`. A `day.toml` with no hash (an older copy's) is not checked; a
/// view that is not there is not named (the date's open writes it again).
pub fn edited_views(date_dir: &Path, fold: Option<&Fold>) -> Vec<EditedView> {
    let Ok(day) = fs::read_to_string(date_dir.join(DAY_TOML)) else {
        return Vec::new();
    };
    let Some((body, stated)) = split_seal(&day) else {
        return Vec::new();
    };
    #[derive(Deserialize)]
    struct Hashes {
        orders_csv_sha256: Option<Sha256>,
        order_contents_sha256: Option<Sha256>,
    }
    let hashes = toml::from_str::<Hashes>(body).ok();
    let mut edited = Vec::new();
    if sha256_of_bytes(body.as_bytes()).as_str() != stated {
        edited.push(DAY_TOML);
    }
    let stated_of = |file| {
        let hashes = hashes.as_ref()?;
        match file {
            ORDERS_CSV => hashes.orders_csv_sha256.as_ref(),
            _ => hashes.order_contents_sha256.as_ref(),
        }
    };
    for file in [ORDERS_CSV, ORDER_CONTENTS_TOML] {
        let path = date_dir.join(file);
        let differs = stated_of(file).is_some_and(|stated| {
            path.is_file() && sha256_of(&path).is_ok_and(|found| &found != stated)
        });
        if differs {
            edited.push(file);
        }
    }
    let written = fold.map(texts);
    edited
        .into_iter()
        .map(|file| {
            let found = fs::read_to_string(date_dir.join(file)).unwrap_or_default();
            let journal = written
                .as_ref()
                .and_then(|texts| texts.iter().find(|(name, _)| *name == file))
                .map(|(_, text)| text.as_str());
            let changed = journal
                .map(|text| changed_lines(text, &found, file == ORDERS_CSV))
                .unwrap_or_default();
            EditedView {
                file,
                more: changed.len().saturating_sub(SHOWN),
                lines: changed.into_iter().take(SHOWN).collect(),
            }
        })
        .collect()
}

/// `day.toml`'s text before its last line, and the hash that line states; none when it has no
/// such line.
fn split_seal(day: &str) -> Option<(&str, &str)> {
    let without_end = day.strip_suffix('\n')?;
    let at = without_end.rfind('\n').map_or(0, |at| at + 1);
    let hash = without_end[at..].strip_prefix(SEAL)?;
    Some((&day[..at], hash))
}

/// The lines, by number, where two texts differ. In a CSV text, a row whose cells still line up
/// with the header gives its changed cells instead, each by its column.
fn changed_lines(written: &str, found: &str, csv: bool) -> Vec<ChangedLine> {
    let header = match csv {
        true => written.lines().next().map(orders_csv::cells),
        false => None,
    };
    let written: Vec<&str> = written.lines().collect();
    let found: Vec<&str> = found.lines().collect();
    let mut changed = Vec::new();
    for at in 0..written.len().max(found.len()) {
        let left = written.get(at).copied().unwrap_or_default();
        let right = found.get(at).copied().unwrap_or_default();
        if left == right {
            continue;
        }
        let number = at + 1;
        let (old, new) = (orders_csv::cells(left), orders_csv::cells(right));
        match &header {
            Some(header) if at > 0 && old.len() == header.len() && new.len() == header.len() => {
                for ((column, old), new) in header.iter().zip(old).zip(new) {
                    if old != new {
                        changed.push(ChangedLine {
                            number,
                            column: Some(column.clone()),
                            written: old,
                            found: new,
                        });
                    }
                }
            }
            _ => changed.push(ChangedLine {
                number,
                column: None,
                written: left.to_string(),
                found: right.to_string(),
            }),
        }
    }
    changed
}

#[cfg(test)]
mod tests {
    use super::{changed_lines, sealed, split_seal};

    #[test]
    fn a_sealed_text_splits_back_into_its_text_and_hash() {
        let text = sealed("schema = \"2.1\"\n".to_string());
        let (body, hash) = split_seal(&text).unwrap();
        assert_eq!(body, "schema = \"2.1\"\n");
        assert_eq!(hash.len(), 64);
        assert_eq!(split_seal("schema = \"2.1\"\n"), None);
        assert_eq!(split_seal(""), None);
    }

    fn shown(lines: &[super::ChangedLine]) -> Vec<(usize, Option<&str>, &str, &str)> {
        lines
            .iter()
            .map(|line| {
                let column = line.column.as_deref();
                (
                    line.number,
                    column,
                    line.written.as_str(),
                    line.found.as_str(),
                )
            })
            .collect()
    }

    #[test]
    fn changed_lines_are_numbered_from_one_and_run_past_the_shorter_text() {
        let lines = changed_lines("a\nb\nc\n", "a\nB\nc\nd\n", false);
        assert_eq!(shown(&lines), [(2, None, "b", "B"), (4, None, "", "d")]);
    }

    #[test]
    fn a_csv_row_gives_its_changed_cells_by_column_unless_they_no_longer_line_up() {
        let written = "order_id,tracking_id,shop\n1,JY0000001234,\"A, B\"\n2,JY0000001235,C\n";
        let found = "order_id,tracking_id,shop\n1,JY0000009999,\"A, B\"\n2,JY0000001235\n";
        let lines = changed_lines(written, found, true);
        assert_eq!(
            shown(&lines),
            [
                (2, Some("tracking_id"), "JY0000001234", "JY0000009999"),
                (3, None, "2,JY0000001235,C", "2,JY0000001235"),
            ]
        );
    }
}
