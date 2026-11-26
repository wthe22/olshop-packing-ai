//! One line of the journal: the seal (XXH3-64 of the JSON bytes, 16 lower-case hex characters),
//! one space, the record as compact JSON, `\n`. The seal covers exactly the bytes between the
//! space and the `\n`.

use packing_engine::record::{Record, RecordBody};
use serde::Deserialize;
use xxhash_rust::xxh3::xxh3_64;

use crate::fold::{Damage, DamageReason, PassedOver, check_journal_schema};

pub fn seal(json: &[u8]) -> String {
    format!("{:016x}", xxh3_64(json))
}

pub fn encode_line(record: &Record) -> Vec<u8> {
    let json = serde_json::to_vec(record).expect("a record is plain JSON");
    let mut line = seal(&json).into_bytes();
    line.push(b' ');
    line.extend_from_slice(&json);
    line.push(b'\n');
    line
}

/// A sealed line: a record this copy reads, or a newer copy's it passes over.
pub(crate) enum Entry {
    Record(Box<Record>),
    PassedOver(PassedOver),
}

/// The journal's bytes as records, with the torn tail an append left behind.
pub(crate) struct Parsed {
    /// Each record with its line number, counted from 1.
    pub records: Vec<(usize, Entry)>,
    /// The length of the good part: the bytes up to the end of the last good line.
    pub good_len: u64,
    /// The bytes after the good part: a last line without `\n` or with a bad seal.
    pub torn: Option<Vec<u8>>,
}

/// Split the journal into its records. A bad last line is a torn write; a bad line before it, or
/// a sealed line that is not a record, is damage. A journal of another major schema stops at its
/// first line, before a line of a newer shape is read as damage.
pub(crate) fn parse(bytes: &[u8]) -> Result<Parsed, Damage> {
    let mut records = Vec::new();
    let mut offset = 0usize;
    let mut number = 0usize;
    while offset < bytes.len() {
        number += 1;
        let rest = &bytes[offset..];
        let Some(end) = rest.iter().position(|&b| b == b'\n') else {
            return Ok(torn(records, offset, rest));
        };
        let line = &rest[..end];
        let is_last = offset + end + 1 == bytes.len();
        match sealed_json(line) {
            Some(json) => {
                let damage = |reason| Damage {
                    line: number,
                    reason,
                };
                if number == 1 {
                    first_schema(json).map_err(damage)?;
                }
                let entry = entry(json).ok_or(damage(DamageReason::Unreadable))?;
                records.push((number, entry));
            }
            None if is_last => return Ok(torn(records, offset, rest)),
            None => {
                return Err(Damage {
                    line: number,
                    reason: DamageReason::BadSeal,
                });
            }
        }
        offset += end + 1;
    }
    Ok(Parsed {
        records,
        good_len: offset as u64,
        torn: None,
    })
}

/// A record, or a line of a type this copy does not know; none when it is neither.
fn entry(json: &[u8]) -> Option<Entry> {
    if let Ok(record) = serde_json::from_slice::<Record>(json) {
        return Some(Entry::Record(Box::new(record)));
    }
    let head: PassedOver = serde_json::from_slice(json).ok()?;
    (!RecordBody::KINDS.contains(&head.kind.as_str())).then_some(Entry::PassedOver(head))
}

/// The first line's `schema` is one this copy reads; a line without one is left to the fold.
fn first_schema(json: &[u8]) -> Result<(), DamageReason> {
    #[derive(Deserialize)]
    struct First {
        schema: String,
        #[serde(default)]
        app: String,
    }
    match serde_json::from_slice::<First>(json) {
        Ok(first) => check_journal_schema(&first.schema, &first.app),
        Err(_) => Ok(()),
    }
}

fn torn(records: Vec<(usize, Entry)>, offset: usize, rest: &[u8]) -> Parsed {
    Parsed {
        records,
        good_len: offset as u64,
        torn: Some(rest.to_vec()),
    }
}

/// The JSON of a line whose seal matches it.
fn sealed_json(line: &[u8]) -> Option<&[u8]> {
    if line.len() < 18 || line[16] != b' ' {
        return None;
    }
    let (seal_bytes, json) = (&line[..16], &line[17..]);
    (seal_bytes == seal(json).as_bytes()).then_some(json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_seal_is_the_xxh3_of_the_json_in_16_hex_characters() {
        assert_eq!(seal(b""), "2d06800538d394c2");
        assert_eq!(seal(b"{}").len(), 16);
    }

    #[test]
    fn a_line_is_good_only_when_its_seal_matches_its_json() {
        let json = br#"{"seq":1}"#;
        let good = [seal(json).as_bytes(), b" ", json].concat();
        assert_eq!(sealed_json(&good), Some(&json[..]));
        let mut changed = good.clone();
        *changed.last_mut().unwrap() = b']';
        let cases: [&[u8]; 4] = [&changed, b"", b"2d06800538d394c2", b"2d06800538d394c2x{}"];
        for line in cases {
            assert_eq!(
                sealed_json(line),
                None,
                "{:?}",
                String::from_utf8_lossy(line)
            );
        }
    }
}
