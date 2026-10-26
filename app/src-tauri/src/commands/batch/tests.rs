//! The app over `packing-batch` on the repeat fixture: a batch read and saved shows the same Day
//! screen to a second copy of the app; Printed, amend and revert through the commands; the plan's
//! edits for this batch; the sort rule preselected and *used in batch n*; the logo prompt's answers;
//! the Repeated orders choices; search, reprint, restore and the panel.
//!
//! Skipped with a message when `pdfium.dll` has not been downloaded (tools/get-pdfium.sh).

use std::fs;
use std::path::{Path, PathBuf};

use jiff::civil::Date;
use packing_engine::ids::{BatchId, OrderId};
use packing_engine::orders::{LogoAnswer, UnseenLogo};
use packing_engine::record::PrintedMarked;
use packing_engine::workflows::{BUNDLED_WORKFLOWS, WORKFLOWS_FILE};
use serde_json::Value;
use testkit::{PdfiumGuard, TempDir, testdata, vendor_dir};

use super::logo::{answer_logo, read_pictures};
use super::read::{Reading, read_into_draft};
use super::repeats::RepeatChoices;
use super::save::save_draft;
use super::views::{PlanEdit, PlanView, ReadResult, SaveResultView};
use super::{amend_start, checked, plan_view, resolve, sort_rules_of};
use crate::AppState;
use crate::commands::day::{mark_printed, overview, restore_file, revert};
use crate::commands::panel::{open_workspace, panel};
use crate::commands::rules::{
    GroupInput, check, new_rule, renamed, rule_groups, rules_view, save_groups,
};
use crate::commands::search::{ReprintChoice, Scope, reprint_orders, search};
use crate::commands::start::{new_workspace, start_information, use_folder};

const DAY: &str = "2026-10-08";
const GROUP_FILE: &str = "1-Z Lainnya x4.pdf";

/// A program folder whose first start chose its `data` folder and named a workspace; the guard keeps
/// this test the only one using PDFium.
fn started(tag: &str) -> Option<(TempDir, AppState, PdfiumGuard)> {
    vendor_dir()?;
    let guard = testkit::pdfium_lock();
    let program = TempDir::new(tag);
    let state = AppState::new(program.path().to_path_buf());
    start_information(&state).unwrap();
    use_folder(&state, &program.path().join("data")).unwrap();
    new_workspace(&state, "Toko Contoh").unwrap();
    overview(&state, DAY).unwrap();
    Some((program, state, guard))
}

fn date() -> Date {
    DAY.parse().unwrap()
}

fn inputs() -> Vec<PathBuf> {
    ["labels-repeat-1.pdf", "labels-repeat-2.pdf"]
        .iter()
        .map(|name| testdata(name))
        .collect()
}

fn read_with(state: &AppState, rule: &str) -> Result<ReadResult, String> {
    let reading = Reading {
        paths: inputs(),
        rule: rule.into(),
        redo: None,
        date: date(),
    };
    read_into_draft(state, reading, |_| {})
}

fn read(state: &AppState) -> ReadResult {
    read_with(state, "default").unwrap()
}

fn save(state: &AppState) -> SaveResultView {
    let saved = save_draft(state, false, &|_| {}).unwrap();
    assert!(saved.saved, "{:?} {:?}", saved.errors, saved.warnings);
    saved
}

fn day(state: &AppState) -> Value {
    serde_json::to_value(overview(state, DAY).unwrap()).unwrap()
}

fn skipped() {
    eprintln!("skipped: pdfium.dll not downloaded (tools/get-pdfium.sh)");
}

