use packing_engine::accounts::{create_account, delete_account, rename_account, set_shops};
use testkit::TempDir;

use super::*;
use crate::commands::start::{start_information, use_folder};
use crate::settings::DEFAULT_DATA_DIR;

fn started(tag: &str) -> (TempDir, AppState) {
    let program = TempDir::new(tag);
    let state = AppState::new(program.path().to_path_buf());
    start_information(&state).unwrap();
    use_folder(&state, &program.path().join(DEFAULT_DATA_DIR)).unwrap();
    (program, state)
}

#[test]
fn accounts_are_made_renamed_given_shops_and_deleted_in_the_settings_file() {
    let _guard = testkit::pdfium_lock();
    let (_program, state) = started("app-accounts");
    change(&state, |accounts| {
        create_account(accounts, "Toko Alfa").map_err(told)
    })
    .unwrap();
    change(&state, |accounts| {
        create_account(accounts, "Toko Beta").map_err(told)
    })
    .unwrap();
    let refused = change(&state, |accounts| {
        create_account(accounts, "toko alfa").map_err(told)
    });
    assert_eq!(refused, Err("Another account has that name.".to_string()));

    let shops = vec!["Toko Alfa".to_string(), "Alfa Official".to_string()];
    change(&state, |accounts| {
        set_shops(accounts, "Toko Alfa", &shops).map_err(|(_, error)| told(error))
    })
    .unwrap();
    change(&state, |accounts| {
        rename_account(accounts, "Toko Beta", "Toko Gamma").map_err(told)
    })
    .unwrap();
    let written = change(&state, |accounts| {
        delete_account(accounts, "Toko Gamma").map_err(told)
    })
    .unwrap();
    assert_eq!(written, settings::accounts(state.program()).unwrap());
    assert_eq!(written.len(), 1);
    assert_eq!(written[0].shops, shops);

    state.index().wait();
    let view = account_dates_of(&state, "Toko Alfa").unwrap();
    assert!(!view.loading && view.dates.is_empty(), "{view:?}");
}
