//! The views (views.md): `orders.csv`, `order-contents.toml` and `session.toml`, written from the
//! fold after every commit so a person or another program can read a date without the app. Only
//! live batches appear. `session.toml` is written last: its `journal_seq` vouches for all three.
//! A view is never read for a decision.

mod orders_csv;
mod session;

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use packing_engine::ids::BatchId;
use packing_engine::record::{OrderContent, SCHEMA};
use serde::ser::{Serialize, SerializeMap, Serializer};
use serde::{Deserialize, Serialize as SerializeDerive};

use crate::StoreError;
use crate::files::write_atomic;
use crate::fold::Fold;

pub const SESSION_TOML: &str = "session.toml";
pub const ORDERS_CSV: &str = "orders.csv";
pub const ORDER_CONTENTS_TOML: &str = "order-contents.toml";

/// Write the three views of the fold, `session.toml` last.
pub fn write_views(date_dir: &Path, fold: &Fold) -> Result<(), StoreError> {
    let ids = ContentIds::of(fold);
    write_atomic(
        &date_dir.join(ORDERS_CSV),
        orders_csv::text(fold, &ids).as_bytes(),
    )?;
    write_atomic(
        &date_dir.join(ORDER_CONTENTS_TOML),
        contents_toml(&ids).as_bytes(),
    )?;
    write_atomic(
        &date_dir.join(SESSION_TOML),
        session::text(fold, &ids).as_bytes(),
    )
}

/// Whether the three views are there and `session.toml` says the journal's last `seq`.
pub fn views_current(date_dir: &Path, fold: &Fold) -> bool {
    #[derive(Deserialize)]
    struct Seq {
        journal_seq: u32,
    }
    let all_there = [ORDERS_CSV, ORDER_CONTENTS_TOML, SESSION_TOML]
        .iter()
        .all(|name| date_dir.join(name).is_file());
    all_there
        && fs::read_to_string(date_dir.join(SESSION_TOML))
            .ok()
            .and_then(|text| toml::from_str::<Seq>(&text).ok())
            .is_some_and(|seq| seq.journal_seq == fold.last_seq)
}

/// The `OrderContentID`s of a fold: numbered from 1 in order of first appearance over the live
/// batches, so the same journal always gives the same views.
pub(crate) struct ContentIds {
    contents: Vec<OrderContent>,
    by_batch: BTreeMap<(BatchId, u16), u16>,
}

impl ContentIds {
    fn of(fold: &Fold) -> Self {
        let mut ids = ContentIds {
            contents: Vec::new(),
            by_batch: BTreeMap::new(),
        };
        for (batch, live) in &fold.live {
            for (index, content) in live.saved.contents.iter().enumerate() {
                let id = match ids.contents.iter().position(|known| known == content) {
                    Some(at) => at as u16 + 1,
                    None => {
                        ids.contents.push(content.clone());
                        ids.contents.len() as u16
                    }
                };
                ids.by_batch.insert((*batch, index as u16), id);
            }
        }
        ids
    }

    /// The ID of a batch's content, by its index in the batch's `contents`.
    pub(crate) fn id(&self, batch: BatchId, content: u16) -> u16 {
        self.by_batch[&(batch, content)]
    }
}

fn contents_toml(ids: &ContentIds) -> String {
    #[derive(SerializeDerive)]
    struct Root<'a> {
        schema: &'static str,
        auto_increment: u16,
        order_content: Numbered<&'a OrderContent>,
    }
    let root = Root {
        schema: SCHEMA,
        auto_increment: ids.contents.len() as u16 + 1,
        order_content: Numbered(
            ids.contents
                .iter()
                .enumerate()
                .map(|(index, content)| (index as u16 + 1, content))
                .collect(),
        ),
    };
    toml::to_string(&root).expect("a view is plain TOML")
}

/// A map keyed by number, in the numbers' order (a TOML key is text, and `10` would sort before
/// `2`).
pub(crate) struct Numbered<T>(pub Vec<(u16, T)>);

impl<T: Serialize> Serialize for Numbered<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (key, value) in &self.0 {
            map.serialize_entry(&key.to_string(), value)?;
        }
        map.end()
    }
}
