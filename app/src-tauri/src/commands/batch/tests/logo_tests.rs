//! The logo prompt: the answer applied to the draft, the base64 picture and the hash taught
//! into `couriers.toml`.

use super::*;
use packing_engine::ids::{OrderId, PdfNumber};

/// A state holding a draft of two orders the courier data could not name — one `JY…` (J&T
/// Express alone), one `00…` (shared with SiCepat) — both carrying one unseen logo, with a
/// plan whose first pick takes the courier `J&T Express`.
fn state_with_logo_draft(folder: &Path) -> AppState {
    let unknown = |id: &str, tracking: &str, name: &str| {
        let mut order = read_order(id, tracking, vec![line(name, "", 1)]);
        order.order.courier = String::new();
        order
    };
    let mut draft = Draft {
        files: Vec::new(),
        orders: vec![
            unknown("580000000000000001", "JY0000001234", "Sepatu A"),
            unknown("580000000000000002", "000000000202", "Spion X"),
        ],
        duplicates: Vec::new(),
        warnings: Vec::new(),
        pages: 2,
        total_orders: 2,
        slip_pages: 2,
        unseen_logos: vec![UnseenLogo {
            hash: "ffffffffffffffff".to_string(),
            appearance: None,
            pages: 2,
            order_ids: vec![
                OrderId::new("580000000000000001"),
                OrderId::new("580000000000000002"),
            ],
        }],
        read_warnings: Vec::new(),
    };
    draft.read_warnings = vec![Warning::unknown_courier(&draft.orders).expect("both unknown")];
    draft.warnings = draft
        .read_warnings
        .iter()
        .map(|warning| warning.to_string())
        .collect();

    let state = AppState::new();
    state.set_data_folder(folder.to_path_buf());
    state.set_day("2026-10-07");
    state.set_draft(OpenDraft {
        draft,
        plan: Plan::from_categories(&[category("A", "J&T", Some("courier starts_with \"J&T\""))]),
        first_number: PdfNumber::new(1),
        couriers: packing_engine::label::courier_data().clone(),
        grouping: Grouping::default(),
        amend: None,
        logo_pictures: HashMap::from([(
            "ffffffffffffffff".to_string(),
            Some("data:image/png;base64,Zm9v".to_string()),
        )]),
    });
    state
}

#[test]
fn base64_encodes_with_padding() {
    assert_eq!(base64(b""), "");
    assert_eq!(base64(b"f"), "Zg==");
    assert_eq!(base64(b"fo"), "Zm8=");
    assert_eq!(base64(b"foo"), "Zm9v");
    assert_eq!(base64(b"foob"), "Zm9vYg==");
    assert_eq!(base64(b"fooba"), "Zm9vYmE=");
    assert_eq!(base64(b"foobar"), "Zm9vYmFy");
    assert_eq!(data_url(b"foo"), "data:image/png;base64,Zm9v");
}

