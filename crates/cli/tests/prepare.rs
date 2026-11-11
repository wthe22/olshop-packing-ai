//! End-to-end test of `packing-cli prepare/revert/amend` on the made-up testdata: the date folder
//! with its journal, inputs and outputs, the duplicate guard, the account rules, revert, amend and
//! the no-slip stop.
//!
//! Skipped with a message when `pdfium.dll` has not been downloaded (tools/get-pdfium.sh).

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

use jiff::civil::date;
use packing_store::{DateFolder, Fold, view_date};
use testkit::{TempDir, testdata, vendor_dir};

const DAY: &str = "2026-10-08";
const REPEAT_ACCOUNTS: &str = "\
[[accounts]]
name = \"Toko Contoh\"
shops = [\"Toko Contoh\"]

[[accounts]]
name = \"Toko Lain\"
shops = [\"Toko Lain\"]
";

/// A program folder and a data folder, and the way to run the built binary on them.
struct Setup {
    program: TempDir,
    data: PathBuf,
    vendor: PathBuf,
}

impl Setup {
    fn new(tag: &str, vendor: PathBuf, settings: Option<&str>) -> Setup {
        let program = TempDir::new(tag);
        if let Some(text) = settings {
            fs::write(program.path().join("packing-settings.toml"), text).unwrap();
        }
        let data = program.path().join("data");
        Setup {
            program,
            data,
            vendor,
        }
    }

    fn run(&self, command: &str, rest: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_packing-cli"))
            .arg(command)
            .args(rest)
            .args(["--day", DAY])
            .arg("--program")
            .arg(self.program.path())
            .arg("--data")
            .arg(&self.data)
            .arg("--pdfium")
            .arg(&self.vendor)
            .output()
            .expect("run packing-cli")
    }

    fn fold(&self) -> Fold {
        let folder = DateFolder::new(&self.data, date(2026, 10, 8));
        view_date(&folder)
            .expect("open the date")
            .journal
            .fold()
            .clone()
    }

    fn output(&self, file: &str) -> PathBuf {
        self.data.join(DAY).join("output").join(file)
    }
}

fn labels(names: &[&str]) -> Vec<String> {
    let mut args = vec!["--labels".to_string()];
    args.extend(
        names
            .iter()
            .map(|name| testdata(name).to_string_lossy().into_owned()),
    );
    args
}

fn run_labels(setup: &Setup, command: &str, names: &[&str], extra: &[&str]) -> Output {
    let mut args = labels(names);
    args.extend(extra.iter().map(|s| s.to_string()));
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    setup.run(command, &args)
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn succeeded(output: &Output) -> String {
    assert!(output.status.success(), "{}", text(&output.stderr));
    text(&output.stdout)
}

fn failed(output: &Output) -> String {
    assert!(
        !output.status.success(),
        "should have stopped: {}",
        text(&output.stdout)
    );
    text(&output.stderr)
}

const REPEAT: [&str; 2] = ["labels-repeat-1.pdf", "labels-repeat-2.pdf"];

#[test]
fn prepare_revert_amend_round_trip() {
    let Some(vendor) = vendor_dir() else {
        eprintln!("skipped: run tools/get-pdfium.sh");
        return;
    };
    let _guard = testkit::pdfium_lock();
    let setup = Setup::new("cli-roundtrip", vendor, Some(REPEAT_ACCOUNTS));

    let said = succeeded(&run_labels(&setup, "prepare", &REPEAT, &[]));
    assert!(said.contains("saved batch 1"), "{said}");
    assert!(setup.data.join(DAY).join("journal.jsonl").is_file());
    assert!(setup.output("1-Z Lainnya x4.pdf").is_file());
    assert!(setup.output("1 summary.pdf").is_file());
    let fold = setup.fold();
    assert_eq!(fold.live.len(), 1);
    let first = fold.live.values().next().unwrap().saved.clone();
    assert_eq!(first.orders.len(), 4);
    assert_eq!(first.inputs.len(), 2);

    let said = failed(&run_labels(&setup, "prepare", &REPEAT, &[]));
    assert!(said.contains("already saved"), "{said}");
    assert_eq!(
        setup.fold().last_seq,
        fold.last_seq,
        "nothing new is written"
    );

    let said = succeeded(&setup.run("revert", &[]));
    assert!(said.contains("reverted batch 1"), "{said}");
    assert!(setup.fold().live.is_empty());
    assert!(!setup.output("1-Z Lainnya x4.pdf").exists());
    let said = succeeded(&setup.run("revert", &[]));
    assert!(said.contains("no batch to revert"), "{said}");

    succeeded(&run_labels(&setup, "prepare", &REPEAT, &[]));
    let again = setup.fold().live.values().next().unwrap().saved.clone();
    let files = |record: &packing_engine::record::BatchSaved| -> Vec<String> {
        record.outputs.iter().map(|o| o.file.clone()).collect()
    };
    assert_eq!(files(&again), files(&first), "the same files come back");

    let before = setup.fold().last_seq;
    let said = succeeded(&setup.run("amend", &[]));
    assert!(said.contains("saved batch 1"), "{said}");
    let amended = setup.fold();
    assert!(amended.last_seq > before, "the amend is a record");
    assert_eq!(
        files(&amended.live.values().next().unwrap().saved),
        files(&first)
    );

    let said = failed(&run_labels(&setup, "prepare", &["labels-plain.pdf"], &[]));
    assert!(said.contains("no packing slip"), "{said}");
    assert_eq!(
        setup.fold().last_seq,
        amended.last_seq,
        "plain labels write nothing"
    );
}

#[test]
fn an_order_of_an_unlisted_shop_stops_unless_an_account_is_given() {
    let Some(vendor) = vendor_dir() else {
        eprintln!("skipped: run tools/get-pdfium.sh");
        return;
    };
    let _guard = testkit::pdfium_lock();
    let setup = Setup::new("cli-accounts", vendor, None);

    let said = failed(&run_labels(&setup, "prepare", &REPEAT, &[]));
    assert!(said.contains("have no account"), "{said}");
    assert!(said.contains("Toko Contoh"), "the shops are named: {said}");
    let said = failed(&run_labels(&setup, "prepare", &REPEAT, &["--quiet-ids"]));
    assert!(said.contains("shops no account lists"), "{said}");
    assert!(!said.contains("Toko Contoh"), "counts only: {said}");
    assert!(!setup.data.join(DAY).join("journal.jsonl").exists());

    let said = succeeded(&run_labels(
        &setup,
        "prepare",
        &REPEAT,
        &["--account", "Toko Alfa"],
    ));
    assert!(said.contains("saved batch 1"), "{said}");
    let fold = setup.fold();
    let record = &fold.live.values().next().unwrap().saved;
    assert!(
        record
            .orders
            .iter()
            .all(|order| order.account == "Toko Alfa")
    );
    assert!(
        !setup.program.path().join("packing-settings.toml").exists(),
        "the run writes no settings"
    );
}

#[test]
fn what_cannot_run_is_said() {
    let Some(vendor) = vendor_dir() else {
        eprintln!("skipped: run tools/get-pdfium.sh");
        return;
    };
    let _guard = testkit::pdfium_lock();
    let setup = Setup::new("cli-refusals", vendor, Some(REPEAT_ACCOUNTS));
    let said = failed(&run_labels(&setup, "prepare", &REPEAT, &["--rule", "nope"]));
    assert!(said.contains("no sort rule \"nope\""), "{said}");
    let said = failed(&setup.run("prepare", &[]));
    assert!(said.contains("--labels"), "{said}");
    let said = succeeded(&setup.run("amend", &[]));
    assert!(said.contains("no batch to amend"), "{said}");
    let said = failed(&setup.run("frobnicate", &[]));
    assert!(said.contains("unknown command"), "{said}");
    fs::write(setup.program.path().join("packing-workflows.toml"), "[[").unwrap();
    let said = failed(&run_labels(&setup, "prepare", &REPEAT, &[]));
    assert!(said.contains("packing-workflows.toml"), "{said}");
    assert!(!setup.data.join(DAY).exists());
}

#[test]
fn the_usage_is_printed_without_a_command() {
    let output = Command::new(env!("CARGO_BIN_EXE_packing-cli"))
        .output()
        .expect("run packing-cli");
    assert!(text(&output.stdout).contains("USAGE"));
    assert!(output.status.success());
}
