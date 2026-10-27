//! Pages → copies (02-read.md step 5, duplicates.md): the label-part rule, a copy's values and
//! the clue that named its courier, and the pages that stop the batch.

use packing_engine::copies::{CopiesStop, page_fingerprint, read_copies};
use packing_engine::ids::OrderId;
use packing_engine::orders::PageItem;
use packing_engine::record::CourierClueKind;
use packing_engine::text::{LogoImage, PageText, Run};
use packing_engine::workflows::{BUNDLED_WORKFLOWS, Workflows, load_workflows};

const JNT_LOGO: &str = "39b9b7b9f61d0e7f";
const SICEPAT_LOGO: &str = "cf33ed60daa291b1";
const CARGO_SHAPE: &str = "753ccbd14749befa";
const ANTERAJA_LOGO: &str = "556e376839c5805d";
/// The `tokopedia | Shop` strip: not a courier.
const TOKOPEDIA_STRIP: &str = "6c1340227db55be4";
/// The `TikTok Shop` strip an instant label carries: not a courier.
const TIKTOK_STRIP: &str = "f2cf7019646c955b";

fn workflows() -> Workflows {
    load_workflows(BUNDLED_WORKFLOWS).unwrap()
}

fn run(x: f64, y: f64, text: &str) -> Run {
    Run {
        x,
        y,
        text: text.to_string(),
    }
}

/// A page from its runs: the text is the runs top to bottom, one per line.
fn page(runs: Vec<Run>, logos: &[(&str, Option<&str>)]) -> PageText {
    PageText {
        text: runs
            .iter()
            .map(|r| r.text.as_str())
            .collect::<Vec<_>>()
            .join("\n"),
        runs,
        width: 298.0,
        height: 420.0,
        logos: logos
            .iter()
            .map(|(hash, appearance)| LogoImage {
                hash: hash.to_string(),
                appearance: appearance.map(str::to_string),
            })
            .collect(),
    }
}

/// The slip table's header and one row of `quantity`; `total` printed under it.
fn slip(name: &str, quantity: i64, total: Option<i64>) -> Vec<Run> {
    let mut runs = vec![
        run(5.8, 130.0, "Product Name"),
        run(129.6, 130.0, "SKU"),
        run(172.2, 130.0, "Seller SKU"),
        run(264.6, 130.0, "Qty"),
        run(5.8, 122.0, name),
        run(129.6, 122.0, "honda"),
        run(172.2, 122.0, "SELL-01"),
        run(268.6, 122.0, &quantity.to_string()),
    ];
    if let Some(total) = total {
        runs.push(run(240.0, 106.0, "Qty Total:"));
        runs.push(run(288.3, 106.0, &total.to_string()));
    }
    runs
}

/// A label page: the Order ID, a tracking ID (when given), the deadline and a slip.
fn label(order: &str, tracking: Option<&str>, extra: &[&str], slip_runs: Vec<Run>) -> Vec<Run> {
    let mut runs = vec![run(5.8, 400.0, &format!("58000000000000000{order}"))];
    if let Some(tracking) = tracking {
        runs.push(run(5.8, 380.0, tracking));
    }
    runs.push(run(5.8, 370.0, "In transit by: 07/10/2026 23:59"));
    for (index, text) in extra.iter().enumerate() {
        runs.push(run(5.8, 360.0 - index as f64 * 10.0, text));
    }
    runs.extend(slip_runs);
    runs
}

/// A continuation page: the slip's header, the Order ID and a row.
fn continuation(order: &str, name: &str, quantity: i64, total: Option<i64>) -> Vec<Run> {
    let mut runs = slip(name, quantity, total);
    runs.push(run(
        150.0,
        98.0,
        &format!("Order ID: 58000000000000000{order}"),
    ));
    runs
}

fn pages(inputs: Vec<Vec<PageText>>) -> Vec<PageItem> {
    inputs
        .into_iter()
        .enumerate()
        .flat_map(|(input, pages)| {
            pages
                .into_iter()
                .enumerate()
                .map(move |(index, page)| (input, index, page))
        })
        .collect()
}