#[test]
fn answer_logo_names_the_form_agreeing_orders_and_recounts_the_plan() {
    let dir = temp_dir("logo-answer");
    let state = state_with_logo_draft(&dir);

    // Before the answer: the prompt's logo, the courier buttons, a pick with no orders.
    let before = state.with_draft(|open| read_result(open)).expect("draft");
    assert_eq!(before.logos.len(), 1);
    assert_eq!(before.logos[0].hash, "ffffffffffffffff");
    assert_eq!(before.logos[0].pages, 2);
    assert_eq!(before.logos[0].orders, 2);
    assert_eq!(
        before.logos[0].picture.as_deref(),
        Some("data:image/png;base64,Zm9v")
    );
    assert_eq!(
        before.couriers,
        vec!["J&T Express", "SiCepat", "J&T Cargo", "IDX", "SPX"]
    );
    assert_eq!(before.plan.rows[0].orders, 0);
    assert_eq!(before.plan.rows[0].number, None);
    assert_eq!(before.plan.rows[1].orders, 2); // the rest row takes both
    assert!(before.warnings[0].contains("Courier unknown for 2 orders"));

    // The JY form names J&T Express; the shared 00 form does not.
    let after = answer_logo_impl(
        &state,
        "ffffffffffffffff",
        &LogoAnswer::Courier {
            name: "J&T Express".to_string(),
        },
    )
    .expect("answer");
    assert!(after.logos.is_empty(), "the answered logo is spent");
    assert_eq!(after.plan.rows[0].orders, 1); // the courier pick takes the JY order
    assert_eq!(after.plan.rows[0].number, Some(PdfNumber::new(1)));
    assert_eq!(after.plan.rows[1].orders, 1);
    assert_eq!(after.plan.rows[1].number, Some(PdfNumber::new(2)));
    assert_eq!(after.orders, before.orders); // no order left or joined the batch
    assert!(after.warnings[0].contains("Courier unknown for 1 order: 000000000202"));

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn skip_and_not_a_courier_leave_the_orders_unknown() {
    for answer in [LogoAnswer::Skip, LogoAnswer::NotCourier] {
        let dir = temp_dir("logo-skip");
        let state = state_with_logo_draft(&dir);
        let result = answer_logo_impl(&state, "ffffffffffffffff", &answer).expect("answer");

        assert!(result.logos.is_empty(), "the answered logo is spent");
        assert_eq!(result.plan.rows[0].orders, 0);
        assert!(result.warnings[0].contains("Courier unknown for 2 orders"));
        // Nothing was applied: the orders still have no courier.
        state
            .with_draft(|open| {
                assert_eq!(open.draft.orders[0].order.courier, "");
                assert_eq!(open.draft.orders[1].order.courier, "");
            })
            .expect("draft");

        std::fs::remove_dir_all(&dir).ok();
    }
}

#[test]
fn an_unknown_hash_changes_nothing() {
    let dir = temp_dir("logo-unknown");
    let state = state_with_logo_draft(&dir);
    let result = answer_logo_impl(
        &state,
        "0123456789abcdef",
        &LogoAnswer::Courier {
            name: "SiCepat".to_string(),
        },
    )
    .expect("answer");
    assert_eq!(result.logos.len(), 1, "the logo is still asked about");
    assert_eq!(result.plan.rows[0].orders, 0);
    std::fs::remove_dir_all(&dir).ok();
}

// ---------------------------------------- the taught hash in couriers.toml (D-LABEL-4)

#[test]
fn a_courier_answer_remembers_the_logo_in_the_data_folders_file() {
    let dir = temp_dir("logo-remember");
    let state = state_with_logo_draft(&dir);
    answer_logo_impl(
        &state,
        "ffffffffffffffff",
        &LogoAnswer::Courier {
            name: "J&T Express".to_string(),
        },
    )
    .expect("answer");

    let text = std::fs::read_to_string(dir.join(COURIERS_FILE)).expect("couriers.toml written");
    // The bundled file's comments and its other entries are untouched.
    assert!(text.contains("975x377 px"), "comments kept:\n{text}");
    assert!(
        text.contains("# its base image is one flat colour"),
        "{text}"
    );
    assert!(
        text.contains("logo = [\"39b9b7b9f61d0e7f\", \"ffffffffffffffff\"]"),
        "{text}"
    );

    // The next read uses the file: the logo is known.
    assert!(
        load_couriers(&dir)
            .expect("reload")
            .knows_byte_hash("ffffffffffffffff")
    );
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn a_not_a_courier_answer_goes_to_the_top_level_list() {
    let dir = temp_dir("logo-not-courier");
    let state = state_with_logo_draft(&dir);
    answer_logo_impl(&state, "ffffffffffffffff", &LogoAnswer::NotCourier).expect("answer");
    let text = std::fs::read_to_string(dir.join(COURIERS_FILE)).expect("written");
    assert!(
        text.contains("not_courier = [\"ffffffffffffffff\"]"),
        "{text}"
    );
    assert!(
        load_couriers(&dir)
            .expect("reload")
            .knows_byte_hash("ffffffffffffffff")
    );
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn skip_and_an_unknown_hash_write_no_courier_file() {
    let dir = temp_dir("logo-no-write");
    let state = state_with_logo_draft(&dir);
    answer_logo_impl(&state, "ffffffffffffffff", &LogoAnswer::Skip).expect("answer");
    assert!(
        !dir.join(COURIERS_FILE).exists(),
        "*I don't know* writes nothing"
    );
    answer_logo_impl(
        &state,
        "0123456789abcdef",
        &LogoAnswer::Courier {
            name: "SiCepat".to_string(),
        },
    )
    .expect("answer");
    assert!(
        !dir.join(COURIERS_FILE).exists(),
        "an unknown hash writes nothing"
    );
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn a_broken_couriers_file_stops_the_read_with_its_line() {
    let dir = temp_dir("couriers-broken");
    std::fs::write(
        dir.join(CATEGORIES_FILE),
        "[[category]]\ncode = \"Z\"\nname = \"Rest\"\n",
    )
    .expect("write categories");
    // An unterminated array: the TOML parser names the line it stopped at.
    std::fs::write(
        dir.join(COURIERS_FILE),
        "not_courier = [\"aa11\"\n[[courier]]\nname = \"X\"\n",
    )
    .expect("write couriers");

    let state = AppState::new();
    state.set_data_folder(dir.clone());
    state.set_day("2026-10-07");
    let error = read_draft(
        &state,
        vec![dir.join("a.pdf")],
        &Grouping::default(),
        |_| {},
    )
    .err()
    .expect("the read must stop");
    assert!(
        error.starts_with("The couriers have an error: couriers.toml: "),
        "{error}"
    );
    assert!(error.contains("line"), "{error}");
    std::fs::remove_dir_all(&dir).ok();
}
