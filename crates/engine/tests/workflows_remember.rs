//! The logo prompt's answers written into `parser.toml`: the hash lands where the rules
//! read it, the owner's comments stay, a hash already there changes nothing.

use packing_engine::ids::CourierCode;
use packing_engine::workflows::{BUNDLED_CONFIG, BUNDLED_PARSER, LogoIs, remember_logo};

const HASH: &str = "0123456789abcdef";

#[test]
fn a_named_logo_goes_into_its_couriers_classifier() {
    let jnt = LogoIs::Courier(CourierCode::new("jnt-express"));
    let (text, rules) = remember_logo(BUNDLED_PARSER, BUNDLED_CONFIG, HASH, &jnt).unwrap();
    let classifier = rules
        .classifiers
        .iter()
        .find(|c| c.courier.as_str() == "jnt-express")
        .unwrap();
    assert_eq!(classifier.logos, ["39b9b7b9f61d0e7f", HASH]);
    assert!(text.contains("# 975x377 px, the same asset in every sample file"));
    assert_eq!(
        text.lines().count(),
        BUNDLED_PARSER.lines().count(),
        "only the list changed"
    );

    let (again, _) = remember_logo(&text, BUNDLED_CONFIG, HASH, &jnt).unwrap();
    assert_eq!(again, text, "a hash already remembered changes nothing");
}

#[test]
fn a_courier_without_a_classifier_gets_one_and_not_a_courier_its_list() {
    let text = "schema = \"1.0\"

# couriers
[[couriers]]
code = \"jne\"
name = \"JNE\"
";
    let jne = LogoIs::Courier(CourierCode::new("jne"));
    let (text, rules) = remember_logo(text, BUNDLED_CONFIG, HASH, &jne).unwrap();
    assert_eq!(rules.classifiers.len(), 1);
    assert_eq!(rules.classifiers[0].logos, [HASH]);
    assert!(text.contains("# couriers"));

    let (_, rules) = remember_logo(
        &text,
        BUNDLED_CONFIG,
        "fedcba9876543210",
        &LogoIs::NotCourier,
    )
    .unwrap();
    assert_eq!(rules.not_courier_logos, ["fedcba9876543210"]);
}

#[test]
fn a_file_the_rules_refuse_is_not_written() {
    let refused = remember_logo(
        "schema = \"9.9\"\n",
        BUNDLED_CONFIG,
        HASH,
        &LogoIs::NotCourier,
    );
    assert!(refused.is_err());
    let unknown = LogoIs::Courier(CourierCode::new("nobody"));
    assert!(remember_logo(BUNDLED_PARSER, BUNDLED_CONFIG, HASH, &unknown).is_err());
}
