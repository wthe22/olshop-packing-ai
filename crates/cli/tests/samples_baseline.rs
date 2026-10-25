//! The local baseline of the owner's sample files: `packing-cli dump` of each `samples/*.pdf`
//! must equal `work/baseline/<file>.json`, so a change to the reading shows on the real
//! downloads. Both folders are confidential and never committed.
//!
//! A missing baseline file is written from today's reading. A change meant to alter the reading
//! deletes the baseline files it alters, runs this test again and states the difference in its
//! commit message. Skipped without `samples/` or `pdfium.dll`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use testkit::vendor_dir;

fn dump(file: &Path, vendor: &Path) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_packing-cli"))
        .arg("dump")
        .arg("--labels")
        .arg(file)
        .arg("--pdfium")
        .arg(vendor)
        .output()
        .expect("run packing-cli");
    if output.status.success() {
        String::from_utf8(output.stdout).expect("UTF-8 JSON")
    } else {
        format!("{}", String::from_utf8_lossy(&output.stderr))
    }
}

#[test]
fn the_sample_files_read_as_their_baseline() {
    let Some(vendor) = vendor_dir() else {
        eprintln!("skipped: pdfium.dll not downloaded (tools/get-pdfium.sh)");
        return;
    };
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let Ok(entries) = fs::read_dir(root.join("samples")) else {
        eprintln!("skipped: no samples/ folder");
        return;
    };
    let mut files: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "pdf"))
        .collect();
    files.sort();
    let baseline = root.join("work").join("baseline");
    fs::create_dir_all(&baseline).expect("work/baseline/");

    let mut written = Vec::new();
    let mut differ = Vec::new();
    for file in &files {
        let name = file.file_name().unwrap().to_string_lossy().into_owned();
        let text = dump(file, &vendor);
        let path = baseline.join(format!("{name}.json"));
        match fs::read_to_string(&path) {
            Ok(kept) if kept == text => {}
            Ok(_) => differ.push(name),
            Err(_) => {
                fs::write(&path, &text).expect("write the baseline");
                written.push(name);
            }
        }
    }
    if !written.is_empty() {
        eprintln!("baseline written for {written:?}");
    }
    assert!(
        differ.is_empty(),
        "{differ:?} no longer read as their baseline in work/baseline/: compare the JSON; delete a \
         baseline file only when the change is meant"
    );
}
