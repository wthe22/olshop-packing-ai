use std::fs;

use packing_engine::workflows::{CONFIG_FILE, PARSER_FILE};
use testkit::TempDir;

use super::*;
use crate::settings::APP_FILE;

#[test]
fn the_shown_version_comes_from_one_place() {
    let shown = format!("v{}", env!("CARGO_PKG_VERSION"));
    assert_eq!(version_text(), shown);
    assert_eq!(window_title(), format!("Packing {shown}"));
    assert_eq!(about().version, shown);
}

#[test]
fn the_build_time_is_a_local_date_and_minute() {
    let built = about().built;
    assert!(
        jiff::civil::DateTime::strptime("%Y-%m-%d %H:%M", &built).is_ok(),
        "{built}"
    );
}

#[test]
fn explorer_gets_the_path_as_its_only_argument() {
    let path = r"C:\Packing\packing-data\2026-10-07\output";
    let command = explorer_command(path);
    let args: Vec<_> = command.get_args().collect();
    assert_eq!(args, [path]);
}

#[test]
fn a_first_start_opens_the_data_folder_and_writes_its_two_files() {
    let _guard = testkit::pdfium_lock();
    let program = TempDir::new("app-start");
    let state = AppState::new(program.path().to_path_buf());
    let info = start_information(&state).unwrap();
    assert!(info.data_folder.is_none());
    assert!(
        !program.path().join(APP_FILE).exists(),
        "nothing is written before a data folder is chosen"
    );

    let dir = use_folder(&state, &program.path().join(DEFAULT_DATA_DIR)).unwrap();
    let written = fs::read_to_string(program.path().join(APP_FILE)).unwrap();
    assert!(written.contains("data_dir = \"packing-data\""), "{written}");
    for file in [PARSER_FILE, CONFIG_FILE] {
        assert!(Path::new(&dir).join(file).is_file(), "the bundled {file}");
    }
    assert!(
        Path::new(&dir).join(".lock").is_file(),
        "its lock is taken once"
    );

    let second = AppState::new(program.path().to_path_buf());
    let info = start_information(&second).expect("a second copy shares the folder");

    assert_eq!(info.data_folder.as_deref(), Some(dir.as_str()));
}

#[test]
fn the_theme_and_the_data_folder_are_kept_apart_in_the_settings() {
    let program = TempDir::new("app-theme");
    assert_eq!(settings::theme(program.path()).unwrap(), Theme::System);
    settings::set_theme(program.path(), Theme::Dark).unwrap();
    let dir = program.path().join(DEFAULT_DATA_DIR);
    settings::set_data_dir(program.path(), &dir).unwrap();
    assert_eq!(settings::theme(program.path()).unwrap(), Theme::Dark);
    settings::set_theme(program.path(), Theme::Light).unwrap();
    assert_eq!(settings::data_dir(program.path()).unwrap(), Some(dir));
    let written = fs::read_to_string(program.path().join(APP_FILE)).unwrap();
    assert!(written.contains("theme = \"light\""), "{written}");
}
