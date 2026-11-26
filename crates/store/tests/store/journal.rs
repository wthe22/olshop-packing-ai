//! The journal: a commit read back, the torn tail, damage by line, a record the fold refuses.

use packing_engine::ids::{BatchId, OrderId, TrackingId};
use packing_engine::record::{
    Area, AreasFilled, BatchReverted, OrderArea, OrderWeight, PrintSent, PrintedMarked, RecordBody,
    Recovered, ReprintMade, Weight, WeightsFilled,
};
use packing_store::{DamageReason, Journal, StoreError, open_date};

use crate::common::*;

#[test]
fn a_first_commit_starts_the_journal_and_reads_back() {
    let (_data, folder) = date_folder("first-commit");
    let mut journal = Journal::open(&folder).unwrap();
    assert_eq!(journal.fold().last_seq, 0);
    assert_eq!(journal.fold().next_batch(), BatchId::new(1));

    let saved = batch(1, false, Vec::new());
    let records = journal
        .commit(vec![RecordBody::BatchSaved(saved.clone())], APP)
        .unwrap();
    assert_eq!(records.len(), 2, "started, then the batch");
    assert_eq!(records[0].body, started());
    assert_eq!(records[1].seq, 2);

    let reopened = Journal::open(&folder).unwrap();
    assert!(reopened.damage().is_none());
    assert_eq!(reopened.fold(), journal.fold());
    assert_eq!(reopened.fold().live[&BatchId::new(1)].saved, saved);
    assert_eq!(reopened.fold().next_batch(), BatchId::new(2));
    assert_eq!(reopened.fold().next_pdf().get(), 2);
    assert_eq!(journal_text(&folder).lines().count(), 2);
}

#[test]
fn prepared_lines_are_what_the_commit_appends_and_go_stale_after_another_commit() {
    let (_data, folder) = date_folder("prepared");
    let mut journal = Journal::open(&folder).unwrap();
    assert!(journal.bytes().unwrap().is_empty());
    let first = outputs(folder.dir(), 1, "one");
    let pending = journal
        .prepare(vec![RecordBody::BatchSaved(batch(1, false, first))], APP)
        .unwrap();
    let bytes = pending.bytes.clone();
    journal.append_pending(pending).unwrap();
    assert_eq!(journal.bytes().unwrap(), bytes);

    let mark = |printed| {
        RecordBody::PrintedMarked(PrintedMarked {
            batch: BatchId::new(1),
            group: "A".into(),
            printed,
        })
    };
    let stale = journal.prepare(vec![mark(true)], APP).unwrap();
    journal.commit(vec![mark(false)], APP).unwrap();
    assert!(matches!(
        journal.append_pending(stale),
        Err(StoreError::Rejected(DamageReason::SeqGap { .. }))
    ));
    assert_eq!(journal.fold().last_seq, 3);
}

#[test]
fn another_copys_commit_is_seen_on_disk_and_a_stale_fold_appends_nothing() {
    let (_data, folder) = date_folder("two-copies");
    let mut mine = Journal::open(&folder).unwrap();
    let mut theirs = Journal::open(&folder).unwrap();
    assert!(!mine.changed_on_disk());
    let saved = |number| RecordBody::BatchSaved(batch(number, false, Vec::new()));
    theirs.commit(vec![saved(1)], APP).unwrap();
    assert!(mine.changed_on_disk() && !theirs.changed_on_disk());

    let before = journal_text(&folder);
    assert!(matches!(
        mine.commit(vec![saved(1)], APP),
        Err(StoreError::JournalChanged { .. })
    ));
    assert_eq!(journal_text(&folder), before, "nothing appended");
    let mut reopened = Journal::open(&folder).unwrap();
    reopened.commit(vec![saved(2)], APP).unwrap();
}

