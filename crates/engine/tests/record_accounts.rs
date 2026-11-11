//! A batch record by account: the orders and units per account over batches, most orders first,
//! and one account's part of a batch with the batch's inputs and left-out copies.

use packing_engine::ids::OrderId;
use packing_engine::record::{AccountCount, Finding, FindingKind, account_counts, of_account};
use testkit::batch;

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
    first.orders[2].account = "Toko Beta".into();
    let mut second = batch(2, false, Vec::new());
    for order in &mut second.orders {
        order.account = "Toko Beta".into();
    }
    let units = |record: &packing_engine::record::BatchSaved, at: usize| {
        packing_engine::record::order_units(record, &record.orders[at])
    };
    let alfa = units(&first, 0) + units(&first, 1);
    let beta = units(&first, 2) + (0..3).map(|at| units(&second, at)).sum::<u32>();
    assert_eq!(
        account_counts([&first, &second]),
        [count("Toko Beta", 4, beta), count("Toko Alfa", 2, alfa)]
    );
    assert_eq!(account_counts([&first])[0].account, "Toko Alfa");
}

#[test]
fn an_accounts_part_keeps_its_orders_groups_and_findings_and_the_batchs_ledger() {
    let mut record = batch(1, false, Vec::new());
    let beta = record.orders.len() - 1;
    record.orders[beta].account = "Toko Beta".into();
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

    let mine = of_account(&record, "Toko Beta");
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
