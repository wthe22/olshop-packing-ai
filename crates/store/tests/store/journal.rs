//! The journal: a commit read back, the torn tail, damage by line, a record the fold refuses.

use packing_engine::ids::BatchId;
use packing_engine::record::{BatchReverted, PrintedMarked, RecordBody, Recovered, ReprintMade};
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
    let first = outputs(&folder.dir().join("output"), 1, "one");
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
fn printed_marks_revert_amend_and_reprints_fold_by_their_rules() {
    let (_data, folder) = date_folder("fold-rules");
    let mut journal = Journal::open(&folder).unwrap();
    let out = folder.dir().join("output");
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

    let amended = batch(1, true, outputs(&out, 1, "amended"));
    journal
        .commit(vec![RecordBody::BatchSaved(amended)], APP)
        .unwrap();
    let live = &journal.fold().live[&BatchId::new(1)];
    assert!(live.printed.is_empty(), "an amend starts unticked");
    assert_ne!(live.create_time.to_string(), "");

    journal
        .commit(
            vec![RecordBody::ReprintMade(ReprintMade {
                number: 1,
                file: "reprint 1 x0.pdf".into(),
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
        started(),
    ];
    for body in cases {
        let kind = body.kind();
        let error = journal.commit(vec![body], APP).unwrap_err();
        assert!(matches!(error, StoreError::Rejected(_)), "{kind}: {error}");
    }
    assert_eq!(journal_text(&folder), before);
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
        schema: "1.0".into(),
        namespace: "Toko Lain".into(),
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
        ("another owner", vec![record(1, other)], Vec::new(), 1),
    ];
    for (what, records, tail, line) in cases {
        write_journal(&folder, &records, &tail);
        let opened = open_date(&folder).unwrap();
        let damage = opened
            .journal
            .damage()
            .unwrap_or_else(|| panic!("{what}: no damage"));
        assert_eq!(damage.line, line, "{what}: {damage}");
        if what == "another owner" {
            assert!(
                matches!(damage.reason, DamageReason::OtherFolder { .. }),
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
