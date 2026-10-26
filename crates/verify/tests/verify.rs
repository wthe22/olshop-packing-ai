//! The verifier on the repeat fixture's first input, with outputs made from it by `lopdf`: a right
//! set passes; a journal, an input, a ledger, an order, a group PDF or a summary that is not what
//! the record says is found.

use std::fs;
use std::path::{Path, PathBuf};

use lopdf::content::{Content, Operation};
use lopdf::{Document, Object, Stream, dictionary};
use packing_verify::{
    BatchCheck, Findings, ReprintCheck, RestoreCheck, page_digests, verify_batch, verify_reprint,
    verify_restore,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use xxhash_rust::xxh3::xxh3_64;

const INPUT: &str = "labels-repeat-1.pdf";
const GROUP_FILE: &str = "1-Z Lainnya x3.pdf";
const SUMMARY_FILE: &str = "1 summary.pdf";

fn testdata(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../testdata")
        .join(name)
}

struct Folder(PathBuf);

impl Folder {
    fn new(tag: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("packing-verify-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("input")).unwrap();
        fs::create_dir_all(dir.join("partial")).unwrap();
        Folder(dir)
    }
    fn input(&self) -> PathBuf {
        self.0.join("input")
    }
    fn partial(&self) -> PathBuf {
        self.0.join("partial")
    }
}

impl Drop for Folder {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn sha_and_size(path: &Path) -> (String, u64) {
    let bytes = fs::read(path).unwrap();
    (format!("{:x}", Sha256::digest(&bytes)), bytes.len() as u64)
}

fn line(value: &Value) -> String {
    let json = serde_json::to_string(value).unwrap();
    format!("{:016x} {json}\n", xxh3_64(json.as_bytes()))
}

/// The input's pages 1, 2–3 and 6 are orders 101, 102 and 103; pages 4–5, order 102's second
/// copy, are left out.
fn record(input_sha: &str, group: (String, u64), summary: (String, u64)) -> Value {
    let order = |id: &str, content: u16, start: u16, end: u16| json!({"order_id": id, "content": content, "pdf": 1, "page_start": start, "page_end": end, "group": "Z"});
    json!({
        "seq": 2, "time": "2026-10-08T09:00:00+07:00", "app": "0.2.0", "type": "batch_saved",
        "batch": 1, "amends": false,
        "inputs": [{"pdf": 1, "sha256": input_sha, "filename": INPUT, "page_count": 6}],
        "contents": [{"items": [{"quantity": 1}]}, {"items": [{"quantity": 1}]}, {"items": [{"quantity": 1}]}],
        "orders": [
            order("580000000000000101", 0, 1, 1),
            order("580000000000000102", 1, 2, 3),
            order("580000000000000103", 2, 6, 6),
        ],
        "left_out": [{"order_id": "580000000000000102", "pdf": 1, "page_start": 4, "page_end": 5,
                      "reason": "repeat", "kept": {"pdf": 1, "page_start": 2}}],
        "outputs": [
            {"file": GROUP_FILE, "group": "Z", "pages": 4, "size": group.1, "sha256": group.0},
            {"file": SUMMARY_FILE, "group": null, "pages": 1, "size": summary.1, "sha256": summary.0},
        ],
    })
}

fn started() -> Value {
    json!({"seq": 1, "time": "2026-10-08T09:00:00+07:00", "app": "0.2.0", "type": "started",
           "schema": "1.0", "namespace": "Toko Contoh", "date": "2026-10-08"})
}

/// The input's pages in `order` (1-based), as a group PDF: the same page objects, another tree.
fn group_pdf(path: &Path, order: &[u32]) {
    let mut doc = Document::load(testdata(INPUT)).unwrap();
    let pages = doc.get_pages();
    let kids: Vec<Object> = order.iter().map(|n| Object::Reference(pages[n])).collect();
    let root = doc
        .catalog()
        .unwrap()
        .get(b"Pages")
        .unwrap()
        .as_reference()
        .unwrap();
    let tree = doc.get_object_mut(root).unwrap().as_dict_mut().unwrap();
    tree.set("Kids", kids);
    tree.set("Count", order.len() as i64);
    doc.compress();
    doc.save(path).unwrap();
}

/// A one-page summary that prints `lines`.
fn summary_pdf(path: &Path, lines: &[String]) {
    let mut doc = Document::with_version("1.4");
    let pages_id = doc.new_object_id();
    let font = doc.add_object(dictionary! {
        "Type" => "Font", "Subtype" => "Type1", "BaseFont" => "Helvetica", "Encoding" => "WinAnsiEncoding",
    });
    let mut operations = Vec::new();
    for (index, text) in lines.iter().enumerate() {
        operations.push(Operation::new("BT", vec![]));
        operations.push(Operation::new("Tf", vec!["F1".into(), 10.into()]));
        operations.push(Operation::new(
            "Td",
            vec![40.into(), (780 - 20 * index as i64).into()],
        ));
        operations.push(Operation::new(
            "Tj",
            vec![Object::string_literal(text.as_str())],
        ));
        operations.push(Operation::new("ET", vec![]));
    }
    let content = doc.add_object(Stream::new(
        dictionary! {},
        Content { operations }.encode().unwrap(),
    ));
    let page = doc.add_object(dictionary! {
        "Type" => "Page", "Parent" => pages_id, "Contents" => content,
        "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        "Resources" => dictionary! { "Font" => dictionary! { "F1" => font } },
    });
    doc.objects.insert(
        pages_id,
        Object::Dictionary(dictionary! {
            "Type" => "Pages", "Kids" => vec![page.into()], "Count" => 1,
        }),
    );
    let catalog = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
    doc.trailer.set("Root", catalog);
    doc.save(path).unwrap();
}

fn right_summary() -> Vec<String> {
    vec![
        "3 orders - 3 units - 1 groups - printed 09:12".into(),
        format!("{GROUP_FILE} - 3 orders - 4 pages - 3 order contents"),
        "6 pages read = 4 written + 2 left out (2 repeats, 0 saved earlier)".into(),
    ]
}

/// A set as the save leaves it: the input stored by content, the outputs written; `tamper` changes
/// the record's JSON before it is sealed.
fn verify(
    tag: &str,
    group_order: &[u32],
    summary: &[String],
    tamper: impl Fn(&mut Value),
) -> Findings {
    let folder = Folder::new(tag);
    let (input_sha, _) = sha_and_size(&testdata(INPUT));
    fs::copy(
        testdata(INPUT),
        folder.input().join(format!("{input_sha}.pdf")),
    )
    .unwrap();
    let group = folder.partial().join(GROUP_FILE);
    group_pdf(&group, group_order);
    let summary_path = folder.partial().join(SUMMARY_FILE);
    summary_pdf(&summary_path, summary);
    let mut batch = record(
        &input_sha,
        sha_and_size(&group),
        sha_and_size(&summary_path),
    );
    tamper(&mut batch);
    let append = format!("{}{}", line(&started()), line(&batch));
    verify_batch(&BatchCheck {
        journal: b"",
        append: append.as_bytes(),
        input_dir: &folder.input(),
        output_dir: &folder.partial(),
    })
}

#[test]
fn a_set_that_is_what_its_record_says_has_no_finding() {
    let findings = verify("right", &[1, 2, 3, 6], &right_summary(), |_| {});
    assert!(findings.is_clean(), "{findings:?}");
}

#[test]
fn a_wrong_page_of_a_group_pdf_is_found() {
    let cases: [(&str, &[u32], &str); 4] = [
        ("doubled", &[1, 2, 2, 6], "is written 2 times"),
        ("missing", &[1, 2, 3], "is in no output"),
        // Order 102's second copy prints what its first does: written instead of page 6, it is a
        // doubled page, and page 6 is missing.
        (
            "a left-out copy instead of page 6",
            &[1, 2, 3, 4],
            "is written 2 times",
        ),
        ("swapped", &[2, 3, 1, 6], "not in the printing order"),
    ];
    for (what, pages, finding) in cases {
        let findings = verify(what, pages, &right_summary(), |batch| {
            batch["outputs"][0]["pages"] = json!(pages.len());
        });
        let all: Vec<&String> = findings.errors.iter().chain(&findings.warnings).collect();
        assert!(
            all.iter().any(|f| f.contains(finding)),
            "{what}: {findings:?}"
        );
        if what == "swapped" {
            assert!(
                findings.errors.is_empty(),
                "{what}: a warning alone: {findings:?}"
            );
        }
    }
}

#[test]
fn a_record_that_does_not_add_up_is_found() {
    type Tamper = fn(&mut Value);
    let cases: [(&str, Tamper, &str); 6] = [
        (
            "a page in no copy",
            |b| b["left_out"] = json!([]),
            "page 4 of labels-repeat-1.pdf is in no copy",
        ),
        (
            "a page in two copies",
            |b| b["orders"][2]["page_start"] = json!(5),
            "in 2 copies",
        ),
        (
            "an order twice",
            |b| b["orders"][2]["order_id"] = json!("580000000000000101"),
            "written twice",
        ),
        (
            "a repeat kept nowhere",
            |b| b["left_out"][0]["kept"] = json!({"pdf": 1, "page_start": 4}),
            "not written",
        ),
        (
            "an input changed",
            |b| b["inputs"][0]["page_count"] = json!(7),
            "has 6 pages, the record says 7",
        ),
        (
            "a group PDF not the one named",
            |b| b["outputs"][0]["size"] = json!(1),
            "not the file the record names",
        ),
    ];
    for (what, tamper, finding) in cases {
        let findings = verify(
            &what.replace(' ', "-"),
            &[1, 2, 3, 6],
            &right_summary(),
            tamper,
        );
        assert!(
            findings.errors.iter().any(|f| f.contains(finding)),
            "{what}: {findings:?}"
        );
    }
}

#[test]
fn a_summary_that_prints_other_counts_is_found() {
    let mut wrong = right_summary();
    wrong[0] = "3 orders - 4 units - 1 groups".into();
    let findings = verify("summary-counts", &[1, 2, 3, 6], &wrong, |_| {});
    assert!(
        findings
            .errors
            .iter()
            .any(|f| f.contains("orders, units and groups")),
        "{findings:?}"
    );
    let mut no_group = right_summary();
    no_group.remove(1);
    let findings = verify("summary-group", &[1, 2, 3, 6], &no_group, |_| {});
    assert!(
        findings.errors.iter().any(|f| f.contains(GROUP_FILE)),
        "{findings:?}"
    );
}

#[test]
fn a_journal_that_changed_is_found() {
    let folder = Folder::new("journal");
    let first = line(&started());
    let mut changed = first.clone().replace("Toko Contoh", "Toko Lain");
    changed.push_str(&line(&json!({"seq": 2, "time": "2026-10-08T09:00:00+07:00", "app": "0.2.0", "type": "recovered", "dropped_bytes": 0, "dropped_text": ""})));
    let next = json!({"seq": 9, "type": "batch_saved"});
    let findings = verify_batch(&BatchCheck {
        journal: changed.as_bytes(),
        append: line(&next).as_bytes(),
        input_dir: &folder.input(),
        output_dir: &folder.partial(),
    });
    assert!(
        findings
            .errors
            .iter()
            .any(|f| f.contains("line 1 is not sealed")),
        "{findings:?}"
    );
    assert!(
        findings
            .errors
            .iter()
            .any(|f| f.contains("no batch record")),
        "{findings:?}"
    );
}

#[test]
fn a_page_has_one_digest_however_it_is_stored() {
    let folder = Folder::new("digest");
    let stored = folder.partial().join("stored.pdf");
    group_pdf(&stored, &[1, 2, 3, 4, 5, 6]);
    let original = page_digests(&testdata(INPUT)).unwrap();
    assert_eq!(
        page_digests(&stored).unwrap(),
        original,
        "recompressed and renumbered"
    );
    let distinct: std::collections::BTreeSet<u64> = original.iter().copied().collect();
    assert_eq!(
        distinct.len(),
        4,
        "order 102's two copies print the same two pages; every other page its own"
    );

    // PDFium writes a real 1.0 as 1, and the page tree may hold an attribute for its pages.
    let mut doc = Document::load(testdata(INPUT)).unwrap();
    let first = doc.get_pages()[&1];
    let page = doc.get_object_mut(first).unwrap().as_dict_mut().unwrap();
    let media = page.get(b"MediaBox").unwrap().clone();
    page.remove(b"MediaBox");
    let respelled: Vec<Object> = media
        .as_array()
        .unwrap()
        .iter()
        .map(|n| Object::Integer(n.as_float().unwrap() as i64))
        .collect();
    let root = doc
        .catalog()
        .unwrap()
        .get(b"Pages")
        .unwrap()
        .as_reference()
        .unwrap();
    let tree = doc.get_object_mut(root).unwrap().as_dict_mut().unwrap();
    tree.set("MediaBox", respelled);
    let inherited = folder.partial().join("inherited.pdf");
    doc.save(&inherited).unwrap();
    assert_eq!(page_digests(&inherited).unwrap()[0], original[0]);

    let mut drawn = Document::load(testdata(INPUT)).unwrap();
    let content = drawn.get_page_contents(first)[0];
    let stream = drawn
        .get_object_mut(content)
        .unwrap()
        .as_stream_mut()
        .unwrap();
    let mut bytes = stream.decompressed_content().unwrap();
    bytes.extend_from_slice(b"\nBT /F1 9 Tf 1 0 0 1 10 10 Tm (x) Tj ET");
    stream.set_plain_content(bytes);
    let changed = folder.partial().join("changed.pdf");
    drawn.save(&changed).unwrap();
    assert_ne!(
        page_digests(&changed).unwrap()[0],
        original[0],
        "a page that draws more"
    );
}

const REPRINT_FILE: &str = "reprint 1 x2.pdf";

/// A reprint of orders 103 and 101 into today's `partial`, its pages in `order`; the input stored
/// under the date it was saved on.
fn verify_a_reprint(tag: &str, order: &[u32], tamper: impl Fn(&Folder)) -> Findings {
    let folder = Folder::new(tag);
    let (input_sha, _) = sha_and_size(&testdata(INPUT));
    let input_dir = folder.0.join("2026-10-08").join("input");
    fs::create_dir_all(&input_dir).unwrap();
    fs::copy(testdata(INPUT), input_dir.join(format!("{input_sha}.pdf"))).unwrap();
    let file = folder.partial().join(REPRINT_FILE);
    group_pdf(&file, order);
    let (sha, size) = sha_and_size(&file);
    let reprinted = |id: &str, page: u16| json!({"order_id": id, "date": "2026-10-08", "batch": 1, "sha256": input_sha, "page_start": page, "page_end": page});
    let record = json!({
        "seq": 2, "time": "2026-10-09T10:00:00+07:00", "app": "0.2.0", "type": "reprint_made",
        "number": 1, "file": REPRINT_FILE, "pages": 2, "size": size, "sha256": sha,
        "orders": [reprinted("580000000000000103", 6), reprinted("580000000000000101", 1)],
    });
    tamper(&folder);
    let append = format!("{}{}", line(&started()), line(&record));
    verify_reprint(&ReprintCheck {
        journal: b"",
        append: append.as_bytes(),
        namespace_dir: &folder.0,
        output_dir: &folder.partial(),
    })
}

#[test]
fn a_reprint_holds_exactly_its_orders_pages_in_their_order() {
    let right = verify_a_reprint("reprint", &[6, 1], |_| {});
    assert!(right.is_clean(), "{right:?}");

    let swapped = verify_a_reprint("reprint-swapped", &[1, 6], |_| {});
    assert!(
        swapped
            .errors
            .iter()
            .any(|f| f.contains("page 1 of reprint 1 x2.pdf is not page 6")),
        "{swapped:?}"
    );
    let short = verify_a_reprint("reprint-short", &[6], |_| {});
    assert!(
        short.errors.iter().any(|f| f.contains("is not in reprint")),
        "{short:?}"
    );
    let changed = verify_a_reprint("reprint-input", &[6, 1], |folder| {
        let input = fs::read_dir(folder.0.join("2026-10-08").join("input"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        fs::write(input, b"not the labels").unwrap();
    });
    assert!(
        changed.errors.iter().any(|f| f.contains("has changed")),
        "{changed:?}"
    );
}

/// The date's journal holds batch 1; `file` is written again into `partial` and named by an
/// `output_restored` of `batch`.
fn verify_a_restore(tag: &str, file: &str, batch: u8, write: impl Fn(&Path)) -> Findings {
    let folder = Folder::new(tag);
    let (input_sha, _) = sha_and_size(&testdata(INPUT));
    fs::copy(
        testdata(INPUT),
        folder.input().join(format!("{input_sha}.pdf")),
    )
    .unwrap();
    let journal = format!(
        "{}{}",
        line(&started()),
        line(&record(
            &input_sha,
            ("0".repeat(64), 1),
            ("0".repeat(64), 1)
        ))
    );
    let path = folder.partial().join(file);
    write(&path);
    let (sha, size) = sha_and_size(&path);
    let pages = page_digests(&path).map_or(0, |pages| pages.len());
    let restored = json!({"seq": 3, "time": "2026-10-08T11:00:00+07:00", "app": "0.2.0", "type": "output_restored",
                          "batch": batch, "file": file, "pages": pages, "size": size, "sha256": sha});
    verify_restore(&RestoreCheck {
        journal: journal.as_bytes(),
        append: line(&restored).as_bytes(),
        input_dir: &folder.input(),
        output_dir: &folder.partial(),
    })
}

#[test]
fn a_restored_output_is_checked_as_at_the_save() {
    let group = verify_a_restore("restore-group", GROUP_FILE, 1, |path| {
        group_pdf(path, &[1, 2, 3, 6])
    });
    assert!(group.is_clean(), "{group:?}");
    let summary = verify_a_restore("restore-summary", SUMMARY_FILE, 1, |path| {
        summary_pdf(path, &right_summary())
    });
    assert!(summary.is_clean(), "{summary:?}");

    let missing = verify_a_restore("restore-missing", GROUP_FILE, 1, |path| {
        group_pdf(path, &[1, 2, 6])
    });
    assert!(
        missing.errors.iter().any(|f| f.contains("is in no output")),
        "{missing:?}"
    );
    let counts = verify_a_restore("restore-counts", SUMMARY_FILE, 1, |path| {
        summary_pdf(path, &right_summary()[..2])
    });
    assert!(
        counts.errors.iter().any(|f| f.contains("prints no ledger")),
        "{counts:?}"
    );
    let not_live = verify_a_restore("restore-not-live", GROUP_FILE, 2, |path| {
        group_pdf(path, &[1, 2, 3, 6])
    });
    assert!(
        not_live
            .errors
            .iter()
            .any(|f| f.contains("batch 2 is not live")),
        "{not_live:?}"
    );
}
