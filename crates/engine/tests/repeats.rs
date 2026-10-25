//! Copies → orders (03-plan.md steps 7–8, duplicates.md): the already-saved guard, repeats and
//! what they differ in, the groups of inputs, the four choices and what is left out.

use std::collections::BTreeMap;

use packing_engine::copies::Copy;
use packing_engine::ids::{BatchId, OrderId, TrackingId};
use packing_engine::record::{CourierClue, CourierClueKind};
use packing_engine::repeats::{
    ChoiceError, Choices, Difference, GroupChoice, LeftOut, LeftOutBecause, sort_out,
};
use packing_engine::slip::SlipLine;

fn id(order: u32) -> OrderId {
    OrderId::new(format!("5800000000000000{order:02}"))
}

/// A one-page copy of `order` at `page` of `input`; its fingerprint is `print`.
fn copy(order: u32, input: usize, page: u16, print: u64) -> Copy {
    Copy {
        order_id: id(order),
        input,
        page_start: page,
        page_end: page,
        tracking_id: Some(TrackingId::new(format!("JY00000000{order:02}"))),
        courier: None,
        courier_clue: CourierClue {
            kind: CourierClueKind::None,
            value: None,
            form: None,
        },
        ship_by_date: None,
        pickup_code: None,
        lines: vec![SlipLine {
            name: "Sepatu".into(),
            variation: String::new(),
            seller_sku: String::new(),
            quantity: 1,
        }],
        total_quantity: Some(1),
        fingerprints: vec![print],
    }
}

fn repeat_left_out(copy: usize, kept: usize) -> LeftOut {
    LeftOut {
        copy,
        because: LeftOutBecause::Repeat { kept },
    }
}

#[test]
fn orders_with_one_copy_are_kept_as_they_are() {
    let copies = [copy(1, 0, 1, 10), copy(2, 0, 2, 20), copy(3, 1, 1, 30)];
    let orders = sort_out(&copies, &BTreeMap::new(), &Choices::default()).unwrap();
    assert_eq!(orders.kept, [0, 1, 2]);
    assert!(orders.left_out.is_empty() && orders.repeats.is_empty() && orders.groups.is_empty());
}

#[test]
fn a_second_copy_right_after_the_first_is_a_repeat_and_the_first_stays() {
    let copies = [copy(1, 0, 1, 10), copy(1, 0, 2, 10), copy(2, 0, 3, 20)];
    let orders = sort_out(&copies, &BTreeMap::new(), &Choices::default()).unwrap();
    assert_eq!(orders.kept, [0, 2]);
    assert_eq!(orders.left_out, [repeat_left_out(1, 0)]);
    assert_eq!(orders.repeats[0].difference, None, "the same pages");
    assert_eq!(orders.groups.len(), 1);
    assert_eq!(orders.groups[0].inputs, [0]);
    assert_eq!(orders.repeat_counts(&copies), (1, 1, 1));
}

/// Input 0 holds orders 1 and 2, input 1 orders 2 and 3: the two inputs repeat each other.
fn across_two_inputs() -> [Copy; 4] {
    [
        copy(1, 0, 1, 10),
        copy(2, 0, 2, 20),
        copy(2, 1, 1, 20),
        copy(3, 1, 2, 30),
    ]
}

#[test]
fn each_choice_keeps_one_copy_and_leaves_out_the_others() {
    let copies = across_two_inputs();
    let choose = |choice: GroupChoice| Choices {
        groups: BTreeMap::from([(1, choice)]),
        by_hand: BTreeMap::new(),
    };
    let cases = [
        ("keep the first", choose(GroupChoice::KeepFirst), 1, 2),
        ("keep the last", choose(GroupChoice::KeepLast), 2, 1),
        ("take input 0 out", choose(GroupChoice::TakeOut(0)), 2, 1),
        ("take input 1 out", choose(GroupChoice::TakeOut(1)), 1, 2),
        (
            "by hand over the group's choice",
            Choices {
                groups: BTreeMap::from([(1, GroupChoice::KeepFirst)]),
                by_hand: BTreeMap::from([(id(2), 2)]),
            },
            2,
            1,
        ),
    ];
    for (what, choices, kept, left) in cases {
        let orders = sort_out(&copies, &BTreeMap::new(), &choices).expect(what);
        let mut want = vec![0, kept, 3];
        want.sort_unstable();
        assert_eq!(orders.kept, want, "{what}");
        assert_eq!(orders.left_out, [repeat_left_out(left, kept)], "{what}");
        assert_eq!(orders.groups[0].inputs, [0, 1], "{what}");
    }
}

