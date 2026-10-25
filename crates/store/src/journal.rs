//! A date's `journal.jsonl`: read and folded at open, and the commit — one append of sealed lines
//! and `sync_all`. Before the append returns nothing is saved; after it, everything is.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use packing_engine::record::{Datetime, Record, RecordBody, Recovered, SCHEMA, Started};

use crate::StoreError;
use crate::date::DateFolder;
use crate::fold::{Damage, Fold};
use crate::line::{encode_line, parse};

pub const JOURNAL: &str = "journal.jsonl";

pub struct Journal {
    path: PathBuf,
    fold: Fold,
    /// The length of the journal up to the end of its last good line.
    good_len: u64,
    /// The bytes of a torn last line, cut and recorded by the next commit.
    torn: Option<Vec<u8>>,
    damage: Option<Damage>,
}

impl Journal {
    /// Read and fold the date's journal; a date without one is empty. Damage is kept, not
    /// returned: the date can still be viewed by the fold of the lines before it.
    pub fn open(folder: &DateFolder) -> Result<Self, StoreError> {
        let path = folder.dir().join(JOURNAL);
        let mut journal = Journal {
            fold: Fold::new(folder.namespace(), folder.date()),
            path,
            good_len: 0,
            torn: None,
            damage: None,
        };
        let bytes = match fs::read(&journal.path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(journal),
            Err(error) => return Err(StoreError::io(&journal.path, error)),
        };
        let parsed = match parse(&bytes) {
            Ok(parsed) => parsed,
            Err(damage) => {
                journal.damage = Some(damage);
                return Ok(journal);
            }
        };
        for (line, record) in &parsed.records {
            if let Err(reason) = journal.fold.apply(record) {
                journal.damage = Some(Damage {
                    line: *line,
                    reason,
                });
                return Ok(journal);
            }
        }
        journal.good_len = parsed.good_len;
        journal.torn = parsed.torn;
        Ok(journal)
    }

    pub fn fold(&self) -> &Fold {
        &self.fold
    }

    pub fn damage(&self) -> Option<&Damage> {
        self.damage.as_ref()
    }

    /// The bytes of a torn last line, if the journal ends in one.
    pub fn torn(&self) -> Option<&[u8]> {
        self.torn.as_deref()
    }

    /// Commit `bodies` as one append, numbered by the fold and stamped with `app` and the time.
    /// A torn tail is cut first and recorded as `recovered`; the first commit of a date starts
    /// with `started`. A record the fold would not accept is refused and nothing is written.
    pub fn commit(
        &mut self,
        bodies: Vec<RecordBody>,
        app: &str,
    ) -> Result<Vec<Record>, StoreError> {
        if let Some(damage) = &self.damage {
            return Err(StoreError::Damaged(damage.clone()));
        }
        let mut all = Vec::with_capacity(bodies.len() + 2);
        if self.fold.last_seq == 0 {
            all.push(RecordBody::Started(Started {
                schema: SCHEMA.to_string(),
                namespace: self.fold.namespace.clone(),
                date: self.fold.date,
            }));
        }
        if let Some(torn) = &self.torn {
            all.push(RecordBody::Recovered(Recovered {
                dropped_bytes: torn.len() as u64,
                dropped_text: String::from_utf8_lossy(torn).into_owned(),
            }));
        }
        all.extend(bodies);

        let time = Datetime::now();
        let mut fold = self.fold.clone();
        let mut records = Vec::with_capacity(all.len());
        let mut bytes = Vec::new();
        for body in all {
            let record = Record {
                seq: fold.last_seq + 1,
                time: time.clone(),
                app: app.to_string(),
                body,
            };
            fold.apply(&record).map_err(StoreError::Rejected)?;
            bytes.extend(encode_line(&record));
            records.push(record);
        }

        self.append(&bytes)?;
        self.fold = fold;
        self.good_len += bytes.len() as u64;
        self.torn = None;
        Ok(records)
    }

    fn append(&self, bytes: &[u8]) -> Result<(), StoreError> {
        let path = &self.path;
        let io = |error| StoreError::io(path, error);
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(|e| StoreError::io(dir, e))?;
        }
        if self.torn.is_some() {
            // A handle opened for appending may not truncate on Windows.
            let cut = OpenOptions::new().write(true).open(path).map_err(io)?;
            cut.set_len(self.good_len).map_err(io)?;
            cut.sync_all().map_err(io)?;
        }
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(io)?;
        file.write_all(bytes).map_err(io)?;
        file.sync_all().map_err(io)
    }
}
