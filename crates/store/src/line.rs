//! One line of the journal: the seal (XXH3-64 of the JSON bytes, 16 lower-case hex characters),
//! one space, the record as compact JSON, `\n`. The seal covers exactly the bytes between the
//! space and the `\n`.

use packing_engine::record::Record;
use xxhash_rust::xxh3::xxh3_64;

use crate::fold::{Damage, DamageReason};

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

/// The journal's bytes as records, with the torn tail an append left behind.
pub(crate) struct Parsed {
    /// Each record with its line number, counted from 1.
    pub records: Vec<(usize, Record)>,
    /// The length of the good part: the bytes up to the end of the last good line.
    pub good_len: u64,
    /// The bytes after the good part: a last line without `\n` or with a bad seal.
    pub torn: Option<Vec<u8>>,
}

/// Split the journal into its records. A bad last line is a torn write; a bad line before it, or
/// a sealed line that is not a record, is damage.
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
                let record = serde_json::from_slice(json).map_err(|_| Damage {
                    line: number,
                    reason: DamageReason::Unreadable,
                })?;
                records.push((number, record));
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

fn torn(records: Vec<(usize, Record)>, offset: usize, rest: &[u8]) -> Parsed {
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