#[test]
fn sent_and_printed_marks_revert_amend_and_reprints_fold_by_their_rules() {
    let (_data, folder) = date_folder("fold-rules");
    let mut journal = Journal::open(&folder).unwrap();
    let out = folder.dir().to_path_buf();
    let first = outputs(&out, 1, "one");
    journal
        .commit(vec![RecordBody::BatchSaved(batch(1, false, first))], APP)
        .unwrap();
    journal
        .commit(
            vec![RecordBody::PrintedMarked(PrintedMarked {
                batch: BatchId::new(1),
                group: "A".into(),
                printed: true,
            })],
            APP,
        )
        .unwrap();
    assert!(journal.fold().live[&BatchId::new(1)].printed.contains("A"));
    let sent = |sent| {
        RecordBody::PrintSent(PrintSent {
            batch: BatchId::new(1),
            group: "A".into(),
            sent,
        })
    };
    journal.commit(vec![sent(true)], APP).unwrap();
    assert!(journal.fold().live[&BatchId::new(1)].sent.contains_key("A"));
    journal.commit(vec![sent(false)], APP).unwrap();
    assert!(
        journal.fold().live[&BatchId::new(1)].sent.is_empty(),
        "the mark taken back"
    );
    journal.commit(vec![sent(true)], APP).unwrap();

    let amended = batch(1, true, outputs(&out, 1, "amended"));
    journal
        .commit(vec![RecordBody::BatchSaved(amended)], APP)
        .unwrap();
    let live = &journal.fold().live[&BatchId::new(1)];
    assert!(live.printed.is_empty(), "an amend starts unticked");
    assert!(live.sent.is_empty(), "and not sent");
    assert_ne!(live.create_time.to_string(), "");

    journal
        .commit(
            vec![RecordBody::ReprintMade(ReprintMade {
                number: 1,
                file: "Reprint 1 x0.pdf".into(),
                pages: 0,
                size: 0,
                sha256: sha('c'),
                orders: Vec::new(),
            })],
            APP,
        )
        .unwrap();
    journal
        .commit(
            vec![RecordBody::BatchReverted(BatchReverted {
                batch: BatchId::new(1),
            })],
            APP,
        )
        .unwrap();
    assert!(journal.fold().live.is_empty());
    assert_eq!(journal.fold().next_batch(), BatchId::new(1));
    assert_eq!(journal.fold().reprints.len(), 1);
    assert_eq!(Journal::open(&folder).unwrap().fold(), journal.fold());
}

#[test]
fn a_record_the_fold_refuses_writes_nothing() {
    let (_data, folder) = date_folder("refused");
    let mut journal = Journal::open(&folder).unwrap();
    journal
        .commit(
            vec![RecordBody::BatchSaved(batch(1, false, Vec::new()))],
            APP,
        )
        .unwrap();
    let before = journal_text(&folder);
    let cases = [
        RecordBody::BatchSaved(batch(3, false, Vec::new())),
        RecordBody::BatchSaved(batch(2, true, Vec::new())),
        RecordBody::BatchReverted(BatchReverted {
            batch: BatchId::new(2),
        }),
        RecordBody::PrintedMarked(PrintedMarked {
            batch: BatchId::new(1),
            group: "Q".into(),
            printed: true,
        }),
        RecordBody::PrintSent(PrintSent {
            batch: BatchId::new(2),
            group: "A".into(),
            sent: true,
        }),
        areas_filled(1, "580000000000000001"),
        areas_filled(1, "580000000000000009"),
        areas_filled(2, "580000000000000001"),
        weights_filled(1, "580000000000000001"),
        weights_filled(1, "580000000000000009"),
        weights_filled(2, "580000000000000001"),
        started(),
    ];
    for body in cases {
        let kind = body.kind();
        let error = journal.commit(vec![body], APP).unwrap_err();
        assert!(matches!(error, StoreError::Rejected(_)), "{kind}: {error}");
    }
    assert_eq!(journal_text(&folder), before);
}

/// The area of one order of `batch`: a province only.
fn areas_filled(batch: u8, order_id: &str) -> RecordBody {
    RecordBody::AreasFilled(AreasFilled {
        batch: BatchId::new(batch),
        orders: vec![OrderArea {
            order_id: OrderId::new(order_id),
            area: Area {
                province: Some("JAWA BARAT".into()),
                ..Area::default()
            },
        }],
    })
}

/// The weight of one order of `batch`.
fn weights_filled(batch: u8, order_id: &str) -> RecordBody {
    RecordBody::WeightsFilled(WeightsFilled {
        batch: BatchId::new(batch),
        orders: vec![OrderWeight {
            order_id: OrderId::new(order_id),
            weight: Weight::Grams(490),
        }],
    })
}

