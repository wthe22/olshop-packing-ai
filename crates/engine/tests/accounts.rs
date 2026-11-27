//! The shop prompt over a read's copies: each copy's account from its label, No account for a label
//! with no sender; the questions about the senders no account lists.

use packing_engine::account_questions::{ShopQuestion, account_questions, assign_accounts};
use packing_engine::accounts::{Account, NO_ACCOUNT, NameError, OrderAccount, check_account_name};
use packing_engine::copies::Copy;
use packing_engine::label::find_sender;
use packing_engine::text::{PageText, Run};
use testkit::copy;

fn account(name: &str, shops: &[&str]) -> Account {
    Account {
        name: name.to_string(),
        shops: shops.iter().map(|shop| shop.to_string()).collect(),
    }
}

/// A copy of `order` in `input` whose label prints `sender` after its caption, or no caption.
fn labelled(order: &str, input: usize, sender: Option<&str>) -> Copy {
    let mut copy = copy(order, input, 1, Vec::new());
    copy.account = None;
    copy.sender = sender.and_then(|name| {
        let line = format!("Pengirim : {name}");
        find_sender(&PageText {
            text: line.clone(),
            runs: vec![Run {
                x: 10.0,
                y: 300.0,
                text: line,
            }],
            ..PageText::default()
        })
    });
    copy
}

fn found(account: &str, shop: Option<&str>) -> Option<OrderAccount> {
    Some(OrderAccount {
        account: account.into(),
        shop: shop.map(str::to_string),
    })
}

fn read() -> Vec<Copy> {
    vec![
        labelled("580000000000000001", 0, Some("Toko Alfa Official")),
        labelled("580000000000000002", 0, Some("Tok o Gamma")),
        labelled("580000000000000003", 0, None),
        labelled("580000000000000004", 1, Some("TokoGamma7")),
        labelled("580000000000000004", 1, Some("Toko Gamma")),
        labelled("580000000000000005", 1, Some("Toko Beta")),
        labelled("580000000000000006", 1, None),
        labelled("580000000000000007", 1, None),
    ]
}

#[test]
fn a_listed_shop_names_the_account_and_the_rest_is_asked_about() {
    let accounts = vec![
        account("Toko Alfa", &["Toko Alfa", "Toko Alfa Official"]),
        account("Toko Beta", &["Toko Beta"]),
    ];
    let mut copies = read();
    assign_accounts(&mut copies, &accounts);
    let given: Vec<_> = copies.iter().map(|copy| copy.account.clone()).collect();
    assert_eq!(
        given,
        [
            found("Toko Alfa", Some("Toko Alfa Official")),
            None,
            found(NO_ACCOUNT, None),
            None,
            None,
            found("Toko Beta", Some("Toko Beta")),
            found(NO_ACCOUNT, None),
            found(NO_ACCOUNT, None),
        ],
        "a label with no sender is under No account and not asked about"
    );
    assert_eq!(
        account_questions(&copies),
        [ShopQuestion {
            name: "Tok o Gamma".into(),
            orders: 2,
            inputs: vec![0, 1],
        }],
        "one row per name read, spaces and a stuck digit aside"
    );
}

#[test]
fn the_answers_give_every_copy_an_account() {
    let accounts = vec![
        account("Toko Alfa", &["Toko Alfa", "Toko Alfa Official"]),
        account("Toko Beta", &["Toko Beta"]),
        account("Toko Gamma", &["Toko Gamma"]),
    ];
    let mut copies = read();
    assign_accounts(&mut copies, &accounts);
    assert_eq!(copies[1].account, found("Toko Gamma", Some("Toko Gamma")));
    assert!(copies.iter().all(|copy| copy.account.is_some()));
    assert!(account_questions(&copies).is_empty());
}

#[test]
fn no_account_may_take_the_name_of_no_account() {
    assert_eq!(
        check_account_name("no ACCOUNT", &[], None),
        Err(NameError::AccountTaken)
    );
}
