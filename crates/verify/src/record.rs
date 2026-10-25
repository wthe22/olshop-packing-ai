//! The journal's lines read as the verifier needs them, into types of its own: a field the writer
//! got wrong is read as written, not as meant.

use serde::Deserialize;
use xxhash_rust::xxh3::xxh3_64;

/// The seq and type every line carries.
#[derive(Deserialize)]
pub(crate) struct Head {
    pub seq: u32,
    #[serde(rename = "type")]
    pub kind: String,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Batch {
    pub batch: u8,
    pub amends: bool,
    pub inputs: Vec<Input>,
    pub contents: Vec<Content>,
    pub orders: Vec<Order>,
    pub left_out: Vec<LeftOut>,
    pub outputs: Vec<Output>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Input {
    pub pdf: u8,
    pub sha256: String,
    pub filename: String,
    pub page_count: u16,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Content {
    pub items: Vec<Item>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Item {
    pub quantity: u16,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Order {
    pub order_id: String,
    pub content: u16,
    pub pdf: u8,
    pub page_start: u16,
    pub page_end: u16,
    pub group: String,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct LeftOut {
    pub order_id: String,
    pub pdf: u8,
    pub page_start: u16,
    pub page_end: u16,
    pub reason: String,
    pub kept: Kept,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum Kept {
    Copy { pdf: u8, page_start: u16 },
    Batch { batch: u8 },
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Output {
    pub file: String,
    pub group: Option<String>,
    pub pages: u16,
    pub size: u64,
    pub sha256: String,
}

#[derive(Deserialize)]
pub(crate) struct Reverted {
    pub batch: u8,
}

/// One line split into its JSON, when its seal is the XXH3-64 of that JSON.
pub(crate) fn sealed(line: &[u8]) -> Option<&[u8]> {
    if line.len() < 18 || line[16] != b' ' {
        return None;
    }
    let (seal, json) = (&line[..16], &line[17..]);
    (seal == format!("{:016x}", xxh3_64(json)).as_bytes()).then_some(json)
}

/// The journal's lines: each complete line with its number from 1, and whether the last one is
/// a torn write (no line break, or a seal that does not match), which the commit cuts.
pub(crate) fn lines(bytes: &[u8]) -> Vec<(usize, &[u8], bool)> {
    let mut out = Vec::new();
    let mut rest = bytes;
    let mut number = 0;
    while !rest.is_empty() {
        number += 1;
        match rest.iter().position(|&b| b == b'\n') {
            Some(end) => {
                out.push((number, &rest[..end], true));
                rest = &rest[end + 1..];
            }
            None => {
                out.push((number, rest, false));
                rest = &[];
            }
        }
    }
    out
}
