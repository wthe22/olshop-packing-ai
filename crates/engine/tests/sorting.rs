//! The plan (03-plan.md steps 9–10, groups.md): label groups by first match and the rest, order
//! contents and the printing order, the file names; and the filter fields a v0.2 filter reads.

use std::fs;

use jiff::civil::date;
use packing_engine::copies::Copy;
use packing_engine::ids::{BatchId, CourierCode, TrackingId};
use packing_engine::rules::{evaluate_filter, parse_filter};
use packing_engine::sorting::{
    FileNameError, PlanGroup, REST_CODE, check_file_names, content_of, default_file_name,
    plan_groups, sort_batch, suggested_file,
};
use packing_engine::workflows::{Filter, load_workflows};
use serde_json::Value;
use testkit::{copy, slip_line as line, testdata};

/// The fixture's orders of `labels-slip.pdf`, as copies, from its answer.
fn fixture_copies() -> Vec<Copy> {
    let text = fs::read_to_string(testdata("expected-labels.json")).unwrap();
    // The answer's keys in page order; a JSON map read back would sort them.
    let key = regex::Regex::new(r#"(?m)^  "(\d+)": \{"#).unwrap();
    let ids: Vec<String> = key.captures_iter(&text).map(|c| c[1].to_string()).collect();
    let answer: Value = serde_json::from_str(&text).unwrap();
    ids.into_iter()
        .filter(|id| answer[id]["has_slip"] == true)
        .enumerate()
        .map(|(page, id)| {
            let want = &answer[&id];
            let lines = want["lines"]
                .as_array()
                .unwrap()
                .iter()
                .map(|l| {
                    line(
                        l[0].as_str().unwrap(),
                        l[1].as_str().unwrap(),
                        l[2].as_str().unwrap(),
                        l[3].as_i64().unwrap(),
                    )
                })
                .collect();
            let mut copy = copy(&id, 0, page as u16 + 1, lines);
            copy.courier = match want["courier"].as_str().unwrap() {
                "J&T Express" => Some(CourierCode::new("jnt")),
                "IDX" => Some(CourierCode::new("idx")),
                _ => None,
            };
            copy.tracking_id = want["tracking_id"].as_str().map(TrackingId::new);
            copy.total_quantity = want["qty_total"].as_i64();
            copy
        })
        .collect()
}

/// The groups of `testdata/categories.toml` written as a v0.2 sort rule.
const FIXTURE_RULE: &str = r#"
schema = "1.0"
[[couriers]]
code = "jnt"
name = "J&T Express"
[[couriers]]
code = "idx"
name = "IDX"
[[label_sort_rules]]
name = "fixture"
[[label_sort_rules.label_groups]]
code = "A"
name = "Sepatu"
filter = 'name contains "sepatu"'
[[label_sort_rules.label_groups]]
code = "B"
name = "Spion & Knalpot"
filter = '''
not name contains "sepatu"
and (name contains "spion" or name contains "knalpot")
'''
[[label_sort_rules.label_groups]]
code = "C"
name = "Banyak unit"
filter = 'total_quantity >= 3'
[[label_sort_rules.label_groups]]
code = "Z"
name = "J&T"
filter = 'courier equals "jnt"'
"#;

#[test]
fn the_fixture_sorts_into_the_groups_and_order_contents_of_its_answer() {
    let copies = fixture_copies();
    assert_eq!(copies.len(), 23);
    let workflows = load_workflows(FIXTURE_RULE).unwrap();
    let plan = plan_groups(&workflows.sort_rules[0]);
    let kept: Vec<usize> = (0..copies.len()).collect();
    let sorted = sort_batch(&copies, &kept, &plan, BatchId::new(1));

    let answer: Value =
        serde_json::from_str(&fs::read_to_string(testdata("expected-picks.json")).unwrap())
            .unwrap();
    let answer = answer.as_array().unwrap();
    assert_eq!(sorted.groups.len(), answer.len());
    for (group, want) in sorted.groups.iter().zip(answer) {
        assert_eq!(group.code, want["code"].as_str().unwrap());
        assert_eq!(group.order_count() as u64, want["orders"].as_u64().unwrap());
        let blocks: Vec<Vec<&str>> = group
            .contents
            .iter()
            .map(|block| {
                block
                    .orders
                    .iter()
                    .map(|&c| copies[c].order_id.as_str())
                    .collect()
            })
            .collect();
        let want_blocks: Vec<Vec<&str>> = want["runs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|run| {
                run.as_array()
                    .unwrap()
                    .iter()
                    .map(|id| id.as_str().unwrap())
                    .collect()
            })
            .collect();
        assert_eq!(blocks, want_blocks, "group {}", group.code);
    }
    let files: Vec<&str> = sorted.groups.iter().map(|g| g.file.as_str()).collect();
    assert_eq!(
        files,
        [
            "1-A Sepatu x7.pdf",
            "1-B Spion Knalpot x8.pdf",
            "1-C Banyak unit x4.pdf",
            "1-Z J T x2.pdf",
            "1 Uncategorised x2.pdf",
        ]
    );
    assert_eq!(sorted.groups.last().unwrap().code, REST_CODE);
    assert_eq!(sorted.printing_order().len(), 23);
    assert_eq!(sorted.summary_file(), "1 summary.pdf");
}