#[test]
fn a_label_page_starts_a_copy_and_a_page_without_a_label_part_continues_it() {
    let input = vec![
        page(
            label("1", Some("JY0000000001"), &[], slip("Sepatu", 1, None)),
            &[(JNT_LOGO, None)],
        ),
        page(continuation("1", "Spion", 2, Some(3)), &[]),
        page(
            label("2", Some("000000000002"), &[], slip("Jok", 1, Some(1))),
            &[(SICEPAT_LOGO, None)],
        ),
    ];
    let read = read_copies(pages(vec![input]), &["labels_1.pdf"], &workflows()).unwrap();
    assert_eq!(read.pages, 3);
    let copies = &read.copies;
    assert_eq!(copies.len(), 2);
    assert_eq!(copies[0].order_id, OrderId::new("580000000000000001"));
    assert_eq!((copies[0].page_start, copies[0].page_end), (1, 2));
    let names: Vec<&str> = copies[0].lines.iter().map(|l| l.name.as_str()).collect();
    assert_eq!(
        names,
        ["Sepatu", "Spion"],
        "the lines of every page, in page order"
    );
    assert_eq!(
        copies[0].total_quantity,
        Some(3),
        "the total of its last slip that prints one"
    );
    assert_eq!(copies[0].total_differs(), None);
    assert_eq!(copies[0].fingerprints.len(), 2);
    assert_eq!(copies[0].ship_by_date.unwrap().to_string(), "2026-10-07");
    assert_eq!(copies[0].courier.as_ref().unwrap().as_str(), "jnt");
    assert_eq!(copies[1].courier.as_ref().unwrap().as_str(), "sicepat");
    assert_eq!((copies[1].page_start, copies[1].page_end), (3, 3));
    assert!(read.unseen_logos.is_empty());
}

#[test]
fn the_same_order_twice_is_two_copies_never_one_longer_order() {
    let one = || {
        page(
            label("1", Some("JY0000000001"), &[], slip("Sepatu", 1, Some(1))),
            &[(JNT_LOGO, None)],
        )
    };
    let read = read_copies(
        pages(vec![vec![one(), one()], vec![one()]]),
        &["a.pdf", "b.pdf"],
        &workflows(),
    )
    .unwrap();
    let places: Vec<(usize, u16)> = read
        .copies
        .iter()
        .map(|c| (c.input, c.page_start))
        .collect();
    assert_eq!(places, [(0, 1), (0, 2), (1, 1)]);
    assert_eq!(read.copies[0].fingerprints, read.copies[2].fingerprints);
}

#[test]
fn a_page_that_cannot_be_placed_stops_the_batch() {
    let good = || {
        page(
            label("1", Some("JY0000000001"), &[], slip("Sepatu", 1, None)),
            &[],
        )
    };
    let cases: Vec<(&str, Vec<Vec<PageText>>, &str)> = vec![
        (
            "a first page that continues nothing",
            vec![
                vec![good()],
                vec![page(continuation("1", "Sepatu", 1, None), &[])],
            ],
            "Page 1 of b.pdf carries no label",
        ),
        (
            "a continuation of another order",
            vec![vec![
                good(),
                page(continuation("2", "Sepatu", 1, None), &[]),
            ]],
            "Page 2 of a.pdf is order 580000000000000002",
        ),
        (
            "a label part without an Order ID",
            vec![vec![page(vec![run(5.8, 380.0, "JY0000000001")], &[])]],
            "Page 1 of a.pdf has no Order ID",
        ),
        (
            "plain labels",
            vec![vec![
                page(label("1", Some("JY0000000001"), &[], Vec::new()), &[]),
                good(),
            ]],
            "1 of 2 pages are plain shipping labels",
        ),
    ];
    for (what, inputs, message) in cases {
        let stop = read_copies(pages(inputs), &["a.pdf", "b.pdf"], &workflows()).expect_err(what);
        assert!(stop.to_string().contains(message), "{what}: {stop}");
        if what == "plain labels" {
            assert!(matches!(stop, CopiesStop::NoSlip(_)));
        }
    }
}

