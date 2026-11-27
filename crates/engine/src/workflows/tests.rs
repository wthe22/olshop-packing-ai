use super::*;

#[test]
fn the_bundled_copy_holds_eight_couriers_and_one_rule() {
    let workflows = load_bundled();
    let codes: Vec<&str> = workflows.couriers.iter().map(|c| c.code.as_str()).collect();
    assert_eq!(
        codes,
        [
            "jnt-express",
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
            (
                "grab",
                ["Sameday".to_string(), "Same-day".to_string()].as_slice()
            )
        ]
    );
    assert_eq!(workflows.not_courier_logos.len(), 5);

    let rule = workflows.sort_rule("default").expect("rule default");
    assert_eq!(rule.groups.len(), 1);
    assert!(rule.groups[0].filter.is_none());
    assert_eq!(rule.snapshot().label_groups[0].code, "Z");
    assert_eq!(
        workflows
            .courier(&CourierCode::new("jnt-express"))
            .unwrap()
            .name,
        "J&T Express"
    );
}

/// A parser with one courier, which the config texts below name in their filters.
const PARSER_HEAD: &str = "schema = \"1.0\"
[[couriers]]
code = \"jnt\"
name = \"J&T Express\"
";
const CONFIG_HEAD: &str = "schema = \"1.0\"
";

fn with_rule(groups: &str) -> String {
    format!(
        "{CONFIG_HEAD}[[label_sort_rules]]
name = \"r\"
{groups}"
    )
}

fn loaded(parser: &str, config: &str) -> Workflows {
    load_workflows(parser, config).unwrap()
}

#[test]
fn a_rule_file_with_filters_reads_its_groups_in_order() {
    let text = with_rule(
        "[[label_sort_rules.label_groups]]\ncode = \"A\"\nname = \"Sepatu\"\nfilter = 'name contains \"sepatu\"'\n\
         [[label_sort_rules.label_groups]]\ncode = \"B\"\nname = \"J&T\"\nfilter = '''\ncourier equals \"jnt\"\n'''\n\
         [[label_sort_rules.label_groups]]\ncode = \"Z\"\nname = \"Lainnya\"\n",
    );
    let workflows = loaded(PARSER_HEAD, &text);
    let rule = &workflows.sort_rules[0];
    let codes: Vec<&str> = rule.groups.iter().map(|g| g.code.as_str()).collect();
    assert_eq!(codes, ["A", "B", "Z"]);
    assert_eq!(
        rule.groups[1].filter.as_ref().unwrap().text,
        "courier equals \"jnt\"\n"
    );
}

#[test]
fn a_newer_minor_reads_and_its_unknown_keys_are_passed_over() {
    let parser = PARSER_HEAD.replace("1.0", "1.4") + "colour = \"red\"\n";
    let config = with_rule(
        "[[label_sort_rules.label_groups]]\ncode = \"Z\"\nname = \"Lainnya\"\nicon = \"box\"\n",
    )
    .replace("1.0", "1.4");
    let workflows = loaded(&parser, &config);
    assert_eq!(workflows.couriers[0].name, "J&T Express");
    assert_eq!(workflows.sort_rules[0].groups[0].code.as_str(), "Z");
}