#[test]
fn a_saved_batch_shows_the_same_day_after_a_restart() {
    let Some((program, state, guard)) = started("app-save") else {
        return skipped();
    };
    assert_eq!(day(&state)["batches"], Value::Array(Vec::new()));
    let read = read(&state);
    assert_eq!((read.orders, read.total_orders, read.files), (4, 4, 2));
    assert!(read.logos.is_empty());
    assert!(
        read.warnings[0].contains("more than once"),
        "{:?}",
        read.warnings
    );
    assert_eq!(read.plan.rows[0].file_name, GROUP_FILE);

    let saved = save(&state);
    assert_eq!((saved.batch, saved.pdfs), (BatchId::new(1), 1));
    let shown = day(&state);
    let batch = &shown["batches"][0];
    assert_eq!(batch["pdfs"][0]["file"], GROUP_FILE);
    assert_eq!(batch["summary"], "1 summary.pdf");
    assert_eq!(batch["orders"], 4);
    let left_out = batch["left_out"].as_array().unwrap();
    assert_eq!(left_out.len(), 2);
    assert_eq!(left_out[0]["reason"], "repeat", "{left_out:?}");
    assert!(
        left_out[0]["kept_input"]
            .as_str()
            .unwrap()
            .starts_with("labels-repeat-"),
        "the copy kept names its input: {left_out:?}"
    );

    drop(state);
    let again = AppState::new(program.path().to_path_buf());
    start_information(&again).unwrap();
    assert_eq!(
        day(&again),
        shown,
        "the journal alone gives the same Day screen"
    );
    drop(guard);
}

#[test]
fn printed_amend_and_revert_go_through_the_journal() {
    let Some((_program, state, _guard)) = started("app-amend") else {
        return skipped();
    };
    read(&state);
    save(&state);
    let mark = PrintedMarked {
        batch: BatchId::new(1),
        group: "Z".into(),
        printed: true,
    };
    mark_printed(&state, DAY, mark).unwrap();
    assert_eq!(day(&state)["batches"][0]["pdfs"][0]["printed"], true);

    let start = amend_start(&state, date(), |_| {}).unwrap();
    assert_eq!(
        (start.batch, start.read.amend),
        (BatchId::new(1), Some(BatchId::new(1)))
    );
    assert_eq!(start.files.len(), 2);
    assert!(
        start
            .files
            .iter()
            .all(|f| Path::new(&f.path).starts_with(state.program()))
    );
    assert_eq!(save(&state).batch, BatchId::new(1));
    let amended = day(&state);
    assert_eq!(amended["batches"][0]["amended"], true);
    assert_eq!(amended["batches"][0]["pdfs"][0]["printed"], false);

    assert_eq!(revert(&state, DAY).unwrap(), BatchId::new(1));
    assert_eq!(day(&state)["batches"], Value::Array(Vec::new()));
}

fn edit(state: &AppState, edits: &[PlanEdit]) -> Result<PlanView, String> {
    state.with_draft(|open| {
        super::plan_view::apply_edits(open, edits)?;
        state.with_data(|data| {
            let day = data.open_day(open.date)?;
            Ok(plan_view::plan_view(open, day.journal.fold()))
        })
    })
}

#[test]
fn a_one_off_group_skip_and_rename_change_this_batch_only() {
    let Some((_program, state, _guard)) = started("app-plan") else {
        return skipped();
    };
    read(&state);
    let added = edit(
        &state,
        &[
            PlanEdit::Add {
                at: 5,
                code: "S".into(),
                name: "Spion".into(),
                condition: "name contains \"spion\"".into(),
            },
            PlanEdit::Rename {
                index: 0,
                name: "Spion hari ini.pdf".into(),
            },
        ],
    )
    .unwrap();
    let counts: Vec<(&str, usize)> = added
        .rows
        .iter()
        .map(|row| (row.code.as_str(), row.orders))
        .collect();
    assert_eq!(counts, [("S", 2), ("Z", 2)], "added above the rest row");
    assert_eq!(added.rows[0].file_name, "Spion hari ini.pdf");

    let skipped = edit(
        &state,
        &[PlanEdit::SetSkipped {
            index: 0,
            skipped: true,
        }],
    )
    .unwrap();
    assert_eq!((skipped.rows[0].orders, skipped.rows[1].orders), (0, 4));

    let refused = PlanEdit::Add {
        at: 0,
        code: "X".into(),
        name: "Broken".into(),
        condition: "name contains".into(),
    };
    assert!(edit(&state, &[refused]).is_err());
    assert_eq!(
        edit(&state, &[]).unwrap().rows.len(),
        2,
        "the plan as before"
    );

    let saved = save(&state);
    assert_eq!(saved.pdfs, 1, "the skipped group writes no file");
}

