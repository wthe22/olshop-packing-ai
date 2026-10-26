//! The logo prompt's answers written into `packing-workflows.toml`: the hash lands where the rules
//! read it, the owner's comments stay, a hash already there changes nothing.

use packing_engine::ids::CourierCode;
use packing_engine::workflows::{BUNDLED_WORKFLOWS, LogoIs, remember_logo};

const HASH: &str = "0123456789abcdef";

#[test]
fn a_named_logo_goes_into_its_couriers_classifier() {
    let jnt = LogoIs::Courier(CourierCode::new("jnt"));
    let (text, rules) = remember_logo(BUNDLED_WORKFLOWS, HASH, &jnt).unwrap();
    let classifier = rules
        .classifiers
        .iter()
        .find(|c| c.courier.as_str() == "jnt")
        .unwrap();
    assert_eq!(classifier.logos, ["39b9b7b9f61d0e7f", HASH]);
    assert!(text.contains("# 975x377 px, the same asset in every sample file"));
    assert_eq!(
        text.lines().count(),
        BUNDLED_WORKFLOWS.lines().count(),
        "only the list changed"
    );

    let (again, _) = remember_logo(&text, HASH, &jnt).unwrap();
    assert_eq!(again, text, "a hash already remembered changes nothing");
}

#[test]
fn a_courier_without_a_classifier_gets_one_and_not_a_courier_its_list() {
    let text = concat!(
        "schema = \"1.0\"\n\n# couriers\n[[couriers]]\ncode = \"jne\"\nname = \"JNE\"\n\n",
        "[[label_sort_rules]]\nname = \"default\"\n\n",
        "[[label_sort_rules.label_groups]]\ncode = \"Z\"\nname = \"Lainnya\"\n",
    );
    let jne = LogoIs::Courier(CourierCode::new("jne"));
    let (text, rules) = remember_logo(text, HASH, &jne).unwrap();
    assert_eq!(rules.classifiers.len(), 1);
    assert_eq!(rules.classifiers[0].logos, [HASH]);
    assert!(text.contains("# couriers"));

    let (_, rules) = remember_logo(&text, "fedcba9876543210", &LogoIs::NotCourier).unwrap();
    assert_eq!(rules.not_courier_logos, ["fedcba9876543210"]);
}

#[test]
fn a_file_the_rules_refuse_is_not_written() {
    let refused = remember_logo("schema = \"9.9\"\n", HASH, &LogoIs::NotCourier);
    assert!(refused.is_err());
    let unknown = LogoIs::Courier(CourierCode::new("nobody"));
    assert!(remember_logo(BUNDLED_WORKFLOWS, HASH, &unknown).is_err());
}