#[test]
fn a_rule_is_on_unless_the_file_says_it_is_off() {
    let rest = "[[label_sort_rules.label_groups]]\ncode = \"Z\"\nname = \"Lainnya\"\n";
    let text = format!(
        "{}[[label_sort_rules]]\nname = \"promo\"\nenabled = false\n{rest}",
        with_rule(rest)
    );
    let workflows = loaded(PARSER_HEAD, &text);
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
    let cases: Vec<Case> = vec![
        (
            "not TOML",
            In::Parser,
            "schema = ".into(),
            (1, 10),
            "string",
        ),
        (
            "a newer major",
            In::Parser,
            "schema = \"2.0\"\n".into(),
            (1, 10),
            "schema 2.0 is of a newer format: this copy reads 1.x. Update this copy",
        ),
        (
            "an older major",
            In::Config,
            "schema = \"0.9\"\n".into(),
            (1, 10),
            "schema 0.9 is of an older format",
        ),
        (
            "no version",
            In::Config,
            "schema = \"1\"\n".into(),
            (1, 10),
            "is not a version (X.Y)",
        ),
        (
            "a bad courier code",
            In::Parser,
            "schema = \"1.0\"\n[[couriers]]\ncode = \"J&T\"\nname = \"x\"\n".into(),
            (3, 8),
            "lower-case",
        ),
        (
            "a courier twice",
            In::Parser,
            format!("{PARSER_HEAD}[[couriers]]\ncode = \"jnt\"\nname = \"again\"\n"),
            (6, 8),
            "twice",
        ),
        (
            "a classifier naming no courier",
            In::Parser,
            format!("{PARSER_HEAD}[[courier_classifier]]\ncourier_code = \"sicepat\"\n"),
            (6, 16),
            "names no courier",
        ),
        (
            "a bad hash",
            In::Parser,
            format!(
                "{PARSER_HEAD}[[courier_classifier]]\ncourier_code = \"jnt\"\nlogos = [\"39B9\"]\n"
            ),
            (7, 10),
            "16 lower-case hex",
        ),
        (
            "a bad tracking form",
            In::Parser,
            format!(
                "{PARSER_HEAD}[[courier_classifier]]\ncourier_code = \"jnt\"\ntracking_id = 'J[XY'\n"
            ),
            (7, 15),
            "not a regex",
        ),
        (
            "every sort rule off",
            In::Config,
            with_rule(&format!("enabled = false\n{rest}")),
            (3, 8),
            "every sort rule is off",
        ),
        (
            "a group without a filter before the last",
            In::Config,
            with_rule(&format!("{}{rest}", group("A", "Sepatu", ""))),
            (5, 8),
            "not the last",
        ),
        (
            "a group code twice",
            In::Config,
            with_rule(&format!(
                "{}{}",
                group("Z", "One", "filter = 'name contains \"a\"'\n"),
                group("z", "Two", "")
            )),
            (9, 8),
            "twice",
        ),
        (
            "a bad group code",
            In::Config,
            with_rule(&group("ABCD", "Long", "")),
            (5, 8),
            "1 to 3",
        ),
        (
            "a filter error, one line",
            In::Config,
            with_rule(&format!(
                "{}{rest}",
                group("A", "Sepatu", "filter = 'name has \"x\"'\n")
            )),
            (7, 16),
            "expected a text operator",
        ),
        (
            "a filter error, on its second line",
            In::Config,
            with_rule(&format!(
                "{}{rest}",
                group(
                    "A",
                    "Sepatu",
                    "filter = '''\nname contains \"a\"\nand ship_by < \"14:00\"\n'''\n"
                )
            )),
            (9, 15),
            "expected a date",
        ),
        (
            "a CSV field in a filter",
            In::Config,
            with_rule(&format!(
                "{}{rest}",
                group("A", "Sepatu", "filter = 'sku_id equals \"1\"'\n")
            )),
            (7, 11),
            "unknown field",
        ),
        (
            "a courier filter naming no courier",
            In::Config,
            with_rule(&format!(
                "{}{rest}",
                group("A", "Sepatu", "filter = 'courier equals \"J&T Express\"'\n")
            )),
            (7, 11),
            "names no courier",
        ),
    ];
    for (what, which, text, (line, col), fragment) in cases {
        let (file, result) = match which {
            In::Parser => (PARSER_FILE, load_workflows(&text, BUNDLED_CONFIG)),
            In::Config => (CONFIG_FILE, load_workflows(PARSER_HEAD, &text)),
        };
        let error = result.expect_err(what);
        assert_eq!((error.line, error.col), (line, col), "{what}: {error}");
        assert!(error.message.contains(fragment), "{what}: {error}");
        assert!(
            error.to_string().starts_with(&format!("{file}, line ")),
            "{what}: {error}"
        );
    }
}

/// What a case of the test above holds: its name, the file it fails in, the text, the line and
/// column of the error and a fragment of its message.
type Case<'a> = (&'a str, In, String, (usize, usize), &'a str);

/// The file a check fails in.
enum In {
    Parser,
    Config,
}

#[test]
fn accounts_are_read_and_checked_like_the_screens_check_them() {
    let rest = with_rule("[[label_sort_rules.label_groups]]\ncode = \"Z\"\nname = \"Lainnya\"\n");
    let rest = rest.strip_prefix(CONFIG_HEAD).unwrap();
    let config = |accounts: &str| format!("{CONFIG_HEAD}{accounts}{rest}");
    let listed = config(
        "[[accounts]]\nname = \"Toko Alfa\"\nshops = [\" Toko Alfa \", \"Toko Alfa Official\"]\n\
         [[accounts]]\nname = \"Toko Beta\"\n",
    );
    let workflows = loaded(PARSER_HEAD, &listed);
    let accounts: Vec<(&str, &[String])> = workflows
        .accounts
        .iter()
        .map(|account| (account.name.as_str(), account.shops.as_slice()))
        .collect();
    assert_eq!(
        accounts,
        [
            (
                "Toko Alfa",
                ["Toko Alfa".to_string(), "Toko Alfa Official".to_string()].as_slice()
            ),
            ("Toko Beta", [].as_slice())
        ]
    );
    let refused = [
        (
            "an account twice",
            "[[accounts]]\nname = \"A\"\n[[accounts]]\nname = \"a\"\n",
            (5, 8),
        ),
        (
            "a shop of two accounts",
            "[[accounts]]\nname = \"A\"\nshops = [\"Toko\"]\n[[accounts]]\nname = \"B\"\nshops = [\"toko\"]\n",
            (7, 10),
        ),
        (
            "a bad account name",
            "[[accounts]]\nname = \"A/B\"\n",
            (3, 8),
        ),
    ];
    for (what, accounts, (line, col)) in refused {
        let error = load_workflows(PARSER_HEAD, &config(accounts)).expect_err(what);
        assert_eq!((error.line, error.col), (line, col), "{what}: {error}");
    }
}

