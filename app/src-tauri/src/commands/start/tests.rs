use std::fs;

use packing_engine::workflows::WORKFLOWS_FILE;
use testkit::TempDir;

use super::*;
use crate::settings::SETTINGS_FILE;

#[test]
fn the_shown_version_comes_from_one_place() {
    let mut numbers = env!("CARGO_PKG_VERSION").split('.');
    let shown = format!("v{}.{}", numbers.next().unwrap(), numbers.next().unwrap());
    assert_eq!(version_text(), shown);
    assert_eq!(window_title(), format!("Packing {shown}"));
    assert_eq!(about().version, shown);
}

#[test]
fn explorer_gets_the_path_as_its_only_argument() {
    let path = r"C:\Packing\packing-data\Toko Contoh\2026-10-07\output";
    let command = explorer_command(path);
    let args: Vec<_> = command.get_args().collect();
    assert_eq!(args, [path]);
}

#[test]
fn a_first_start_writes_the_two_files_and_names_the_first_workspace() {
    let _guard = testkit::pdfium_lock();
    let program = TempDir::new("app-start");
    let state = AppState::new(program.path().to_path_buf());
    let info = start_information(&state).unwrap();
    assert!(info.data_folder.is_none() && info.workspace.is_none());
    assert!(
        program.path().join(WORKFLOWS_FILE).is_file(),
        "the bundled rules"
    );

    let dir = use_folder(&state, &program.path().join(DEFAULT_DATA_DIR)).unwrap();
    let written = fs::read_to_string(program.path().join(SETTINGS_FILE)).unwrap();
    assert!(written.contains("data_dir = \"packing-data\""), "{written}");
    assert_eq!(new_workspace(&state, "Toko Contoh").unwrap(), "Toko Contoh");
    assert!(Path::new(&dir).join("Toko Contoh").is_dir());
    assert!(
        new_workspace(&state, "toko contoh").is_err(),
        "a name differs by more than case"
    );

    let second = AppState::new(program.path().to_path_buf());
    let held = start_information(&second).err().unwrap_or_default();
    assert!(
        held.contains("in use"),
        "the first copy holds the folder: {held}"
    );
    drop(state);
    let info = start_information(&second).unwrap();
    assert_eq!(info.workspace.as_deref(), Some("Toko Contoh"));
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
    let written = fs::read_to_string(program.path().join(SETTINGS_FILE)).unwrap();
    assert!(written.contains("theme = \"light\""), "{written}");
}
