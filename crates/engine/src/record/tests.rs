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
    let json = r#"{"seq":2,"time":"2026-10-07T07:40:12+07:00","app":"0.2.0","type":"batch_reverted","batch":3}"#;
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
        r#"{"seq":1,"time":"2026-10-07T07:40:12+07:00","app":"0.2.0","type":"started","schema":"1.0","namespace":"Toko Contoh","date":"2026-10-07"}"#,
    );
    round_trip(
        r#"{"seq":4,"time":"2026-10-07T09:00:00+07:00","app":"0.2.0","type":"recovered","dropped_bytes":5,"dropped_text":"3f9a0"}"#,
    );
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