#[test]
fn the_newest_batchs_rule_is_preselected_and_its_files_are_used_in_it() {
    let Some((program, state, _guard)) = started("app-rules") else {
        return skipped();
    };
    let second = "
[[label_sort_rules]]
name = \"harian\"

[[label_sort_rules.label_groups]]
code = \"H\"
name = \"Harian\"
";
    fs::write(
        program.path().join(WORKFLOWS_FILE),
        format!("{BUNDLED_WORKFLOWS}{second}"),
    )
    .unwrap();
    let before = sort_rules_of(&state).unwrap();
    assert_eq!(before.rules, ["default", "harian"]);
    assert_eq!(
        before.preselected, "default",
        "no batch yet: the first rule"
    );
    assert!(
        read_with(&state, "nobody").is_err(),
        "a rule the file does not hold"
    );

    read_with(&state, "harian").unwrap();
    save(&state);
    assert_eq!(sort_rules_of(&state).unwrap().preselected, "harian");
    let files = checked(&state, &inputs()).unwrap();
    assert!(
        files
            .iter()
            .all(|file| file.used_in_batch == Some(BatchId::new(1)))
    );
    assert_eq!(files[0].pages, 6);
}

#[test]
fn a_logo_answer_is_remembered_in_the_rules_and_the_prompt_moves_on() {
    let Some((program, state, _guard)) = started("app-logo") else {
        return skipped();
    };
    read(&state);
    let unseen = |hash: &str| UnseenLogo {
        hash: hash.into(),
        appearance: None,
        pages: 1,
        order_ids: vec![OrderId::new("580000000000000101")],
    };
    state
        .with_draft(|open| {
            open.draft.unseen_logos = vec![unseen("0123456789abcdef"), unseen("fedcba9876543210")];
            open.logo_pictures = state.with_worker(|worker| read_pictures(worker, &open.draft))?;
            Ok(())
        })
        .unwrap();

    let passed = answer_logo(&state, "fedcba9876543210", &LogoAnswer::Skip).unwrap();
    let asked: Vec<&str> = passed.logos.iter().map(|logo| logo.hash.as_str()).collect();
    assert_eq!(asked, ["0123456789abcdef"], "*I don't know* asks no more");
    assert_eq!(
        passed.logos[0].picture, None,
        "no page carries the made-up logo"
    );
    assert_eq!(passed.couriers[0], "J&T Express");
    let nobody = LogoAnswer::Courier {
        name: "Nobody".into(),
    };
    assert!(answer_logo(&state, "0123456789abcdef", &nobody).is_err());

    let jnt = LogoAnswer::Courier {
        name: "J&T Express".into(),
    };
    let named = answer_logo(&state, "0123456789abcdef", &jnt).unwrap();
    assert!(
        named.logos.is_empty(),
        "the copies were read again with the new rules"
    );
    answer_logo(&state, "fedcba9876543210", &LogoAnswer::NotCourier).unwrap();
    let rules = fs::read_to_string(program.path().join(WORKFLOWS_FILE)).unwrap();
    assert!(
        rules.contains("\"39b9b7b9f61d0e7f\", \"0123456789abcdef\""),
        "{rules}"
    );
    assert!(rules.contains("fedcba9876543210"));
}