#[test]
fn the_spx_forms_read_both_layouts_and_nothing_else() {
    let workflows = load_bundled();
    let spx = workflows
        .classifiers
        .iter()
        .find(|c| c.courier.as_str() == "spx")
        .expect("an SPX classifier");
    let form = spx.tracking_id.as_ref().unwrap();
    assert!(form.matches("SPXID00000000001A"));
    assert!(form.matches("CM00000000001"));
    assert!(!form.matches("CM0000000001"));
    assert!(!form.matches("CM00000000001X"));
    assert!(!form.matches("SPXID00000000001"));
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

#[test]
fn a_config_without_a_sort_rule_stands_in_with_the_built_in_one() {
    let workflows = loaded(PARSER_HEAD, CONFIG_HEAD);
    let rule = workflows.sort_rule("default").expect("the built-in rule");
    assert!(rule.enabled && rule.groups.len() == 1);
    assert_eq!(rule.groups[0].code, "Z");
    assert!(rule.groups[0].filter.is_none());
}

#[test]
fn a_key_no_reader_knows_is_named_unless_the_file_is_newer() {
    let config = with_rule(
        "[[label_sort_rules.label_groups]]\ncode = \"Z\"\nname = \"Lainnya\"\nicon = \"box\"\n",
    );
    let parser = format!("{PARSER_HEAD}colour = \"red\"\n");
    let found: Vec<String> = unknown_rules_keys(&parser, &config)
        .iter()
        .map(|(file, key)| format!("{file} {key}"))
        .collect();
    assert_eq!(
        found,
        [
            "parser.toml line 5: couriers[0].colour",
            "config.toml line 7: label_sort_rules[0].label_groups[0].icon",
        ]
    );
    let newer = config.replace("1.0", "1.4");
    assert_eq!(unknown_rules_keys(PARSER_HEAD, &newer), []);
    assert!(of_a_newer_version(&newer) && !of_a_newer_version(&config));
}

#[test]
fn an_account_keeps_the_keys_this_version_does_not_know_through_a_rename() {
    let config = format!(
        "{CONFIG_HEAD}[[accounts]]\nname = \"Toko Alfa\"\nshops = [\"Alfa\"]\ncolour = \"red\"\n\n\
         [[accounts]]\n# the second\nname = \"Toko Beta\"\nshops = []\nicon = \"b\"\n"
    );
    let account = |name: &str, shops: &[&str]| Account {
        name: name.into(),
        shops: shops.iter().map(|shop| shop.to_string()).collect(),
    };
    let renamed = [
        account("Toko Alfa Jaya", &["Alfa"]),
        account("Toko Beta", &["Beta"]),
    ];
    let (written, _) = write_accounts(PARSER_HEAD, &config, &renamed).unwrap();
    assert!(
        written.contains("name = \"Toko Alfa Jaya\"\nshops = [\"Alfa\"]\ncolour = \"red\""),
        "{written}"
    );
    assert!(
        written.contains("# the second\nname = \"Toko Beta\"\nshops = [\"Beta\"]\nicon = \"b\""),
        "{written}"
    );

    let added = [
        account("Toko Alfa", &["Alfa"]),
        account("Toko Beta", &[]),
        account("Toko Gama", &[]),
    ];
    let (written, _) = write_accounts(PARSER_HEAD, &config, &added).unwrap();
    assert_eq!(
        written.matches("colour").count() + written.matches("icon").count(),
        2,
        "{written}"
    );
    let deleted = [account("Toko Beta", &[])];
    let (written, _) = write_accounts(PARSER_HEAD, &config, &deleted).unwrap();
    assert!(
        !written.contains("colour") && written.contains("icon"),
        "{written}"
    );
}
