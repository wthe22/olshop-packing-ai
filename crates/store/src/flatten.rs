//! A date folder of v0.7 made flat: each file of its `input/` and `output/` and each summary made on
//! request moves up under its v0.8 name, every move listed first in the Update's `update.toml`
//!. A label PDF two batches name is copied to its second name. The journal is only read.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use packing_engine::record::{RecordBody, Sha256};

use crate::StoreError;
use crate::date::DateFolder;
use crate::files::sha256_of;
use crate::inputs::input_path;
use crate::journal::Journal;
use crate::line::{Entry, parse};
use crate::update_log::{UpdateLog, listed_flat};

/// Where v0.8.8 and older kept a date's v0.7 folders when it made the date flat.
pub const V0_7_BACKUP: &str = "v0.7 backup";
/// Written last by v0.8.8 and older: that conversion ran to the end.
const COMPLETE: &str = "backup complete.txt";
/// The subfolders a date folder of v0.7 and older keeps its inputs and outputs in.
pub(crate) const V0_7_FOLDERS: [&str; 2] = ["input", "output"];

/// Whether the date folder still keeps v0.7's folders, or a backup of v0.8.8 whose conversion
/// stopped, and no Update has listed it flat.
pub fn is_v0_7(dir: &Path) -> bool {
    let old = V0_7_FOLDERS.iter().any(|name| dir.join(name).is_dir()) || stopped_backup(dir);
    old && !listed_flat(dir)
}

fn stopped_backup(dir: &Path) -> bool {
    let backup = dir.join(V0_7_BACKUP);
    backup.is_dir() && !backup.join(COMPLETE).is_file()
}

/// Make the date folder flat; `progress` hears each file put in place of how many. A stopped
/// conversion continues: what is in place already is left.
pub fn flatten_date(
    folder: &DateFolder,
    journal: &Journal,
    log: &mut UpdateLog,
    progress: &mut dyn FnMut(usize, usize),
) -> Result<(), StoreError> {
    let dir = folder.dir();
    let mut roots = vec![dir.to_path_buf()];
    if stopped_backup(dir) {
        roots.push(dir.join(V0_7_BACKUP));
    }
    let partial = dir.join(".partial");
    for root in &roots {
        let old = root.join("output").join(".partial");
        if old.is_dir() && !partial.exists() {
            log.moving(&old, &partial)?;
        }
    }
    let sources = sources(&roots, &named(folder, journal)?)?;
    let files = sources.iter().map(|source| source.names.len()).sum();
    let mut done = 0;
    for source in &sources {
        put_in_place(dir, log, source, &mut || {
            done += 1;
            progress(done, files);
        })?;
    }
    for root in &roots {
        for name in V0_7_FOLDERS {
            // Only an emptied folder goes; one that keeps a file no journal names stays.
            let _ = fs::remove_dir(root.join(name));
        }
    }
    log.flattened()
}

/// The v0.8 name of a summary made on request (`day summary.pdf`, `day summary {account}.pdf`,
/// `{batch} summary {account}.pdf`), or none for any other file.
fn new_summary_name(name: &str) -> Option<String> {
    let stem = name.strip_suffix(".pdf")?;
    if stem == "day summary" {
        return Some("Day Summary.pdf".into());
    }
    if let Some(account) = stem.strip_prefix("day summary ") {
        return (!account.is_empty()).then(|| format!("Day Summary - {account}.pdf"));
    }
    let (batch, rest) = stem.split_once(' ')?;
    let account = rest.strip_prefix("summary ")?;
    let numbered = !batch.is_empty() && batch.bytes().all(|byte| byte.is_ascii_digit());
    (numbered && !account.is_empty()).then(|| format!("{batch} Summary - {account}.pdf"))
}

/// What the journal names: each label PDF's names in the date folder by its bytes (every batch that
/// recorded it, reverted or not), and each output of a live batch or a reprint with the bytes its
/// newest record names, by its lower-case name.
struct Named {
    inputs: BTreeMap<Sha256, BTreeSet<String>>,
    outputs: BTreeMap<String, Sha256>,
}