#[test]
fn a_group_that_takes_nothing_has_no_file_and_a_last_group_without_filter_leaves_no_rest() {
    let copies = vec![
        copy("580000000000000001", 0, 1, vec![line("Sepatu", "", "", 1)]),
        copy("580000000000000002", 0, 2, vec![line("Jok", "", "", 1)]),
    ];
    let plan = vec![
        PlanGroup {
            code: "A".into(),
            name: "Spion".into(),
            filter: Some(Filter {
                text: "name contains \"spion\"".into(),
                condition: parse_filter("name contains \"spion\"").unwrap(),
            }),
            file_name: None,
        },
        PlanGroup {
            code: "Z".into(),
            name: "Lainnya".into(),
            filter: None,
            file_name: Some("Semua hari ini".into()),
        },
    ];
    let sorted = sort_batch(&copies, &[0, 1], &plan, BatchId::new(2));
    assert_eq!(sorted.groups.len(), 1);
    assert_eq!(sorted.groups[0].code, "Z");
    assert_eq!(
        sorted.groups[0].file, "Semua hari ini.pdf",
        "the owner's name"
    );
    // One order each: equal in size, so by content as text — Jok before Sepatu.
    assert_eq!(sorted.printing_order(), [(0, 1), (0, 0)]);
}

#[test]
fn an_order_content_leaves_out_the_seller_sku_and_keeps_it_beside() {
    let one = copy(
        "580000000000000001",
        0,
        1,
        vec![line("Spion", "honda", "S-2", 1), line("Knalpot", "", "", 2)],
    );
    let other_sku = copy(
        "580000000000000002",
        0,
        2,
        vec![
            line("Knalpot", "", "K-9", 2),
            line("Spion", "honda", "S-1", 1),
        ],
    );
    let (content, skus) = content_of(&one);
    let names: Vec<&str> = content.items.iter().map(|i| i.name.as_str()).collect();
    assert_eq!(
        names,
        ["Knalpot", "Spion"],
        "sorted by name, variant, quantity"
    );
    assert_eq!(
        skus,
        [None, Some("S-2".to_string())],
        "aligned with the sorted items"
    );
    assert_eq!(
        content_of(&other_sku).0,
        content,
        "a Seller SKU does not split a content"
    );
}

#[test]
fn file_names_follow_the_default_rule_and_the_windows_rules() {
    let cases = [
        ("2-B Spion & Knalpot x35", "2-B Spion Knalpot x35"),
        ("  a  b  ", "a b"),
        ("Jok/Kulit: Vario", "Jok Kulit Vario"),
        ("Sepatu_ukuran-L", "Sepatu_ukuran-L"),
    ];
    for (text, want) in cases {
        assert_eq!(default_file_name(text), want, "{text}");
    }
    assert_eq!(default_file_name(&"x".repeat(150)).len(), 100);
    assert_eq!(
        suggested_file(BatchId::new(3), "?", "Uncategorised", 12),
        "3 Uncategorised x12.pdf"
    );

    let names = |list: &[&str]| list.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    assert!(check_file_names(&names(&["1-A Sepatu x2.pdf", "1 summary.pdf"])).is_ok());
    for (list, error) in [
        (
            vec!["a.pdf", "A.PDF"],
            FileNameError::Twice {
                name: "A.PDF".into(),
            },
        ),
        (
            vec!["a?b.pdf"],
            FileNameError::NotAllowed {
                name: "a?b.pdf".into(),
            },
        ),
        (
            vec!["CON.pdf"],
            FileNameError::NotAllowed {
                name: "CON.pdf".into(),
            },
        ),
        (
            vec![" .pdf"],
            FileNameError::NotAllowed {
                name: " .pdf".into(),
            },
        ),
    ] {
        assert_eq!(
            check_file_names(&names(&list)),
            Err(error.clone()),
            "{list:?}"
        );
        assert!(!error.to_string().is_empty());
    }
}

#[test]
fn each_filter_field_reads_its_value_of_the_order() {
    let mut order = copy(
        "580000000000000042",
        0,
        1,
        vec![
            line("Spion Beat | Kaca Spion", "Standard, honda", "SPI-01", 2),
            line("Knalpot", "", "", 1),
        ],
    );
    order.courier = Some(CourierCode::new("jnt"));
    order.tracking_id = Some(TrackingId::new("JY0000001234"));
    order.total_quantity = Some(3);
    order.ship_by_date = Some(date(2026, 10, 8));
    let bare = copy("580000000000000043", 0, 2, vec![line("Jok", "", "", 1)]);
    let cases = [
        ("name contains \"KACA\"", true, false),
        (
            "display_name equals \"Spion Beat — Standard, honda\"",
            true,
            false,
        ),
        ("variation equals \"standard, honda\"", true, false),
        ("seller_sku starts_with \"SPI\"", true, false),
        ("line_quantity >= 2", true, false),
        ("total_quantity = 3", true, false),
        ("distinct_items = 2", true, false),
        ("courier equals \"jnt\"", true, false),
        ("courier equals \"\"", false, true),
        ("order_id equals \"580000000000000042\"", true, false),
        ("tracking_id starts_with \"JY\"", true, false),
        ("ship_by < \"2026-10-09\"", true, false),
        ("ship_by != \"2026-10-09\"", true, false),
        ("not name contains \"sepatu\"", true, true),
        ("name contains \"jok\" or total_quantity > 0", true, true),
    ];
    for (text, on_order, on_bare) in cases {
        let filter = parse_filter(text).expect(text);
        assert_eq!(evaluate_filter(&filter, &order), on_order, "{text}");
        assert_eq!(
            evaluate_filter(&filter, &bare),
            on_bare,
            "{text} on an order without values"
        );
    }
}
