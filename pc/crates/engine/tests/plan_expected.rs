//! The plan on the shared fixtures: `testdata/expected-labels.json` (orders) with
//! `testdata/categories.toml` must give `testdata/expected-picks.json` exactly — number, code,
//! name, orders and runs — the same result as the Python script (07 › *Testing*).

use std::fs;
use std::path::PathBuf;

use packing_engine::orders::{Line, Order, PageRef, ReadOrder, display_name};
use packing_engine::plan::{Plan, REST_CODE, REST_NAME};
use packing_engine::rules::load_rules;
use serde::Deserialize;
use serde::de::{MapAccess, Visitor};
use serde_json::Value;

fn testdata(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../testdata")
        .join(name)
}

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
    let order = Order {
        order_id: order_id.to_string(),
        tracking_id: str_field(entry, "tracking_id"),
        courier: str_field(entry, "courier"),
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
fn candidates_plan_matches_expected_picks_json() {
    let orders = slip_orders();
    assert_eq!(orders.len(), 23);

    let categories =
        load_rules(&fs::read_to_string(testdata("categories.toml")).expect("read categories"))
            .expect("load rules");
    let plan = Plan::from_categories(&categories);
    let result = plan.count(&orders, 1);

    let expected: Value = serde_json::from_str(
        &fs::read_to_string(testdata("expected-picks.json")).expect("read expected-picks"),
    )
    .expect("parse expected-picks");
    let expected = expected.as_array().expect("array");

    assert_eq!(result.picks.len(), expected.len());
    assert_eq!(result.total_orders(), orders.len());

    for (pick, entry) in result.picks.iter().zip(expected) {
        assert_eq!(pick.number, entry["number"].as_i64().unwrap());
        assert_eq!(pick.code, entry["code"].as_str().unwrap());
        assert_eq!(pick.name, entry["name"].as_str().unwrap());
        assert_eq!(
            pick.orders.len(),
            entry["orders"].as_u64().unwrap() as usize
        );

        let want_runs: Vec<Vec<String>> = entry["runs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|run| {
                run.as_array()
                    .unwrap()
                    .iter()
                    .map(|id| id.as_str().unwrap().to_string())
                    .collect()
            })
            .collect();
        let got_runs: Vec<Vec<String>> = pick.runs.iter().map(|run| run.orders.clone()).collect();
        assert_eq!(got_runs, want_runs, "runs of pick {}", pick.number);

        // The runs are the orders, in printing order.
        let flattened: Vec<String> = got_runs.into_iter().flatten().collect();
        let mut sorted_orders = pick.orders.clone();
        sorted_orders.sort();
        let mut sorted_flat = flattened.clone();
        sorted_flat.sort();
        assert_eq!(sorted_flat, sorted_orders);
    }

    let rest = result.picks.last().unwrap();
    assert_eq!(rest.code, REST_CODE);
    assert_eq!(rest.name, REST_NAME);
}
