use super::*;
use crate::label::find_sender;
use crate::text::{PageText, Run};

fn account(name: &str, shops: &[&str]) -> Account {
    Account {
        name: name.to_string(),
        shops: shops.iter().map(|shop| shop.to_string()).collect(),
    }
}

fn sender(line: &str) -> SenderText {
    let page = PageText {
        text: line.to_string(),
        runs: vec![Run {
            x: 10.0,
            y: 300.0,
            text: line.to_string(),
        }],
        ..PageText::default()
    };
    find_sender(&page).expect("a caption").text
}

#[test]
fn an_account_name_follows_the_file_name_rule_and_is_unique() {
    let accounts = vec![account("Toko Alfa", &[])];
    let cases = [
        ("Toko Beta", None, Ok(())),
        ("Toko_2-B", None, Ok(())),
        ("", None, Err(NameError::NotAName)),
        (" Toko", None, Err(NameError::NotAName)),
        ("Toko & Co", None, Err(NameError::NotAName)),
        ("Toko  Dua", None, Err(NameError::NotAName)),
        ("TOKO ALFA", None, Err(NameError::AccountTaken)),
        ("TOKO ALFA", Some("Toko Alfa"), Ok(())),
        ("labels", None, Ok(())),
    ];
    for (name, renamed, want) in cases {
        let got = check_account_name(name, &accounts, renamed);
        assert_eq!(got, want, "{name:?}");
        if let Err(error) = got {
            assert!(!error.to_string().is_empty());
        }
    }
    assert!(check_account_name(&"x".repeat(40), &accounts, None).is_ok());
    assert_eq!(
        check_account_name(&"x".repeat(41), &accounts, None),
        Err(NameError::NotAName)
    );
}

#[test]
fn a_shop_belongs_to_one_account_only() {
    let accounts = vec![
        account("Toko Alfa", &["Toko Alfa"]),
        account("Toko Beta", &["Toko Beta"]),
    ];
    assert_eq!(check_shop_name("Toko Gamma & Co", &accounts), Ok(()));
    assert_eq!(check_shop_name("  ", &accounts), Err(NameError::NotAName));
    assert_eq!(
        check_shop_name(&"x".repeat(81), &accounts),
        Err(NameError::NotAName)
    );
    let listed = check_shop_name("toko  BETA", &accounts).unwrap_err();
    assert_eq!(
        listed,
        NameError::ShopListed {
            account: "Toko Beta".into()
        }
    );
    assert!(listed.to_string().contains("Toko Beta"));
    assert!(same_shop("Tok o Alfa", "toko alfa"));
}

#[test]
fn the_longest_listed_shop_found_names_the_account() {
    let accounts = vec![
        account("Toko Alfa", &["Toko Alfa"]),
        account("Grosir", &["Toko Alfa Official", "Alfa"]),
        account("Toko Beta", &["Toko Beta"]),
    ];
    let cases = [
        (
            "Pengirim : Tok o Alfa Official 7",
            Some(("Grosir", "Toko Alfa Official")),
        ),
        (
            "Pengirim : Toko Alfa (+62)8****",
            Some(("Toko Alfa", "Toko Alfa")),
        ),
        ("Pengirim : TOKO BETA", Some(("Toko Beta", "Toko Beta"))),
        ("Pengirim : Toko Gamma", None),
    ];
    for (line, want) in cases {
        let got = account_of(&sender(line), &accounts);
        let want = want.map(|(account, shop)| OrderAccount {
            account: account.into(),
            shop: Some(shop.into()),
        });
        assert_eq!(got, want, "{line}");
    }
    let tie = vec![
        account("Pertama", &["Toko Alfa"]),
        account("Kedua", &["tokoalfa"]),
    ];
    assert_eq!(
        account_of(&sender("Pengirim : Toko Alfa"), &tie).map(|found| found.account),
        Some("Pertama".into()),
        "the same length: the one listed first"
    );
}

#[test]
fn a_shop_goes_to_a_listed_account_or_a_new_one() {
    let mut accounts = vec![account("Toko Alfa", &["Toko Alfa"])];
    let alfa = ShopAccount::Listed("Toko Alfa".into());
    add_shop(&mut accounts, " Toko Alfa Official ", &alfa).unwrap();
    add_shop(
        &mut accounts,
        "Toko Gamma",
        &ShopAccount::New("Toko Gamma".into()),
    )
    .unwrap();
    assert_eq!(
        accounts,
        [
            account("Toko Alfa", &["Toko Alfa", "Toko Alfa Official"]),
            account("Toko Gamma", &["Toko Gamma"]),
        ]
    );
    let before = accounts.clone();
    let refused = [
        (
            "toko gamma",
            alfa.clone(),
            NameError::ShopListed {
                account: "Toko Gamma".into(),
            },
        ),
        (
            "Toko Delta",
            ShopAccount::Listed("Toko Tiada".into()),
            NameError::NoAccount,
        ),
        (
            "Toko Delta",
            ShopAccount::New("toko alfa".into()),
            NameError::AccountTaken,
        ),
        (
            "Toko & Co",
            ShopAccount::New("Toko & Co".into()),
            NameError::NotAName,
        ),
    ];
    for (shop, to, error) in refused {
        assert_eq!(
            add_shop(&mut accounts, shop, &to),
            Err(error.clone()),
            "{shop}"
        );
        assert!(!error.to_string().is_empty());
    }
    assert_eq!(accounts, before, "nothing changes when a name is refused");
}

#[test]
fn an_account_is_made_renamed_given_its_shops_and_deleted() {
    let mut accounts = vec![account("Toko Alfa", &["Toko Alfa"])];
    create_account(&mut accounts, "Toko Beta").unwrap();
    assert_eq!(accounts[1], account("Toko Beta", &[]));
    assert_eq!(
        create_account(&mut accounts, "toko beta"),
        Err(NameError::AccountTaken)
    );

    rename_account(&mut accounts, "Toko Beta", "TOKO BETA").unwrap();
    assert_eq!(accounts[1].name, "TOKO BETA", "a change of case only");
    assert_eq!(
        rename_account(&mut accounts, "TOKO BETA", "Toko Alfa"),
        Err(NameError::AccountTaken)
    );
    assert_eq!(
        rename_account(&mut accounts, "Toko Gamma", "Toko Delta"),
        Err(NameError::NoAccount)
    );

    let shops = |names: &[&str]| -> Vec<String> { names.iter().map(|n| n.to_string()).collect() };
    set_shops(
        &mut accounts,
        "TOKO BETA",
        &shops(&[" Toko Beta ", "Beta Official"]),
    )
    .unwrap();
    assert_eq!(accounts[1].shops, ["Toko Beta", "Beta Official"]);
    let refused = [
        (shops(&["Toko Beta", "toko  beta"]), 1, "TOKO BETA"),
        (shops(&["Beta", "TokoAlfa"]), 1, "Toko Alfa"),
    ];
    for (names, at, account) in refused {
        assert_eq!(
            set_shops(&mut accounts, "TOKO BETA", &names),
            Err((
                at,
                NameError::ShopListed {
                    account: account.into()
                }
            ))
        );
    }
    assert_eq!(
        set_shops(&mut accounts, "TOKO BETA", &shops(&[""])),
        Err((0, NameError::NotAName))
    );
    assert_eq!(
        accounts[1].shops,
        ["Toko Beta", "Beta Official"],
        "unchanged"
    );

    delete_account(&mut accounts, "Toko Alfa").unwrap();
    assert_eq!(accounts.len(), 1);
    assert_eq!(
        delete_account(&mut accounts, "Toko Alfa"),
        Err(NameError::NoAccount)
    );
}
