use super::*;

#[test]
fn the_bundled_copy_holds_eight_couriers_and_one_rule() {
    let workflows = load_workflows(BUNDLED_WORKFLOWS).expect("the bundled copy is valid");
    let codes: Vec<&str> = workflows.couriers.iter().map(|c| c.code.as_str()).collect();
    assert_eq!(
        codes,
        [
            "jnt",
            "sicepat",
            "jnt-cargo",
            "idx",
            "spx",
            "gojek",
            "grab",
            "anteraja"
        ]
    );
    assert_eq!(workflows.classifiers.len(), 8);
    let jnt = &workflows.classifiers[0];
    assert!(jnt.tracking_id.as_ref().unwrap().matches("JY0000001234"));
    assert!(!jnt.tracking_id.as_ref().unwrap().matches("JY0000001234X"));
    let tags: Vec<(&str, &[String])> = workflows
        .classifiers
        .iter()
        .filter(|c| !c.instant_tags.is_empty())
        .map(|c| (c.courier.as_str(), c.instant_tags.as_slice()))
        .collect();
    assert_eq!(
        tags,
        [
            ("gojek", ["GKS".to_string()].as_slice()),
            ("grab", ["Sameday".to_string()].as_slice())
        ]
    );
    assert_eq!(workflows.not_courier_logos.len(), 5);

    let rule = workflows.sort_rule("default").expect("rule default");
    assert_eq!(rule.groups.len(), 1);
    assert!(rule.groups[0].filter.is_none());
    assert_eq!(rule.snapshot().label_groups[0].code, "Z");
    assert_eq!(
        workflows.courier(&CourierCode::new("jnt")).unwrap().name,
        "J&T Express"
    );
}

const HEAD: &str = "schema = \"1.0\"\n[[couriers]]\ncode = \"jnt\"\nname = \"J&T Express\"\n";

fn with_rule(groups: &str) -> String {
    format!("{HEAD}[[label_sort_rules]]\nname = \"r\"\n{groups}")
}

#[test]
fn a_rule_file_with_filters_reads_its_groups_in_order() {
    let text = with_rule(
        "[[label_sort_rules.label_groups]]\ncode = \"A\"\nname = \"Sepatu\"\nfilter = 'name contains \"sepatu\"'\n\
         [[label_sort_rules.label_groups]]\ncode = \"B\"\nname = \"J&T\"\nfilter = '''\ncourier equals \"jnt\"\n'''\n\
         [[label_sort_rules.label_groups]]\ncode = \"Z\"\nname = \"Lainnya\"\n",
    );
    let workflows = load_workflows(&text).unwrap();
    let rule = &workflows.sort_rules[0];
    let codes: Vec<&str> = rule.groups.iter().map(|g| g.code.as_str()).collect();
    assert_eq!(codes, ["A", "B", "Z"]);
    assert_eq!(
        rule.groups[1].filter.as_ref().unwrap().text,
        "courier equals \"jnt\"\n"
    );
}

#[test]
fn a_rule_is_on_unless_the_file_says_it_is_off() {
    let rest = "[[label_sort_rules.label_groups]]\ncode = \"Z\"\nname = \"Lainnya\"\n";
    let text = format!(
        "{}[[label_sort_rules]]\nname = \"promo\"\nenabled = false\n{rest}",
        with_rule(rest)
    );
    let workflows = load_workflows(&text).unwrap();
    let on: Vec<(&str, bool)> = workflows
        .sort_rules
        .iter()
        .map(|rule| (rule.name.as_str(), rule.enabled))
        .collect();
    assert_eq!(on, [("r", true), ("promo", false)]);
}

