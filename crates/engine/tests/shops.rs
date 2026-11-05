//! The shop check over a batch's copies: per file, the orders none of the workspace's shops sent
//! and who sent them.

use packing_engine::copies::Copy;
use packing_engine::label::find_sender;
use packing_engine::shops::{OtherSender, OtherShopFile, WorkspaceShops, other_shop_files};
use packing_engine::text::PageText;

fn copy(input: usize, order: &str, sender: Option<&str>) -> Copy {
    let mut copy = testkit::copy(order, input, 1, Vec::new());
    copy.sender = sender.and_then(|name| {
        find_sender(&PageText {
            text: format!("Pengirim : {name}"),
            ..PageText::default()
        })
    });
    copy
}

#[test]
fn each_file_names_its_orders_of_another_shop_and_who_sent_them() {
    let own = vec![
        "Toko Contoh".to_string(),
        "Toko Contoh Official".to_string(),
    ];
    let others = vec![WorkspaceShops {
        workspace: "Lain".to_string(),
        shops: vec!["Toko Lain".to_string()],
    }];
    let copies = [
        copy(0, "580000000000000001", Some("Toko Contoh")),
        copy(0, "580000000000000002", Some("TOKO CONTOH OFFICIAL")),
        copy(1, "580000000000000003", Some("Toko Contoh")),
        copy(1, "580000000000000004", None),
        copy(1, "580000000000000004", None),
        copy(2, "580000000000000005", Some("Toko Baru")),
        copy(2, "580000000000000006", Some("Toko Lain")),
        copy(2, "580000000000000007", None),
    ];
    let want = vec![
        OtherShopFile {
            input: 1,
            orders: 1,
            of_orders: 2,
            sender: OtherSender::NotFound,
        },
        OtherShopFile {
            input: 2,
            orders: 3,
            of_orders: 3,
            sender: OtherSender::OfWorkspace("Lain".to_string()),
        },
    ];
    assert_eq!(other_shop_files(&copies, &own, &others), want);
    assert_eq!(other_shop_files(&copies, &[], &others), Vec::new());
}
