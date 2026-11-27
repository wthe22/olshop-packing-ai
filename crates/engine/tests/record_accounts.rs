//! A batch record by account: an order's account is the one that lists its shop now; the orders and
//! units per account over batches, most orders first, and one account's part of a batch with the
//! batch's inputs and left-out copies.

use packing_engine::accounts::{Account, NO_ACCOUNT, account_of_shop};
use packing_engine::ids::OrderId;
use packing_engine::record::{AccountCount, Finding, FindingKind, account_counts, of_account};
use testkit::batch;

fn account(name: &str, shops: &[&str]) -> Account {
    Account {
        name: name.into(),
        shops: shops.iter().map(|shop| shop.to_string()).collect(),
    }
}

/// Toko Alfa lists its shop; Toko Beta lists Toko Beta Official.
fn accounts() -> Vec<Account> {
    vec![
        account("Toko Alfa", &["Toko Alfa"]),
        account("Toko Beta", &["Toko Beta Official"]),
    ]
}

#[test]
fn an_orders_account_is_the_one_that_lists_its_shop_now() {
    let renamed = vec![account("Alfa Baru", &["Toko Alfa"])];
    let cases: [(&str, Option<&str>, &[Account], &str); 5] = [
        (
            "listed, spaces and case ignored",
            Some("toko  ALFA"),
            &accounts(),
            "Toko Alfa",
        ),
        (
            "renamed after the save",
            Some("Toko Alfa"),
            &renamed,
            "Alfa Baru",
        ),
        (
            "its account deleted",
            Some("Toko Beta Official"),
            &renamed,
            NO_ACCOUNT,
        ),
        (
            "a shop no account lists",
            Some("Toko Gamma"),
            &accounts(),
            NO_ACCOUNT,
        ),
        ("no sender on the label", None, &accounts(), NO_ACCOUNT),
    ];
    for (what, shop, accounts, want) in cases {
        assert_eq!(account_of_shop(shop, accounts), want, "{what}");
    }
}

fn count(account: &str, orders: usize, units: u32) -> AccountCount {
    AccountCount {
        account: account.into(),
        orders,
        units,
    }
}

#[test]
fn each_account_counts_its_orders_and_units_most_orders_first() {
    let mut first = batch(1, false, Vec::new());
    first.orders[2].shop = Some("Toko Beta Official".into());
    let mut second = batch(2, false, Vec::new());
    for order in &mut second.orders {
        order.shop = Some("Toko Beta Official".into());
    }
    second.orders[0].shop = None;
    let units = |record: &packing_engine::record::BatchSaved, at: usize| {
        packing_engine::record::order_units(record, &record.orders[at])
    };
    let alfa = units(&first, 0) + units(&first, 1);
    let beta = units(&first, 2) + (1..3).map(|at| units(&second, at)).sum::<u32>();
    let none = units(&second, 0);
    assert_eq!(
        account_counts([&first, &second], &accounts()),
        [
            count("Toko Beta", 3, beta),
            count("Toko Alfa", 2, alfa),
            count(NO_ACCOUNT, 1, none)
        ]
    );
    assert_eq!(
        account_counts([&first], &accounts())[0].account,
        "Toko Alfa"
    );
}

#[test]
fn an_accounts_part_keeps_its_orders_groups_and_findings_and_the_batchs_ledger() {
    let mut record = batch(1, false, Vec::new());
    let beta = record.orders.len() - 1;
    record.orders[beta].shop = Some("Toko Beta Official".into());
    let beta_id = record.orders[beta].order_id.clone();
    let finding = |order_id: Option<OrderId>| Finding {
        kind: FindingKind::new("courier_unknown"),
        pdf: None,
        page: None,
        order_id,
        text: "a finding".into(),
    };
    record.warnings = vec![finding(Some(beta_id.clone())), finding(None)];
    record.errors = vec![finding(Some(record.orders[0].order_id.clone()))];

    let mine = of_account(&record, "Toko Beta", &accounts());
    let ids: Vec<&OrderId> = mine.orders.iter().map(|o| &o.order_id).collect();
    assert_eq!(ids, [&beta_id]);
    let group = &record.orders[beta].group;
    let kept: Vec<Option<&String>> = mine.outputs.iter().map(|o| o.group.as_ref()).collect();
    let want: Vec<Option<&String>> = record
        .outputs
        .iter()
        .map(|o| o.group.as_ref())
        .filter(|g| g.is_none_or(|g| g == group))
        .collect();
    assert_eq!(kept, want);
    assert_eq!(mine.warnings, [finding(Some(beta_id))]);
    assert!(mine.errors.is_empty());
    assert_eq!(
        (mine.inputs, mine.left_out),
        (record.inputs, record.left_out)
    );
}
