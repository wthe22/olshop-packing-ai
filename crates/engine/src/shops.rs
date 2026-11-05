//! The shop check before the plan (02-read.md step 6, workspaces.md *Its shops*): the files whose
//! labels show none of the workspace's shops.

use std::collections::BTreeSet;

use crate::copies::Copy;
use crate::ids::OrderId;

/// One workspace's list of shops, as its `settings.toml` holds it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceShops {
    pub workspace: String,
    pub shops: Vec<String>,
}

/// Who sent the labels of a file that are not the workspace's: the first that applies, in this
/// order.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum OtherSender {
    /// A shop another workspace lists.
    OfWorkspace(String),
    /// A sender no workspace lists.
    Unlisted,
    /// No `Pengirim` caption on the label.
    NotFound,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OtherShopFile {
    /// The input, by its place in the batch's reading order.
    pub input: usize,
    /// The file's orders with a label the workspace's shops did not send.
    pub orders: usize,
    /// All the file's orders.
    pub of_orders: usize,
    pub sender: OtherSender,
}

/// The files, in reading order, with labels none of `own` sent; none when `own` is empty. `others`
/// are the other workspaces' lists, which say whose shop a label is.
pub fn other_shop_files(
    copies: &[Copy],
    own: &[String],
    others: &[WorkspaceShops],
) -> Vec<OtherShopFile> {
    if own.is_empty() {
        return Vec::new();
    }
    let inputs = copies
        .iter()
        .map(|copy| copy.input)
        .max()
        .map_or(0, |n| n + 1);
    (0..inputs)
        .filter_map(|input| {
            let of_file: Vec<&Copy> = copies.iter().filter(|c| c.input == input).collect();
            let all: BTreeSet<&OrderId> = of_file.iter().map(|c| &c.order_id).collect();
            let mut other: BTreeSet<&OrderId> = BTreeSet::new();
            let mut sender: Option<OtherSender> = None;
            for copy in &of_file {
                if let Some(found) = other_sender(copy, own, others) {
                    other.insert(&copy.order_id);
                    sender = Some(sender.map_or(found.clone(), |s| s.min(found)));
                }
            }
            sender.map(|sender| OtherShopFile {
                input,
                orders: other.len(),
                of_orders: all.len(),
                sender,
            })
        })
        .collect()
}

/// Who sent `copy` when none of `own` did.
fn other_sender(copy: &Copy, own: &[String], others: &[WorkspaceShops]) -> Option<OtherSender> {
    let Some(text) = &copy.sender else {
        return Some(OtherSender::NotFound);
    };
    if own.iter().any(|shop| text.shows(shop)) {
        return None;
    }
    let owner = others
        .iter()
        .find(|other| other.shops.iter().any(|shop| text.shows(shop)));
    Some(owner.map_or(OtherSender::Unlisted, |other| {
        OtherSender::OfWorkspace(other.workspace.clone())
    }))
}