#[test]
fn a_saved_order_is_found_reprinted_and_a_lost_output_restored() {
    let Some((program, state, _guard)) = started("app-search") else {
        return skipped();
    };
    read(&state);
    save(&state);
    state.index().wait();
    let found =
        serde_json::to_value(search(&state, "name contains \"\"", Scope::All, DAY).unwrap())
            .unwrap();
    assert_eq!(
        (found["orders"].clone(), found["batches"].clone()),
        (4.into(), 1.into())
    );
    let row = &found["rows"][0];
    assert_eq!(row["group"], "1-Z");
    let trace = row["trace"].as_str().unwrap();
    assert!(
        trace.starts_with("from labels-repeat-1.pdf p. ") && trace.ends_with("batch 1 · 1-Z"),
        "{trace}"
    );
    let on_other_day = search(&state, "name contains \"\"", Scope::Day, "2026-10-09").unwrap();
    assert_eq!(serde_json::to_value(on_other_day).unwrap()["orders"], 0);
    assert!(search(&state, "name has \"x\"", Scope::All, DAY).is_err());

    let choices: Vec<ReprintChoice> = found["rows"].as_array().unwrap()[..2]
        .iter()
        .map(|row| ReprintChoice {
            day: DAY.into(),
            order_id: row["order_id"].as_str().unwrap().into(),
        })
        .collect();
    let made = serde_json::to_value(reprint_orders(&state, &choices).unwrap()).unwrap();
    assert_eq!(made["errors"], Value::Array(Vec::new()));
    assert_eq!(made["made"]["file"], "reprint 1 x2.pdf");
    assert!(Path::new(made["made"]["path"].as_str().unwrap()).is_file());
    let today = crate::today().to_string();
    let shown = serde_json::to_value(overview(&state, &today).unwrap()).unwrap();
    assert_eq!(shown["reprints"][0]["orders"], 2);

    let output = program
        .path()
        .join("data/Toko Contoh")
        .join(DAY)
        .join("output");
    fs::remove_file(output.join(GROUP_FILE)).unwrap();
    let missing = day(&state);
    assert_eq!(
        missing["batches"][0]["pdfs"][0]["problem"], "missing",
        "the reprint opened today, so the date is checked again when it opens"
    );
    let restored = serde_json::to_value(restore_file(&state, DAY, GROUP_FILE).unwrap()).unwrap();
    assert_eq!(restored["restored"], true, "{restored}");
    assert!(output.join(GROUP_FILE).is_file());
}

#[test]
fn the_panel_lists_the_workspaces_and_the_dates_with_their_orders() {
    let Some((_program, state, _guard)) = started("app-panel") else {
        return skipped();
    };
    read(&state);
    save(&state);
    new_workspace(&state, "Toko Lain").unwrap();
    open_workspace(&state, "Toko Contoh").unwrap();
    assert!(open_workspace(&state, "Toko Ketiga").is_err());
    state.index().wait();
    let shown = serde_json::to_value(panel(&state).unwrap()).unwrap();
    assert_eq!(
        shown["workspaces"],
        serde_json::json!(["Toko Contoh", "Toko Lain"])
    );
    assert_eq!(shown["open"], "Toko Contoh");
    let dates = shown["dates"].as_array().unwrap();
    assert!(dates.iter().any(|row| row["today"] == true));
    let saved = dates.iter().find(|row| row["date"] == DAY).unwrap();
    assert_eq!(saved["orders"], 4);
    assert!(saved["folder"].as_str().unwrap().ends_with(DAY));
    let today = dates.iter().find(|row| row["today"] == true).unwrap();
    assert_eq!(
        today["folder"],
        Value::Null,
        "no folder before its first commit"
    );
}

