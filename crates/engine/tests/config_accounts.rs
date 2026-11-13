//! The accounts written into `config.toml`: the owner's comments and the sort rules stay.

use packing_engine::accounts::Account;
use packing_engine::workflows::{BUNDLED_CONFIG, BUNDLED_PARSER, write_accounts};

fn account(name: &str, shops: &[&str]) -> Account {
    Account {
        name: name.into(),
        shops: shops.iter().map(ToString::to_string).collect(),
    }
}

#[test]
fn accounts_are_written_before_the_sort_rules_and_the_rest_of_the_file_stays() {
    let config = format!("{BUNDLED_CONFIG}\n# last word\n");
    let accounts = [
        account("Toko Alfa", &["Toko Alfa", "Toko Alfa Official"]),
        account("Toko Beta", &[]),
    ];
    let (written, workflows) = write_accounts(BUNDLED_PARSER, &config, &accounts).unwrap();
    assert_eq!(workflows.accounts, accounts);
    assert!(written.contains("# Sort rules: a batch is sorted by the rule chosen for it."));
    assert!(
        written.find("[[accounts]]").unwrap() < written.find("[[label_sort_rules]]").unwrap(),
        "{written}"
    );
    let (again, _) = write_accounts(BUNDLED_PARSER, &written, &accounts).unwrap();
    assert_eq!(again, written, "the same accounts again change nothing");

    let (none, workflows) = write_accounts(BUNDLED_PARSER, &written, &[]).unwrap();
    assert!(workflows.accounts.is_empty());
    assert!(!none.contains("[[accounts]]"));
}

#[test]
fn accounts_the_rules_refuse_are_not_written() {
    let twice = [account("A", &["Toko"]), account("B", &["toko"])];
    assert!(write_accounts(BUNDLED_PARSER, BUNDLED_CONFIG, &twice).is_err());
}
