//! The views (views.md): `orders.csv`, `order-contents.toml` and `day.toml`, written from the
//! fold after every commit so a person or another program can read a date without the app. Only
//! live batches appear. `day.toml` is written last: its `journal_seq` vouches for all three.
//! A view is never read for a decision.

mod day;
mod orders_csv;

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use packing_engine::ids::BatchId;
use packing_engine::record::{OrderContent, SCHEMA};
use packing_engine::schema::{Schema, newer_app};
use serde::ser::{Serialize, SerializeMap, Serializer};
use serde::{Deserialize, Serialize as SerializeDerive};

use crate::StoreError;
use crate::files::write_atomic;
use crate::fold::Fold;

pub const DAY_TOML: &str = "day.toml";
pub const ORDERS_CSV: &str = "orders.csv";
pub const ORDER_CONTENTS_TOML: &str = "order-contents.toml";
/// `day.toml` under its older name, which the migration replaces.
pub const SESSION_TOML: &str = "session.toml";

/// The schema version of `day.toml` and of `orders.csv` (its last column's header). A file of an
/// older version is written again only by the migration the owner starts, never when a date opens.
pub const VIEWS_SCHEMA: Schema = Schema::new(2, 1);

/// Write the three views of the fold, `day.toml` last. A copy writes none it could not write
/// whole: none of a date with a newer copy's record, none over a newer copy's `day.toml`.
pub fn write_views(date_dir: &Path, fold: &Fold) -> Result<(), StoreError> {
    if left_to_a_newer_copy(date_dir, fold) {
        return Ok(());
    }
    let ids = ContentIds::of(fold);
    write_atomic(
        &date_dir.join(ORDERS_CSV),
        orders_csv::text(fold, &ids).as_bytes(),
    )?;
    write_atomic(
        &date_dir.join(ORDER_CONTENTS_TOML),
        contents_toml(&ids).as_bytes(),
    )?;
    write_atomic(&date_dir.join(DAY_TOML), day::text(fold, &ids).as_bytes())
}

/// The views are a newer copy's to write: the journal holds a record or a field a newer copy wrote
/// and this one does not know, or `day.toml` is of a newer version than this copy writes.
fn left_to_a_newer_copy(date_dir: &Path, fold: &Fold) -> bool {
    fold.unknown
        .iter()
        .any(|unknown| newer_app(&unknown.app, env!("CARGO_PKG_VERSION")))
        || views_schema(date_dir, DAY_TOML).is_some_and(|found| {
            Schema::parse(&found).is_some_and(|version| version > VIEWS_SCHEMA)
        })
}

/// Whether the three views are there and `day.toml` says the journal's last `seq`, or they are a
/// newer copy's to write. Its `schema` is not looked at otherwise: a view of an older version waits
/// for the owner's migration.
pub fn views_current(date_dir: &Path, fold: &Fold) -> bool {
    if left_to_a_newer_copy(date_dir, fold) {
        return true;
    }
    #[derive(Deserialize)]
    struct Seq {
        journal_seq: u32,
    }
    let all_there = [ORDERS_CSV, ORDER_CONTENTS_TOML, DAY_TOML]
        .iter()
        .all(|name| date_dir.join(name).is_file());
    all_there
        && fs::read_to_string(date_dir.join(DAY_TOML))
            .ok()
            .and_then(|text| toml::from_str::<Seq>(&text).ok())
            .is_some_and(|seq| seq.journal_seq == fold.last_seq)
}

/// The `schema` the migration finds in a date's views, when it writes them again: `session.toml`'s,
/// whatever it is, or that of a `day.toml` older than this copy writes.
pub(crate) fn older_views(date_dir: &Path) -> Option<String> {
    if let Some(found) = views_schema(date_dir, SESSION_TOML) {
        return Some(found);
    }
    views_schema(date_dir, DAY_TOML)
        .filter(|found| Schema::parse(found).is_none_or(|version| version < VIEWS_SCHEMA))
}

/// The views written again by the migration: `session.toml` goes once `day.toml` is there.
pub fn write_views_again(date_dir: &Path, fold: &Fold) -> Result<(), StoreError> {
    write_views(date_dir, fold)?;
    let old = date_dir.join(SESSION_TOML);
    if date_dir.join(DAY_TOML).is_file() && old.is_file() {
        fs::remove_file(&old).map_err(|error| StoreError::io(&old, error))?;
    }
    Ok(())
}

/// The `schema` of the date's `file` (`day.toml` or `session.toml`); none when there is no such
/// file or it does not read.
pub(crate) fn views_schema(date_dir: &Path, file: &str) -> Option<String> {
    #[derive(Deserialize)]
    struct Found {
        schema: String,
    }
    fs::read_to_string(date_dir.join(file))
        .ok()
        .and_then(|text| toml::from_str::<Found>(&text).ok())
        .map(|found| found.schema)
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
        schema: Schema,
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
