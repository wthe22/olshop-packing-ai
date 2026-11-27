//! A date whose views were changed outside the program: the Day screen names each file with the
//! lines that differ from the journal, and the owner's answer — **Rebuild**, **Read only** or
//! **Ignore** — holds until the program closes.

use packing_store::{EditedView, edited_views, write_views};
use serde::Serialize;
use tauri::State;

use super::date_of;
use crate::{AppState, EditedChoice, OpenDay};

/// The views of the date changed outside the program, unless the owner answered *Ignore*.
#[derive(Debug, Default, Serialize)]
pub struct EditedNote {
    files: Vec<EditedFile>,
    /// The owner answered *Read only*: no buttons, nothing of the date is written.
    read_only: bool,
}

#[derive(Debug, Serialize)]
pub struct EditedFile {
    file: &'static str,
    lines: Vec<EditedLine>,
    /// How many more lines differ.
    more: usize,
}

#[derive(Debug, Serialize)]
pub struct EditedLine {
    number: usize,
    /// The cell's column in `orders.csv`; none for a whole line.
    column: Option<String>,
    /// As the journal gives it.
    written: String,
    /// As the file holds it now.
    found: String,
}

pub(super) fn note(day: &OpenDay, choice: Option<EditedChoice>) -> EditedNote {
    if choice == Some(EditedChoice::Ignore) {
        return EditedNote::default();
    }
    let fold = day.journal.damage().is_none().then(|| day.journal.fold());
    EditedNote {
        files: edited_views(day.folder.dir(), fold)
            .into_iter()
            .map(file_view)
            .collect(),
        read_only: choice == Some(EditedChoice::ReadOnly),
    }
}

fn file_view(edited: EditedView) -> EditedFile {
    EditedFile {
        file: edited.file,
        lines: edited
            .lines
            .into_iter()
            .map(|line| EditedLine {
                number: line.number,
                column: line.column,
                written: line.written,
                found: line.found,
            })
            .collect(),
        more: edited.more,
    }
}

pub(crate) fn rebuild(state: &AppState, day: &str) -> Result<(), String> {
    let date = date_of(day)?;
    state.write_day(date, |open| {
        write_views(open.folder.dir(), open.journal.fold()).map_err(|error| error.to_string())
    })
}

pub(crate) fn choose(state: &AppState, day: &str, choice: EditedChoice) -> Result<(), String> {
    let date = date_of(day)?;
    state.with_data(|data| {
        data.edited_choices.insert(date, choice);
        Ok(())
    })
}

/// **Rebuild**: write the date's views again from its journal.
#[tauri::command]
pub fn rebuild_views(state: State<'_, AppState>, day: String) -> Result<(), String> {
    rebuild(&state, &day)
}

/// **Read only**: show the date as its files are and write nothing of it until the program closes.
#[tauri::command]
pub fn views_read_only(state: State<'_, AppState>, day: String) -> Result<(), String> {
    choose(&state, &day, EditedChoice::ReadOnly)
}

/// **Ignore**: show the date as its files are; the next write rewrites them.
#[tauri::command]
pub fn ignore_views(state: State<'_, AppState>, day: String) -> Result<(), String> {
    choose(&state, &day, EditedChoice::Ignore)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use jiff::civil::date;
    use packing_engine::ids::BatchId;
    use packing_engine::record::{OutputFile, PrintedMarked, RecordBody};
    use packing_store::{DateFolder, Journal, ORDERS_CSV, open_date, sha256_of};
    use serde_json::{Value, json};
    use testkit::TempDir;

    use super::*;
    use crate::commands::day::{mark_printed, overview};
    use crate::commands::start::use_folder;
    use crate::{APP_VERSION, DATE_READ_ONLY, settings};

    const DAY: &str = "2026-10-07";

    /// A data folder with one batch saved on `DAY` and its `orders.csv` changed by hand.
    fn edited_folder(tag: &str) -> (TempDir, AppState, DateFolder) {
        let program = TempDir::new(tag);
        let dir = settings::set_data_dir(program.path(), &program.path().join("data")).unwrap();
        let folder = DateFolder::new(&dir, date(2026, 10, 7));
        fs::create_dir_all(folder.dir()).unwrap();
        let group = folder.dir().join("1-A Sepatu x2.pdf");
        fs::write(&group, b"A").unwrap();
        let output = OutputFile {
            file: "1-A Sepatu x2.pdf".into(),
            group: Some("A".into()),
            pages: 1,
            size: 1,
            sha256: sha256_of(&group).unwrap(),
        };
        let mut journal = Journal::open(&folder).unwrap();
        let saved = testkit::batch(1, false, vec![output]);
        journal
            .commit(vec![RecordBody::BatchSaved(saved)], APP_VERSION)
            .unwrap();
        open_date(&folder).unwrap();
        let csv = folder.dir().join(ORDERS_CSV);
        let text = fs::read_to_string(&csv).unwrap();
        fs::write(&csv, format!("{text}extra\n")).unwrap();
        let state = AppState::new(program.path().to_path_buf());
        use_folder(&state, &dir).unwrap();
        (program, state, folder)
    }

    fn shown(state: &AppState) -> Value {
        serde_json::to_value(overview(state, DAY).unwrap()).unwrap()["edited"].clone()
    }

    fn printed(state: &AppState) -> Result<(), String> {
        let mark = PrintedMarked {
            batch: BatchId::new(1),
            group: "A".into(),
            printed: true,
        };
        mark_printed(state, DAY, mark)
    }

    #[test]
    fn a_changed_view_is_named_with_the_line_that_differs() {
        let (_program, state, folder) = edited_folder("app-edited");
        let lines = fs::read_to_string(folder.dir().join(ORDERS_CSV))
            .unwrap()
            .lines()
            .count();
        assert_eq!(
            shown(&state),
            json!({
                "files": [{
                    "file": ORDERS_CSV,
                    "lines": [{ "number": lines, "column": null, "written": "", "found": "extra" }],
                    "more": 0,
                }],
                "read_only": false,
            })
        );
        rebuild(&state, DAY).unwrap();
        assert_eq!(shown(&state)["files"], json!([]));
    }

    #[test]
    fn read_only_refuses_every_write_and_ignore_hides_the_note() {
        let (_program, state, folder) = edited_folder("app-edited-choice");
        choose(&state, DAY, EditedChoice::ReadOnly).unwrap();
        assert_eq!(shown(&state)["read_only"], true);
        assert_eq!(printed(&state), Err(DATE_READ_ONLY.to_string()));
        assert_eq!(rebuild(&state, DAY), Err(DATE_READ_ONLY.to_string()));
        assert!(
            fs::read_to_string(folder.dir().join(ORDERS_CSV))
                .unwrap()
                .ends_with("extra\n")
        );

        choose(&state, DAY, EditedChoice::Ignore).unwrap();
        assert_eq!(shown(&state)["files"], json!([]));
        printed(&state).unwrap();
        assert!(
            !fs::read_to_string(folder.dir().join(ORDERS_CSV))
                .unwrap()
                .contains("extra"),
            "the next write rewrites the views from the journal"
        );
    }
}