#[test]
fn an_older_record_reads_its_pickup_code_as_tracking_id_and_gets_its_area_and_weight_once() {
    let (_data, folder) = date_folder("older-record");
    let mut journal = Journal::open(&folder).unwrap();
    let mut older = batch(1, false, Vec::new());
    older.orders[0].tracking_id = None;
    older.orders[0].pickup_code = Some("WX1234".into());
    for order in &mut older.orders {
        order.area = None;
        order.weight = None;
    }
    journal
        .commit(vec![RecordBody::BatchSaved(older)], APP)
        .unwrap();
    let order = &journal.fold().live[&BatchId::new(1)].saved.orders[0];
    assert_eq!(order.tracking_id, Some(TrackingId::new("WX1234")));
    assert_eq!(order.area, None);

    journal
        .commit(vec![areas_filled(1, "580000000000000001")], APP)
        .unwrap();
    let orders = &journal.fold().live[&BatchId::new(1)].saved.orders;
    assert_eq!(
        orders[0]
            .area
            .as_ref()
            .and_then(|area| area.province.as_deref()),
        Some("JAWA BARAT")
    );
    assert_eq!(orders[1].area, None, "an order not named keeps none");
    let again = journal.commit(vec![areas_filled(1, "580000000000000001")], APP);
    assert!(matches!(again, Err(StoreError::Rejected(_))), "once only");

    journal
        .commit(vec![weights_filled(1, "580000000000000001")], APP)
        .unwrap();
    let orders = &journal.fold().live[&BatchId::new(1)].saved.orders;
    assert_eq!(orders[0].weight, Some(Weight::Grams(490)));
    assert_eq!(orders[1].weight, None, "an order not named keeps none");
    let again = journal.commit(vec![weights_filled(1, "580000000000000001")], APP);
    assert!(matches!(again, Err(StoreError::Rejected(_))), "once only");
    assert_eq!(Journal::open(&folder).unwrap().fold(), journal.fold());
}

#[test]
fn a_torn_tail_is_left_out_then_cut_and_recorded_by_the_next_commit() {
    let (_data, folder) = date_folder("torn");
    let first = record(2, RecordBody::BatchSaved(batch(1, false, Vec::new())));
    let torn_tails: [&[u8]; 2] = [b"3f9a0c71b2d4e5f6 {\"seq\":3,", b"0000000000000000 {}\n"];
    for tail in torn_tails {
        write_journal(&folder, &[record(1, started()), first.clone()], tail);
        let mut journal = Journal::open(&folder).unwrap();
        assert!(journal.damage().is_none());
        assert_eq!(journal.torn(), Some(tail));
        assert_eq!(journal.fold().last_seq, 2);

        let records = journal
            .commit(
                vec![RecordBody::BatchReverted(BatchReverted {
                    batch: BatchId::new(1),
                })],
                APP,
            )
            .unwrap();
        let recovered = RecordBody::Recovered(Recovered {
            dropped_bytes: tail.len() as u64,
            dropped_text: String::from_utf8_lossy(tail).into_owned(),
        });
        assert_eq!(records[0].body, recovered);
        assert_eq!(records[0].seq, 3);

        let reopened = Journal::open(&folder).unwrap();
        assert!(reopened.damage().is_none() && reopened.torn().is_none());
        assert_eq!(reopened.fold().last_seq, 4);
        assert_eq!(journal_text(&folder).lines().count(), 4);
    }
}

#[test]
fn damage_is_named_by_its_line_and_nothing_is_written() {
    let (_data, folder) = date_folder("damage");
    let saved = |n| RecordBody::BatchSaved(batch(n, false, Vec::new()));
    let other = RecordBody::Started(packing_engine::record::Started {
        schema: packing_engine::record::JOURNAL_SCHEMA.to_string(),
        date: folder.date().tomorrow().unwrap(),
    });
    let v0_4 = RecordBody::Started(packing_engine::record::Started {
        schema: "1.0".into(),
        date: folder.date(),
    });
    let mut changed_line = String::from_utf8(packing_store::encode_line(&record(2, saved(1))))
        .unwrap()
        .replace("labels_1", "labels_9");
    changed_line
        .push_str(&String::from_utf8(packing_store::encode_line(&record(3, saved(2)))).unwrap());
    type Case = (
        &'static str,
        Vec<packing_engine::record::Record>,
        Vec<u8>,
        usize,
    );
    let cases: Vec<Case> = vec![
        (
            "a bad line before the last",
            vec![record(1, started())],
            changed_line.into_bytes(),
            2,
        ),
        (
            "a seq gap",
            vec![record(1, started()), record(3, saved(1))],
            Vec::new(),
            2,
        ),
        (
            "a batch number out of turn",
            vec![record(1, started()), record(2, saved(2))],
            Vec::new(),
            2,
        ),
        (
            "started after line 1",
            vec![record(1, started()), record(2, started())],
            Vec::new(),
            2,
        ),
        ("no started first", vec![record(1, saved(1))], Vec::new(), 1),
        ("another date", vec![record(1, other)], Vec::new(), 1),
        ("a v0.4 journal", vec![record(1, v0_4)], Vec::new(), 1),
    ];
    for (what, records, tail, line) in cases {
        write_journal(&folder, &records, &tail);
        let opened = open_date(&folder).unwrap();
        let damage = opened
            .journal
            .damage()
            .unwrap_or_else(|| panic!("{what}: no damage"));
        assert_eq!(damage.line, line, "{what}: {damage}");
        if what == "another date" {
            assert!(
                matches!(damage.reason, DamageReason::OtherDate { .. }),
                "{what}"
            );
        }
        assert!(
            !opened.views_written,
            "{what}: a damaged date gets no views"
        );
        let mut journal = opened.journal;
        let before = journal_text(&folder);
        let error = journal.commit(vec![saved(1)], APP).unwrap_err();
        assert!(matches!(error, StoreError::Damaged(_)), "{what}: {error}");
        assert_eq!(journal_text(&folder), before, "{what}: nothing written");
    }
}