fn named(folder: &DateFolder, journal: &Journal) -> Result<Named, StoreError> {
    let parsed = parse(&journal.bytes()?).map_err(StoreError::Damaged)?;
    let mut inputs: BTreeMap<Sha256, BTreeSet<String>> = BTreeMap::new();
    for (_, record) in &parsed.records {
        if let Entry::Record { record, .. } = record
            && let RecordBody::BatchSaved(saved) = &record.body
        {
            for input in &saved.inputs {
                let path = input_path(folder, saved.batch, input.pdf, &input.sha256);
                let name = file_name(&path).unwrap_or_default().to_string();
                inputs.entry(input.sha256.clone()).or_default().insert(name);
            }
        }
    }
    let fold = journal.fold();
    let mut outputs = BTreeMap::new();
    for live in fold.live.values() {
        for output in &live.saved.outputs {
            outputs.insert(output.file.to_lowercase(), output.sha256.clone());
        }
        for restored in live.restored.values() {
            outputs.insert(restored.file.to_lowercase(), restored.sha256.clone());
        }
    }
    for reprint in &fold.reprints {
        outputs.insert(
            reprint.made.file.to_lowercase(),
            reprint.made.sha256.clone(),
        );
    }
    Ok(Named { inputs, outputs })
}

/// A file of v0.7, its names in the date folder, its bytes, and the bytes the journal names for
/// them: none for a summary made on request, which the program may have made again.
struct Source {
    path: PathBuf,
    names: Vec<String>,
    have: Sha256,
    wanted: Option<Sha256>,
}

/// Each label PDF with every name the journal gives its bytes (one it names none for stays where it
/// is), then each output and summary.
fn sources(roots: &[PathBuf], named: &Named) -> Result<Vec<Source>, StoreError> {
    let mut sources = Vec::new();
    for root in roots {
        for path in files_in(&root.join("input"))? {
            let have = sha256_of(&path)?;
            let names: Vec<String> = named
                .inputs
                .get(&have)
                .into_iter()
                .flatten()
                .cloned()
                .collect();
            if !names.is_empty() {
                sources.push(Source {
                    path,
                    names,
                    wanted: Some(have.clone()),
                    have,
                });
            }
        }
    }
    for root in roots {
        let loose = files_in(root)?
            .into_iter()
            .filter(|path| file_name(path).and_then(new_summary_name).is_some());
        for path in files_in(&root.join("output"))?.into_iter().chain(loose) {
            let old = file_name(&path).unwrap_or_default();
            let name = new_summary_name(old).unwrap_or_else(|| old.to_string());
            sources.push(Source {
                have: sha256_of(&path)?,
                wanted: named.outputs.get(&name.to_lowercase()).cloned(),
                names: vec![name],
                path,
            });
        }
    }
    Ok(sources)
}

/// Move one source to its first name and copy it to the others, unless the date folder holds it
/// already. An output the journal names newer bytes for was written again by the program, and the
/// old one stays where it is; a file in the way that holds other bytes is set aside first.
fn put_in_place(
    dir: &Path,
    log: &mut UpdateLog,
    source: &Source,
    tick: &mut dyn FnMut(),
) -> Result<(), StoreError> {
    if source
        .wanted
        .as_ref()
        .is_some_and(|wanted| *wanted != source.have)
    {
        return Ok(());
    }
    let mut placed: Option<PathBuf> = None;
    for name in &source.names {
        let target = dir.join(name);
        // Windows sees `day summary.pdf` as `Day Summary.pdf`: that one is renamed, not left.
        let itself =
            target.to_string_lossy().to_lowercase() == source.path.to_string_lossy().to_lowercase();
        if target.is_file() && !itself {
            match &source.wanted {
                None => {
                    tick();
                    continue;
                }
                Some(wanted) if sha256_of(&target)? == *wanted => {
                    placed.get_or_insert(target);
                    tick();
                    continue;
                }
                Some(_) => log.set_aside(&target)?,
            }
        }
        match &placed {
            None => {
                log.moving(&source.path, &target)?;
                placed = Some(target);
            }
            Some(first) => log.copying(first, &target, &source.have)?,
        }
        tick();
    }
    Ok(())
}

/// The files directly in `dir`, by name; none when it is not there.
fn files_in(dir: &Path) -> Result<Vec<PathBuf>, StoreError> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    for entry in fs::read_dir(dir).map_err(|e| StoreError::io(dir, e))? {
        let path = entry.map_err(|e| StoreError::io(dir, e))?.path();
        if path.is_file() {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

fn file_name(path: &Path) -> Option<&str> {
    path.file_name().and_then(|name| name.to_str())
}
