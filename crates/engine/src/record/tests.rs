use super::*;

fn round_trip(json: &str) -> Record {
    let record: Record = serde_json::from_str(json).expect("parse the record");
    assert_eq!(
        serde_json::to_string(&record).unwrap(),
        json,
        "written back unchanged"
    );
    record
}

#[test]
fn the_numbering_comes_first_then_the_type_and_its_fields() {
    let json = r#"{"seq":2,"time":"2026-10-07T07:40:12+07:00","app":"0.3.3","type":"batch_reverted","batch":3}"#;
    let record = round_trip(json);
    assert_eq!(record.seq, 2);
    assert_eq!(
        record.body,
        RecordBody::BatchReverted(BatchReverted {
            batch: BatchId::new(3)
        })
    );
    assert_eq!(record.body.kind(), "batch_reverted");
}

#[test]
fn a_left_out_copy_stores_its_reason_and_what_was_kept() {
    let repeat = r#"{"order_id":"580000000000000001","pdf":2,"page_start":5,"page_end":5,"reason":"repeat","kept":{"pdf":1,"page_start":3}}"#;
    let copy: LeftOutCopy = serde_json::from_str(repeat).unwrap();
    assert_eq!(
        copy.reason,
        LeftOutReason::Repeat {
            pdf: PdfId::new(1),
            page_start: 3
        }
    );
    assert_eq!(serde_json::to_string(&copy).unwrap(), repeat);

    let earlier = r#"{"order_id":"580000000000000002","pdf":1,"page_start":7,"page_end":8,"reason":"saved_earlier","kept":{"batch":1}}"#;
    let copy: LeftOutCopy = serde_json::from_str(earlier).unwrap();
    assert_eq!(
        copy.reason,
        LeftOutReason::SavedEarlier {
            batch: BatchId::new(1)
        }
    );
    assert_eq!(serde_json::to_string(&copy).unwrap(), earlier);
}

#[test]
fn a_started_record_and_a_recovered_one_read_back() {
    round_trip(
        r#"{"seq":1,"time":"2026-10-07T07:40:12+07:00","app":"0.3.3","type":"started","schema":"2.0","date":"2026-10-07"}"#,
    );
    round_trip(
        r#"{"seq":4,"time":"2026-10-07T09:00:00+07:00","app":"0.3.3","type":"recovered","dropped_bytes":5,"dropped_text":"3f9a0"}"#,
    );
}

#[test]
fn an_areas_filled_record_names_each_order_with_its_area_flat() {
    let json = r#"{"seq":5,"time":"2026-10-07T09:00:00+07:00","app":"0.8.2","type":"areas_filled","batch":1,"orders":[{"order_id":"580000000000000001","province":"JAWA BARAT","city":"KAB. CONTOH","district":null}]}"#;
    let RecordBody::AreasFilled(filled) = round_trip(json).body else {
        panic!("an areas_filled record");
    };
    assert_eq!(
        filled.orders[0].area,
        Area {
            province: Some("JAWA BARAT".into()),
            city: Some("KAB. CONTOH".into()),
            district: None,
        }
    );
}

#[test]
fn a_weights_filled_record_names_each_order_with_its_grams_or_none_printed() {
    let json = r#"{"seq":6,"time":"2026-10-07T09:00:00+07:00","app":"0.8.4","type":"weights_filled","batch":1,"orders":[{"order_id":"580000000000000001","weight":{"grams":490}},{"order_id":"580000000000000002","weight":"not_printed"}]}"#;
    let RecordBody::WeightsFilled(filled) = round_trip(json).body else {
        panic!("a weights_filled record");
    };
    let weights: Vec<Weight> = filled.orders.iter().map(|order| order.weight).collect();
    assert_eq!(weights, [Weight::Grams(490), Weight::NotPrinted]);
}

#[test]
fn a_sha256_is_64_lower_case_hex_characters() {
    let good = "a".repeat(64);
    assert!(Sha256::try_from(good).is_ok());
    for bad in [
        "A".repeat(64),
        "a".repeat(63),
        format!("{}g", "a".repeat(63)),
    ] {
        assert!(Sha256::try_from(bad.clone()).is_err(), "{bad}");
    }
    let json = format!("\"{}\"", "b".repeat(64));
    let sha: Sha256 = serde_json::from_str(&json).unwrap();
    assert_eq!(sha.as_str(), "b".repeat(64));
    assert!(serde_json::from_str::<Sha256>("\"xyz\"").is_err());
}

#[test]
fn every_type_this_copy_reads_is_one_of_its_kinds() {
    let error = |kind: &str| {
        let json = format!(
            r#"{{"seq":2,"time":"2026-10-07T07:40:12+07:00","app":"0.8.0","type":"{kind}"}}"#
        );
        serde_json::from_str::<Record>(&json)
            .unwrap_err()
            .to_string()
    };
    for kind in RecordBody::KINDS {
        assert!(
            !error(kind).contains("unknown variant"),
            "{kind}: {}",
            error(kind)
        );
    }
    assert!(error("orders_moved").contains("unknown variant"));
}