#[test]
fn a_newer_copys_record_type_is_passed_over_and_the_date_still_takes_commits() {
    let (_data, folder) = date_folder("passed-over");
    let mut tail = sealed_line(
        r#"{"seq":3,"time":"2026-10-07T09:00:00+07:00","app":"0.9.0","type":"orders_moved","to":2}"#,
    );
    tail.extend(packing_store::encode_line(&record(
        4,
        RecordBody::BatchSaved(batch(2, false, Vec::new())),
    )));
    let records = [
        record(1, started()),
        record(2, RecordBody::BatchSaved(batch(1, false, Vec::new()))),
    ];
    write_journal(&folder, &records, &tail);

    let mut journal = Journal::open(&folder).unwrap();
    assert!(journal.damage().is_none());
    let fold = journal.fold();
    assert_eq!(fold.last_seq, 4);
    assert_eq!(fold.live.len(), 2, "the records after it fold as before");
    let passed: Vec<_> = fold
        .passed_over
        .iter()
        .map(|r| (r.seq, r.app.as_str(), r.kind.as_str()))
        .collect();
    assert_eq!(passed, [(3, "0.9.0", "orders_moved")]);
    let reverted = RecordBody::BatchReverted(BatchReverted {
        batch: BatchId::new(2),
    });
    journal.commit(vec![reverted], APP).unwrap();
    assert_eq!(Journal::open(&folder).unwrap().fold().last_seq, 5);
}

#[test]
fn a_journal_of_another_major_stops_at_its_first_line() {
    let (_data, folder) = date_folder("newer-major");
    let first = |schema: &str| {
        sealed_line(&format!(
            r#"{{"seq":1,"time":"2026-10-07T07:00:00+07:00","app":"0.9.0","type":"started","schema":"{schema}","date":"2026-10-07"}}"#
        ))
    };
    let unreadable = sealed_line(r#"{"seq":2,"app":"0.9.0","type":"batch_saved","batch":"one"}"#);
    let cases = [
        (
            "3.0",
            Some(DamageReason::Newer {
                schema: "3.0".into(),
                app: "0.9.0".into(),
            }),
        ),
        ("2.7", None),
    ];
    for (schema, newer) in cases {
        let bytes = [first(schema), unreadable.clone()].concat();
        write_journal(&folder, &[], &bytes);
        let damage = Journal::open(&folder).unwrap().damage().cloned().unwrap();
        match newer {
            Some(reason) => assert_eq!((damage.line, damage.reason), (1, reason), "{schema}"),
            None => assert_eq!(
                (damage.line, damage.reason),
                (2, DamageReason::Unreadable),
                "{schema}: a minor of this major reads, and a known type that does not read is damage"
            ),
        }
    }
    let damaged = Journal::open(&folder).unwrap();
    assert!(matches!(
        damaged.prepare(Vec::new(), APP),
        Err(StoreError::Damaged(_))
    ));
    let text = DamageReason::Newer {
        schema: "3.0".into(),
        app: "0.9.0".into(),
    }
    .to_string();
    assert_eq!(
        text,
        "it is of a newer format (schema 3.0, Packing v0.9.0): this copy reads 2.x"
    );
}