#[test]
fn the_courier_and_the_clue_that_named_it() {
    type Case = (
        &'static str,
        Option<&'static str>,
        Vec<(&'static str, Option<&'static str>)>,
        &'static [&'static str],
        Option<&'static str>,
        CourierClueKind,
        Option<&'static str>,
    );
    let cases: Vec<Case> = vec![
        (
            "logo and form agree",
            Some("JY0000000001"),
            vec![(JNT_LOGO, None)],
            &[],
            Some("jnt"),
            CourierClueKind::Logo,
            Some(JNT_LOGO),
        ),
        (
            "a form with a hyphen, beside a channel strip",
            Some("TSA-00000000001"),
            vec![(ANTERAJA_LOGO, None), (TOKOPEDIA_STRIP, None)],
            &[],
            Some("anteraja"),
            CourierClueKind::Logo,
            Some(ANTERAJA_LOGO),
        ),
        (
            "a masked logo by its shape",
            Some("570000000001"),
            vec![("aaaaaaaaaaaaaaaa", Some(CARGO_SHAPE))],
            &[],
            Some("jnt-cargo"),
            CourierClueKind::Shape,
            Some(CARGO_SHAPE),
        ),
        (
            "logo and form disagree",
            Some("JY0000000001"),
            vec![(SICEPAT_LOGO, None)],
            &[],
            None,
            CourierClueKind::None,
            None,
        ),
        (
            "two logos naming two couriers",
            Some("JY0000000001"),
            vec![(JNT_LOGO, None), (SICEPAT_LOGO, None)],
            &[],
            None,
            CourierClueKind::None,
            None,
        ),
        (
            "no image, printed text",
            Some("570000000001"),
            vec![],
            &["FastTrack"],
            Some("jnt-cargo"),
            CourierClueKind::Text,
            Some("FastTrack"),
        ),
        (
            "no image, a lettered form",
            Some("TKP0000000001"),
            vec![],
            &[],
            Some("idx"),
            CourierClueKind::Form,
            None,
        ),
        (
            "no image, a bare-number form",
            Some("000000000001"),
            vec![],
            &[],
            None,
            CourierClueKind::None,
            None,
        ),
        (
            "no tracking ID",
            None,
            vec![(JNT_LOGO, None)],
            &[],
            None,
            CourierClueKind::None,
            None,
        ),
    ];
    for (what, tracking, logos, extra, courier, kind, value) in cases {
        let one = page(label("1", tracking, extra, slip("Sepatu", 1, None)), &logos);
        let read = read_copies(pages(vec![vec![one]]), &["a.pdf"], &workflows()).expect(what);
        let copy = &read.copies[0];
        assert_eq!(copy.courier.as_ref().map(|c| c.as_str()), courier, "{what}");
        assert_eq!(copy.courier_clue.kind, kind, "{what}");
        assert_eq!(copy.courier_clue.value.as_deref(), value, "{what}");
        if courier.is_some() && kind != CourierClueKind::Tag {
            assert!(
                copy.courier_clue.form.is_some(),
                "{what}: the form that agreed"
            );
        }
    }
}

#[test]
fn an_instant_label_is_named_by_its_tag_and_keeps_its_pickup_code() {
    let cases = [
        ("GKS", TIKTOK_STRIP, Some("gojek")),
        ("Sameday", TIKTOK_STRIP, Some("grab")),
        ("GKS", "bbbbbbbbbbbbbbbb", Some("gojek")),
        ("XYZ", TIKTOK_STRIP, None),
    ];
    for (tag, strip, courier) in cases {
        // The code is printed in large type beside its caption; a Seller SKU of the same form
        // further down is not it.
        let mut runs = label("1", None, &["Kode Pengambilan"], slip("Sepatu", 1, Some(1)));
        runs.push(run(5.8, 115.0, "AB1234"));
        runs.push(run(146.3, 347.0, "WX1234"));
        runs.push(run(250.0, 405.0, tag));
        let read = read_copies(
            pages(vec![vec![page(runs, &[(strip, None)])]]),
            &["a.pdf"],
            &workflows(),
        )
        .unwrap();
        let copy = &read.copies[0];
        assert_eq!(copy.courier.as_ref().map(|c| c.as_str()), courier, "{tag}");
        if courier.is_some() {
            assert_eq!(copy.courier_clue.kind, CourierClueKind::Tag);
            assert_eq!(copy.courier_clue.value.as_deref(), Some(tag));
        }
        assert_eq!(copy.pickup_code.as_deref(), Some("WX1234"));
        assert!(
            read.unseen_logos.is_empty(),
            "{tag}: an instant label's header image is a channel strip, never asked about"
        );
    }
}

#[test]
fn a_printed_total_that_differs_from_the_lines_is_found_per_copy() {
    let one = page(
        label("1", Some("JY0000000001"), &[], slip("Sepatu", 1, Some(3))),
        &[],
    );
    let read = read_copies(pages(vec![vec![one]]), &["a.pdf"], &workflows()).unwrap();
    assert_eq!(read.copies[0].total_differs(), Some((3, 1)));
}

#[test]
fn the_fingerprint_names_a_page_by_what_it_prints() {
    let base = page(
        vec![run(5.8, 400.0, "580000000000000001")],
        &[(JNT_LOGO, None)],
    );
    let same = base.clone();
    let mut moved = base.clone();
    moved.runs[0].x = 6.0;
    let mut other_logo = base.clone();
    other_logo.logos[0].hash = SICEPAT_LOGO.into();
    let mut hair = base.clone();
    hair.runs[0].x = 5.81;
    assert_eq!(page_fingerprint(&base), page_fingerprint(&same));
    assert_eq!(
        page_fingerprint(&base),
        page_fingerprint(&hair),
        "rounded to 0.1 pt"
    );
    assert_ne!(page_fingerprint(&base), page_fingerprint(&moved));
    assert_ne!(page_fingerprint(&base), page_fingerprint(&other_logo));
}
