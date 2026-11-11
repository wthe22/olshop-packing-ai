//! The shop prompt over a read's copies: each copy's account from its label, a chosen account or
//! its amended record; the questions about the rest; the names an answer may give.

use std::collections::BTreeMap;

use packing_engine::account_questions::{
    AccountChoices, AccountQuestions, NoSenderQuestion, ShopAnswerError, ShopQuestion,
    account_questions, assign_accounts, check_shop_answer,
};
use packing_engine::accounts::{Account, NameError, OrderAccount};
use packing_engine::copies::Copy;
use packing_engine::ids::OrderId;
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
    assign_accounts(&mut copies, &accounts, &AccountChoices::default());
    let given: Vec<_> = copies.iter().map(|copy| copy.account.clone()).collect();
    assert_eq!(
        given,
        [
            found("Toko Alfa", Some("Toko Alfa Official")),
            None,
            None,
            None,
            None,
            found("Toko Beta", Some("Toko Beta")),
            None,
            None,
        ]
    );
    assert_eq!(
        account_questions(&copies),
        AccountQuestions {
            shops: vec![ShopQuestion {
                name: "Tok o Gamma".into(),
                orders: 2,
                inputs: vec![0, 1],
            }],
            no_sender: vec![
                NoSenderQuestion {
                    input: 0,
                    orders: 1,
                    of_orders: 3,
                },
                NoSenderQuestion {
                    input: 1,
                    orders: 2,
                    of_orders: 4,
                },
            ],
        },
        "one row per name read, spaces and a stuck digit aside; one per file with no sender"
    );
}

#[test]
fn the_answers_give_every_copy_an_account() {
    let mut accounts = vec![
        account("Toko Alfa", &["Toko Alfa", "Toko Alfa Official"]),
        account("Toko Beta", &["Toko Beta"]),
    ];
    let mut copies = read();
    assign_accounts(&mut copies, &accounts, &AccountChoices::default());

    assert_eq!(
        check_shop_answer(&copies, "Toko Gamma", "Toko Gama", &accounts),
        Err(ShopAnswerError::NotOnLabels)
    );
    assert_eq!(
        ShopAnswerError::NotOnLabels.to_string(),
        "not on these labels"
    );
    assert_eq!(
        check_shop_answer(&copies, "Toko Gamma", "Toko Alfa", &accounts),
        Err(ShopAnswerError::NotOnLabels),
        "not on the labels of that sender"
    );
    assert_eq!(
        check_shop_answer(&copies, "Tok o Gamma", "Toko Gamma", &accounts),
        Ok(())
    );
    accounts.push(account("Toko Gamma", &["Toko Gamma"]));
    assert_eq!(
        check_shop_answer(&copies, "Tok o Gamma", "toko gamma", &accounts),
        Err(ShopAnswerError::Name(NameError::ShopListed {
            account: "Toko Gamma".into()
        }))
    );
    let listed = ShopAnswerError::Name(NameError::ShopListed {
        account: "Toko Gamma".into(),
    });
    assert_eq!(listed.to_string(), "Toko Gamma already lists that shop");

    let choices = AccountChoices {
        no_sender: BTreeMap::from([(0, "Toko Alfa".to_string())]),
        recorded: BTreeMap::from([(OrderId::new("580000000000000006"), "Toko Beta".to_string())]),
    };
    assign_accounts(&mut copies, &accounts, &choices);
    assert_eq!(copies[1].account, found("Toko Gamma", Some("Toko Gamma")));
    assert_eq!(copies[2].account, found("Toko Alfa", None));
    assert_eq!(
        copies[6].account,
        found("Toko Beta", None),
        "an amend keeps the recorded account of a label with no sender"
    );
    assert_eq!(
        account_questions(&copies),
        AccountQuestions {
            shops: Vec::new(),
            no_sender: vec![NoSenderQuestion {
                input: 1,
                orders: 1,
                of_orders: 4,
            }],
        }
    );
}
