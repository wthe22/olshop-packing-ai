//! What the Day screen says about a date's journal (ui/messages.md, *Opening*): damage or a newer
//! format as a problem, and what it holds that this version does not know as a quiet note.

use packing_engine::record::JOURNAL_SCHEMA;
use packing_engine::schema::newer_app;
use packing_store::{Damage, DamageReason, Unknown, UnknownPart};

/// Damage, or a journal of a newer format.
pub(super) fn journal_problem(damage: Option<&Damage>) -> Option<String> {
    let damage = damage?;
    Some(match &damage.reason {
        DamageReason::Newer { schema, app } => format!(
            "This date was saved by a newer Packing (v{app}, journal schema {schema}); this copy \
             reads {}. Update this copy to work on it. Nothing of it is shown.",
            JOURNAL_SCHEMA.family()
        ),
        _ => format!(
            "The journal of this date is damaged at {damage}. The date can be viewed; nothing is \
             written to it."
        ),
    })
}

/// What the journal holds that this version (`ours`) does not know, all of it kept: a newer copy's
/// in one note; anything else once per record type and field, with its lines.
pub(super) fn journal_notes(unknown: &[Unknown], ours: &str) -> Vec<String> {
    let (newer, same): (Vec<&Unknown>, Vec<&Unknown>) = unknown
        .iter()
        .partition(|found| newer_app(&found.app, ours));
    let mut notes = Vec::new();
    // The last one written: app versions do not sort as text (0.10 after 0.9).
    if let Some(last) = newer.last() {
        notes.push(format!(
            "This date holds records of a newer Packing (v{}) that this copy does not know; they \
             are kept. Update this copy to see all of it.",
            last.app
        ));
    }
    let mut seen: Vec<(String, Vec<u32>)> = Vec::new();
    for found in same {
        let what = match &found.part {
            UnknownPart::Record => format!("a record of type {}", found.kind),
            UnknownPart::Field(path) => format!("{} in {}", any_index(path), found.kind),
        };
        match seen.iter_mut().find(|(known, _)| *known == what) {
            Some((_, lines)) if lines.last() == Some(&found.seq) => {}
            Some((_, lines)) => lines.push(found.seq),
            None => seen.push((what, vec![found.seq])),
        }
    }
    notes.extend(seen.into_iter().map(|(what, lines)| {
        let lines: Vec<String> = lines.iter().map(u32::to_string).collect();
        let word = if lines.len() == 1 { "line" } else { "lines" };
        format!(
            "Journal {word} {}: {what} is not known to this version. It is kept and not used.",
            lines.join(", ")
        )
    }));
    notes
}

/// A field's path with its positions left out, so every order's copy of it is one: `orders[].colour`.
fn any_index(path: &str) -> String {
    let mut out = String::new();
    let mut in_index = false;
    for c in path.chars() {
        match c {
            '[' => {
                in_index = true;
                out.push_str("[]");
            }
            ']' => in_index = false,
            _ if in_index => {}
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unknown(seq: u32, app: &str, part: UnknownPart) -> Unknown {
        Unknown {
            seq,
            app: app.to_string(),
            kind: if part == UnknownPart::Record {
                "orders_moved"
            } else {
                "batch_saved"
            }
            .to_string(),
            part,
        }
    }

    fn field(path: &str) -> UnknownPart {
        UnknownPart::Field(path.to_string())
    }

    #[test]
    fn the_journal_names_damage_or_a_newer_format() {
        let newer = Damage {
            line: 1,
            reason: DamageReason::Newer {
                schema: "3.0".to_string(),
                app: "0.9.0".to_string(),
            },
        };
        let bad = Damage {
            line: 3,
            reason: DamageReason::Unreadable,
        };
        let text = |damage| journal_problem(Some(damage)).unwrap();
        assert!(
            text(&newer)
                .contains("newer Packing (v0.9.0, journal schema 3.0); this copy reads 2.x")
        );
        assert!(text(&bad).contains("damaged at line 3: it is not a record"));
        assert_eq!(journal_problem(None), None);
    }

    #[test]
    fn what_this_version_does_not_know_is_named_once_with_its_lines() {
        let found = [
            unknown(2, "0.8.7-alpha", field("orders[0].colour")),
            unknown(2, "0.8.7-alpha", field("orders[1].colour")),
            unknown(3, "0.9.1", UnknownPart::Record),
            unknown(4, "0.8.5-alpha", UnknownPart::Record),
            unknown(5, "0.8.7-alpha", field("orders[0].colour")),
            unknown(6, "0.9.0", field("note")),
        ];
        assert_eq!(
            journal_notes(&found, "0.8.7-alpha"),
            [
                "This date holds records of a newer Packing (v0.9.0) that this copy does not know; \
                 they are kept. Update this copy to see all of it.",
                "Journal lines 2, 5: orders[].colour in batch_saved is not known to this \
                 version. It is kept and not used.",
                "Journal line 4: a record of type orders_moved is not known to this version. It is kept \
                 and not used.",
            ]
        );
        assert!(journal_notes(&[], "0.8.7-alpha").is_empty());
    }
}
