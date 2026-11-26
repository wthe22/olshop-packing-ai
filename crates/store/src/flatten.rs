//! A date folder of v0.7 made flat: its `input/`, `output/` and summaries made on request
//! move unchanged into `v0.7 backup/`, and each file is copied up under its v0.8 name and checked by
//! its hash. The journal is only read and nothing is deleted: the owner's `remove-v07-backup.py`
//! deletes a backup once every file of it is in the date folder, by the same names.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use packing_engine::record::{RecordBody, Sha256};

use crate::StoreError;
use crate::date::DateFolder;
use crate::files::{move_file, sha256_of};
use crate::inputs::input_path;
use crate::journal::Journal;
use crate::line::{Entry, parse};

pub const V0_7_BACKUP: &str = "v0.7 backup";
/// Written last: the date was converted to the end.
const COMPLETE: &str = "backup complete.txt";
/// Where a file of the date folder that holds other bytes than its backup's is moved aside.
const REPLACED: &str = "replaced";
/// The subfolders a date folder of v0.7 and older keeps its inputs and outputs in.
pub(crate) const V0_7_FOLDERS: [&str; 2] = ["input", "output"];

/// Whether the date folder still keeps v0.7's folders, or a backup whose conversion stopped.
pub fn is_v0_7(dir: &Path) -> bool {
    let backup = dir.join(V0_7_BACKUP);
    V0_7_FOLDERS.iter().any(|name| dir.join(name).is_dir())
        || backup.is_dir() && !backup.join(COMPLETE).is_file()
}

/// Make the date folder flat; `progress` hears each file copied up of how many. A stopped
/// conversion continues: what is in place already is left.
pub fn flatten_date(
    folder: &DateFolder,
    journal: &Journal,
    progress: &mut dyn FnMut(usize, usize),
) -> Result<(), StoreError> {
    let dir = folder.dir();
    let backup = dir.join(V0_7_BACKUP);
    fs::create_dir_all(&backup).map_err(|e| StoreError::io(&backup, e))?;
    for item in old_items(dir)? {
        move_file(&item, &backup.join(item.file_name().unwrap_or_default()))?;
    }
    let partial = backup.join("output").join(".partial");
    if partial.is_dir() && !dir.join(".partial").exists() {
        copy_folder(&partial, &dir.join(".partial"))?;
    }
    let sources = sources(&backup, &named(folder, journal)?)?;
    for (done, source) in sources.iter().enumerate() {
        copy_up(dir, &backup, source)?;
        progress(done + 1, sources.len());
    }
    let complete = backup.join(COMPLETE);
    fs::write(
        &complete,
        "Every file of Packing v0.7 of this date is in this folder.\n",
    )
    .map_err(|e| StoreError::io(&complete, e))
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

/// What v0.7 left in the date folder: `input/`, `output/` and the summaries made on request.
fn old_items(dir: &Path) -> Result<Vec<PathBuf>, StoreError> {
    let mut items: Vec<PathBuf> = V0_7_FOLDERS
        .iter()
        .map(|name| dir.join(name))
        .filter(|path| path.is_dir())
        .collect();
    for path in files_in(dir)? {
        if file_name(&path).and_then(new_summary_name).is_some() {
            items.push(path);
        }
    }
    Ok(items)
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
        if let Entry::Record(record) = record
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

/// A file of the backup, its name in the date folder, its bytes, and the bytes the journal names
/// for that name: none for a summary made on request, which the program may have made again.
struct Source {
    path: PathBuf,
    name: String,
    have: Sha256,
    wanted: Option<Sha256>,
}

/// Each label PDF once for every name the journal gives its bytes (one it names none for stays in
/// the backup), then each output and summary.
fn sources(backup: &Path, named: &Named) -> Result<Vec<Source>, StoreError> {
    let mut sources = Vec::new();
    for path in files_in(&backup.join("input"))? {
        let have = sha256_of(&path)?;
        for name in named.inputs.get(&have).into_iter().flatten() {
            sources.push(Source {
                path: path.clone(),
                name: name.clone(),
                have: have.clone(),
                wanted: Some(have.clone()),
            });
        }
    }
    let loose = files_in(backup)?
        .into_iter()
        .filter(|path| file_name(path).and_then(new_summary_name).is_some());
    for path in files_in(&backup.join("output"))?.into_iter().chain(loose) {
        let old = file_name(&path).unwrap_or_default();
        let name = new_summary_name(old).unwrap_or_else(|| old.to_string());
        sources.push(Source {
            have: sha256_of(&path)?,
            wanted: named.outputs.get(&name.to_lowercase()).cloned(),
            path,
            name,
        });
    }
    Ok(sources)
}

/// Copy one source up unless the date folder holds it already. An output the journal names newer
/// bytes for was written again by the program and is left; a file holding other bytes is moved into
/// `replaced/` of the backup first.
fn copy_up(dir: &Path, backup: &Path, source: &Source) -> Result<(), StoreError> {
    if source
        .wanted
        .as_ref()
        .is_some_and(|wanted| *wanted != source.have)
    {
        return Ok(());
    }
    let target = dir.join(&source.name);
    if target.is_file() {
        let wanted = match &source.wanted {
            None => return Ok(()),
            Some(wanted) => wanted,
        };
        if sha256_of(&target)? == *wanted {
            return Ok(());
        }
        let aside = backup.join(REPLACED);
        fs::create_dir_all(&aside).map_err(|e| StoreError::io(&aside, e))?;
        move_file(&target, &aside.join(&source.name))?;
    }
    copy_checked(&source.path, &target, &source.have)
}

/// Copy through `{name}.partial`, synced and hashed again, then renamed.
fn copy_checked(source: &Path, target: &Path, sha256: &Sha256) -> Result<(), StoreError> {
    let partial =
        target.with_file_name(format!("{}.partial", file_name(target).unwrap_or_default()));
    let mut from = fs::File::open(source).map_err(|e| StoreError::io(source, e))?;
    let mut copy = fs::File::create(&partial).map_err(|e| StoreError::io(&partial, e))?;
    std::io::copy(&mut from, &mut copy)
        .and_then(|_| copy.sync_all())
        .map_err(|e| StoreError::io(&partial, e))?;
    drop(copy);
    if sha256_of(&partial)? != *sha256 {
        let _ = fs::remove_file(&partial);
        return Err(StoreError::InputChanged {
            path: source.to_path_buf(),
        });
    }
    move_file(&partial, target)
}

fn copy_folder(from: &Path, to: &Path) -> Result<(), StoreError> {
    fs::create_dir_all(to).map_err(|e| StoreError::io(to, e))?;
    for entry in fs::read_dir(from).map_err(|e| StoreError::io(from, e))? {
        let path = entry.map_err(|e| StoreError::io(from, e))?.path();
        let target = to.join(path.file_name().unwrap_or_default());
        if path.is_dir() {
            copy_folder(&path, &target)?;
        } else {
            fs::copy(&path, &target).map_err(|e| StoreError::io(&path, e))?;
        }
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
