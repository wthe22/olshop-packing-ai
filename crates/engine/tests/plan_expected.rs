//! The plan on the shared fixtures: `testdata/expected-labels.json` (orders) with
//! `testdata/categories.toml` must give `testdata/expected-picks.json` exactly — number, code,
//! name, orders and runs.

use std::fs;

use packing_engine::ids::{OrderId, PdfNumber, TrackingId};
use packing_engine::orders::{Line, Order, PageRef, ReadOrder, display_name};
use packing_engine::plan::{Plan, REST_CODE, REST_NAME};
use packing_engine::rules::load_rules;
use serde::Deserialize;
use serde::de::{MapAccess, Visitor};
use serde_json::Value;
use testkit::testdata;

/// The top-level JSON object as ordered `(key, value)` pairs, so the download order is the
/// order the keys appear in the file (`serde_json` sorts keys by default).
struct OrderedMap(Vec<(String, Value)>);

impl<'de> Deserialize<'de> for OrderedMap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct MapVisitor;
        impl<'de> Visitor<'de> for MapVisitor {
            type Value = OrderedMap;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a JSON object")
            }
            fn visit_map<A>(self, mut map: A) -> Result<OrderedMap, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut entries = Vec::new();
                while let Some(key) = map.next_key::<String>()? {
                    entries.push((key, map.next_value::<Value>()?));
                }
                Ok(OrderedMap(entries))
            }
        }
        deserializer.deserialize_map(MapVisitor)
    }
}

fn str_field(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

/// The `index`-th string of a JSON array (a slip row's cells).
fn cell(row: &Value, index: usize) -> String {
    row.get(index)
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

/// A `ReadOrder` built from one entry of `expected-labels.json`, in download (file) order.
fn read_order(order_id: &str, entry: &Value, position: usize) -> ReadOrder {
    let lines = entry
        .get("lines")
        .and_then(Value::as_array)
        .map(|rows| {
            rows.iter()
                .map(|row| {
                    let name = cell(row, 0);
                    let variation = cell(row, 1);
                    Line {
                        display_name: display_name(&name, &variation),
                        name,
                        variation,
                        seller_sku: cell(row, 2),
                        quantity: row.get(3).and_then(Value::as_i64).unwrap_or(0),
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    let ship_by = entry
        .get("ship_by")
        .and_then(Value::as_str)
        .map(|text| text.parse().expect("ISO date-time"));
    let pages = entry.get("pages").and_then(Value::as_u64).unwrap_or(0) as usize;
    let tracking_id = str_field(entry, "tracking_id");
    let courier = str_field(entry, "courier");
    let order = Order {
        order_id: OrderId::new(order_id),
        tracking_id: (!tracking_id.is_empty()).then(|| TrackingId::new(tracking_id)),
        courier: (!courier.is_empty()).then_some(courier),
        ship_by,
        lines,
        customer_message: str_field(entry, "customer_message"),
    };
    ReadOrder {
        order,
        pages: (0..pages).map(|page| PageRef { file: 0, page }).collect(),
        position,
        has_slip: entry
            .get("has_slip")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        qty_total: entry.get("qty_total").and_then(Value::as_i64),
    }
}

/// The orders of `labels-slip.pdf` (the fixture's orders that carry a slip); the plain-label
/// orders of `labels-plain.pdf` are not part of the pick flow.
fn slip_orders() -> Vec<ReadOrder> {
    let text = fs::read_to_string(testdata("expected-labels.json")).expect("read expected-labels");
    let OrderedMap(entries) = serde_json::from_str::<OrderedMap>(&text).expect("parse JSON");
    entries
        .iter()
        .enumerate()
        .filter(|(_, pair)| pair.1.get("has_slip").and_then(Value::as_bool) == Some(true))
        .map(|(index, (order_id, entry))| read_order(order_id, entry, index + 1))
        .collect()
}

#[test]
fn candidates_plan_matches_expected_categories_json() {
    let orders = slip_orders();
    assert_eq!(orders.len(), 23);

    let categories =
        load_rules(&fs::read_to_string(testdata("categories.toml")).expect("read categories"))
            .expect("load rules");
    let plan = Plan::from_categories(&categories);
    let result = plan.count(&orders, PdfNumber::new(1));

    let expected: Value = serde_json::from_str(
        &fs::read_to_string(testdata("expected-picks.json")).expect("read expected-picks"),
    )
    .expect("parse expected-picks");
    let expected = expected.as_array().expect("array");

    assert_eq!(result.categories.len(), expected.len());
    assert_eq!(result.total_orders(), orders.len());

    for (category, entry) in result.categories.iter().zip(expected) {
        assert_eq!(
            category.number,
            PdfNumber::new(entry["number"].as_i64().unwrap())
        );
        assert_eq!(category.code, entry["code"].as_str().unwrap());
        assert_eq!(category.name, entry["name"].as_str().unwrap());
        assert_eq!(
            category.orders.len(),
            entry["orders"].as_u64().unwrap() as usize
        );

        let want_runs: Vec<Vec<OrderId>> = entry["runs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|run| {
                run.as_array()
                    .unwrap()
                    .iter()
                    .map(|id| OrderId::new(id.as_str().unwrap()))
                    .collect()
            })
            .collect();
        let got_runs: Vec<Vec<OrderId>> =
            category.runs.iter().map(|run| run.orders.clone()).collect();
        assert_eq!(got_runs, want_runs, "runs of pick {}", category.number);

        // The runs are the orders, in printing order.
        let flattened: Vec<OrderId> = got_runs.into_iter().flatten().collect();
        let mut sorted_orders = category.orders.clone();
        sorted_orders.sort();
        let mut sorted_flat = flattened.clone();
        sorted_flat.sort();
        assert_eq!(sorted_flat, sorted_orders);
    }

    let rest = result.categories.last().unwrap();
    assert_eq!(rest.code, REST_CODE);
    assert_eq!(rest.name, REST_NAME);
}