#[test]
fn the_last_copies_are_kept_when_chosen_and_the_batch_saves() {
    let Some((_program, state, _guard)) = started("app-repeats") else {
        return skipped();
    };
    let first = serde_json::to_value(read(&state)).unwrap();
    let group = &first["repeats"]["groups"][0];
    assert_eq!(first["repeats"]["groups"].as_array().unwrap().len(), 1);
    assert_eq!(group["choice"]["kind"], "keep_first");
    assert_eq!(
        group["take_out"],
        serde_json::json!([{ "file": 2, "only_here": 1 }]),
        "file 1 holds a repeat with no copy elsewhere; file 2 also holds an order of its own"
    );

    let choices = |value: Value| -> RepeatChoices { serde_json::from_value(value).unwrap() };
    let wrong = choices(serde_json::json!({
        "groups": [],
        "by_hand": [{ "order_id": "580000000000000103", "copy": 0 }]
    }));
    assert!(
        resolve(&state, &wrong).is_err(),
        "a copy of another order is refused"
    );
    let by_hand = choices(serde_json::json!({
        "groups": [],
        "by_hand": [{ "order_id": "580000000000000103", "copy": 3 }]
    }));
    let chosen = serde_json::to_value(resolve(&state, &by_hand).unwrap()).unwrap();
    assert_eq!(chosen["repeats"]["groups"][0]["by_hand"], true);

    let last = choices(serde_json::json!({
        "groups": [{ "group": 1, "choice": { "kind": "keep_last" } }],
        "by_hand": []
    }));
    let resolved = serde_json::to_value(resolve(&state, &last).unwrap()).unwrap();
    let files = &resolved["repeats"]["groups"][0]["files"];
    assert_eq!(
        (files[0]["kept"].clone(), files[0]["left_out"].clone()),
        (1.into(), 2.into())
    );
    save(&state);
    let shown = day(&state);
    let left_out = shown["batches"][0]["left_out"].as_array().unwrap();
    let spion = left_out
        .iter()
        .find(|copy| copy["order_id"] == "580000000000000102")
        .unwrap();
    assert_eq!(
        (spion["page_start"].clone(), spion["kept_page"].clone()),
        (2.into(), 4.into()),
        "the first copy left out, the last kept"
    );
}

#[test]
fn a_saved_sort_rule_keeps_the_files_comments_and_the_open_plan_follows_it() {
    let Some((program, state, _guard)) = started("app-rules") else {
        return skipped();
    };
    read(&state);
    let groups = |value: Value| -> Vec<GroupInput> { serde_json::from_value(value).unwrap() };
    let spion = groups(serde_json::json!([
        { "code": "A", "name": "Spion", "filter": "name contains \"spion\"", "origin": null },
        { "code": "Z", "name": "Lainnya", "filter": null, "origin": 0 }
    ]));
    let checked = serde_json::to_value(check(&state, "default", &spion)).unwrap();
    assert_eq!(checked["general"], Value::Null, "{checked}");
    assert_eq!(
        checked["counts"],
        serde_json::json!([2, 2]),
        "the two spion orders, then the rest"
    );
    let broken = groups(serde_json::json!([
        { "code": "A", "name": "Spion", "filter": "name has \"spion\"", "origin": null },
        { "code": "Z", "name": "Lainnya", "filter": null, "origin": 0 }
    ]));
    let refused = serde_json::to_value(check(&state, "default", &broken)).unwrap();
    assert!(
        refused["errors"][0].is_string() && refused["counts"].is_null(),
        "{refused}"
    );

    save_groups(&state, "default", &spion).unwrap();
    let file = fs::read_to_string(program.path().join(WORKFLOWS_FILE)).unwrap();
    assert!(file.contains("# Sort rules: a batch is sorted by the rule chosen for it."));
    assert!(
        file.contains("filter = 'name contains \"spion\"'"),
        "{file}"
    );
    let plan = |state: &AppState| {
        state.with_draft(|open| {
            state.with_data(|data| {
                let day = data.open_day(open.date)?;
                Ok(plan_view::plan_view(open, day.journal.fold()))
            })
        })
    };
    let followed = plan(&state).unwrap();
    assert_eq!(
        followed.rows[0].code, "A",
        "the open plan follows the saved rule"
    );

    new_rule(&state, "harian", Some("default")).unwrap();
    renamed(&state, "default", "biasa").unwrap();
    let view = serde_json::to_value(rules_view(&state).unwrap()).unwrap();
    assert_eq!(view["rules"], serde_json::json!(["biasa", "harian"]));
    assert_eq!(
        view["draft_rule"], "biasa",
        "the draft's rule takes the new name"
    );
    assert_eq!(plan(&state).unwrap().sort_rule, "biasa");
    assert_eq!(rule_groups(&state, "harian").unwrap().len(), 2);
}