#[test]
fn a_choice_that_cannot_apply_is_refused() {
    let copies = [copy(1, 0, 1, 10), copy(1, 0, 2, 10), copy(2, 1, 1, 20)];
    let take_out = Choices {
        groups: BTreeMap::from([(1, GroupChoice::TakeOut(0))]),
        by_hand: BTreeMap::new(),
    };
    assert_eq!(
        sort_out(&copies, &BTreeMap::new(), &take_out),
        Err(ChoiceError::CannotTakeOut { group: 1, input: 0 })
    );
    let not_a_copy = Choices {
        groups: BTreeMap::new(),
        by_hand: BTreeMap::from([(id(1), 2)]),
    };
    let error = sort_out(&copies, &BTreeMap::new(), &not_a_copy).unwrap_err();
    assert_eq!(
        error,
        ChoiceError::NotACopy {
            order_id: id(1),
            copy: 2
        }
    );
    assert!(error.to_string().contains("copy 2 is not a copy"));
}

#[test]
fn an_order_saved_earlier_loses_every_copy_and_is_no_repeat() {
    let copies = [copy(1, 0, 1, 10), copy(1, 0, 2, 10), copy(2, 0, 3, 20)];
    let saved = BTreeMap::from([(id(1), BatchId::new(1))]);
    let orders = sort_out(&copies, &saved, &Choices::default()).unwrap();
    assert_eq!(orders.kept, [2]);
    let earlier = |copy| LeftOut {
        copy,
        because: LeftOutBecause::SavedEarlier {
            batch: BatchId::new(1),
        },
    };
    assert_eq!(orders.left_out, [earlier(0), earlier(1)]);
    assert!(orders.repeats.is_empty() && orders.groups.is_empty());
}

#[test]
fn the_first_difference_of_a_repeat_is_named() {
    let mut tracking = copy(1, 0, 2, 10);
    tracking.tracking_id = Some(TrackingId::new("JY0000000099"));
    let mut lines = copy(1, 0, 2, 10);
    lines.lines.push(lines.lines[0].clone());
    let mut pages = copy(1, 0, 2, 10);
    pages.fingerprints.push(11);
    let printed = copy(1, 0, 2, 99);
    let cases = [
        (tracking, Difference::TrackingId),
        (lines, Difference::LineCount),
        (pages, Difference::PageCount),
        (printed, Difference::Pages),
    ];
    for (second, want) in cases {
        let copies = [copy(1, 0, 1, 10), second];
        let orders = sort_out(&copies, &BTreeMap::new(), &Choices::default()).unwrap();
        assert_eq!(orders.repeats[0].difference, Some(want));
    }
}

#[test]
fn groups_are_the_connected_inputs_numbered_by_their_lowest() {
    let copies = [
        copy(1, 2, 1, 10),
        copy(1, 3, 1, 10),
        copy(2, 0, 1, 20),
        copy(2, 0, 2, 20),
        copy(3, 3, 2, 30),
        copy(3, 4, 1, 30),
    ];
    let orders = sort_out(&copies, &BTreeMap::new(), &Choices::default()).unwrap();
    let groups: Vec<(usize, Vec<usize>)> = orders
        .groups
        .iter()
        .map(|group| (group.number, group.inputs.clone()))
        .collect();
    assert_eq!(groups, [(1, vec![0]), (2, vec![2, 3, 4])]);
    assert_eq!(orders.groups[1].repeats.len(), 2);
}