#[test]
fn every_check_names_its_line_and_column() {
    let group = |code: &str, name: &str, filter: &str| {
        format!("[[label_sort_rules.label_groups]]\ncode = \"{code}\"\nname = \"{name}\"\n{filter}")
    };
    let rest = group("Z", "Lainnya", "");
    let cases: Vec<(&str, String, (usize, usize), &str)> = vec![
        ("not TOML", "schema = ".into(), (1, 10), "string"),
        (
            "an unknown key",
            format!("{HEAD}colour = 1\n"),
            (5, 1),
            "unknown field",
        ),
        (
            "another schema",
            "schema = \"2.0\"\n".into(),
            (1, 10),
            "schema 2.0",
        ),
        (
            "a bad courier code",
            "schema = \"1.0\"\n[[couriers]]\ncode = \"J&T\"\nname = \"x\"\n".into(),
            (3, 8),
            "lower-case",
        ),
        (
            "a courier twice",
            format!("{HEAD}[[couriers]]\ncode = \"jnt\"\nname = \"again\"\n"),
            (6, 8),
            "twice",
        ),
        (
            "a classifier naming no courier",
            format!("{HEAD}[[courier_classifier]]\ncourier_code = \"sicepat\"\n"),
            (6, 16),
            "names no courier",
        ),
        (
            "a bad hash",
            format!("{HEAD}[[courier_classifier]]\ncourier_code = \"jnt\"\nlogos = [\"39B9\"]\n"),
            (7, 10),
            "16 lower-case hex",
        ),
        (
            "a bad tracking form",
            format!("{HEAD}[[courier_classifier]]\ncourier_code = \"jnt\"\ntracking_id = 'J[XY'\n"),
            (7, 15),
            "not a regex",
        ),
        ("no sort rule", HEAD.to_string(), (1, 1), "no sort rule"),
        (
            "every sort rule off",
            with_rule(&format!("enabled = false\n{rest}")),
            (6, 8),
            "every sort rule is off",
        ),
        (
            "a group without a filter before the last",
            with_rule(&format!("{}{rest}", group("A", "Sepatu", ""))),
            (8, 8),
            "not the last",
        ),
        (
            "a group code twice",
            with_rule(&format!(
                "{}{}",
                group("Z", "One", "filter = 'name contains \"a\"'\n"),
                group("z", "Two", "")
            )),
            (12, 8),
            "twice",
        ),
        (
            "a bad group code",
            with_rule(&group("ABCD", "Long", "")),
            (8, 8),
            "1 to 3",
        ),
        (
            "a filter error, one line",
            with_rule(&format!(
                "{}{rest}",
                group("A", "Sepatu", "filter = 'name has \"x\"'\n")
            )),
            (10, 16),
            "expected a text operator",
        ),
        (
            "a filter error, on its second line",
            with_rule(&format!(
                "{}{rest}",
                group(
                    "A",
                    "Sepatu",
                    "filter = '''\nname contains \"a\"\nand ship_by < \"14:00\"\n'''\n"
                )
            )),
            (12, 15),
            "expected a date",
        ),
        (
            "a CSV field in a filter",
            with_rule(&format!(
                "{}{rest}",
                group("A", "Sepatu", "filter = 'sku_id equals \"1\"'\n")
            )),
            (10, 11),
            "unknown field",
        ),
        (
            "a courier filter naming no courier",
            with_rule(&format!(
                "{}{rest}",
                group("A", "Sepatu", "filter = 'courier equals \"J&T Express\"'\n")
            )),
            (10, 11),
            "names no courier",
        ),
    ];
    for (what, text, (line, col), fragment) in cases {
        let error = load_workflows(&text).expect_err(what);
        assert_eq!((error.line, error.col), (line, col), "{what}: {error}");
        assert!(error.message.contains(fragment), "{what}: {error}");
        assert!(
            error
                .to_string()
                .starts_with("packing-workflows.toml, line "),
            "{what}"
        );
    }
}

#[test]
fn the_design_examples_are_filters() {
    let examples = [
        "name contains \"sepatu\"",
        "total_quantity = 1",
        "tracking_id starts_with \"JY\"",
        "courier equals \"jnt\"",
        "ship_by < \"2026-10-08\"",
        "not name contains \"sepatu\" and (name contains \"spion\" or name contains \"knalpot\")",
        "display_name equals \"Spion Beat — Standard, honda\" and line_quantity >= 2",
        "distinct_items > 1 and order_id equals \"580000000000000001\" and seller_sku contains \"SEL\"",
        "variation equals \"honda\"",
        "courier is_empty or tracking_id is_empty",
    ];
    for example in examples {
        assert!(crate::rules::parse_filter(example).is_ok(), "{example}");
    }
    for refused in [
        "ship_by < \"14:00\"",
        "paid_time < \"2026-10-08\"",
        "category equals \"A\"",
        "total_quantity is_empty",
        "courier is_empty \"jnt\"",
    ] {
        assert!(crate::rules::parse_filter(refused).is_err(), "{refused}");
    }
}
