//! The shop check of a read (new-batch.md, *The shop prompt*): the workspace's shops and the
//! other workspaces' lists, read when labels are read, and the files the prompt names.

use packing_engine::shops::{OtherSender, OtherShopFile, WorkspaceShops};
use packing_store::{other_workspace_shops, workspace_shops};
use serde::Serialize;

use crate::{AppState, OpenDraft};

/// One file the shop prompt names.
#[derive(Serialize)]
pub struct OtherShopView {
    file: String,
    orders: usize,
    of_orders: usize,
    sender: SenderView,
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum SenderView {
    OfWorkspace { workspace: String },
    Unlisted,
    NotFound,
}

/// The open workspace's shops, and every other workspace's; none when it lists none, since then
/// nothing is checked.
pub(super) fn shop_lists(state: &AppState) -> Result<(Vec<String>, Vec<WorkspaceShops>), String> {
    state.with_data(|data| {
        let Some(workspace) = &data.workspace else {
            return Ok((Vec::new(), Vec::new()));
        };
        let own = workspace_shops(&data.dir, workspace).map_err(|e| e.to_string())?;
        if own.is_empty() {
            return Ok((own, Vec::new()));
        }
        let others = other_workspace_shops(&data.dir, workspace).map_err(|e| e.to_string())?;
        Ok((own, others))
    })
}

pub(super) fn other_shops_view(open: &OpenDraft) -> Vec<OtherShopView> {
    open.other_shops
        .iter()
        .map(|file: &OtherShopFile| OtherShopView {
            file: open.draft.files[file.input].filename.clone(),
            orders: file.orders,
            of_orders: file.of_orders,
            sender: match &file.sender {
                OtherSender::OfWorkspace(workspace) => SenderView::OfWorkspace {
                    workspace: workspace.clone(),
                },
                OtherSender::Unlisted => SenderView::Unlisted,
                OtherSender::NotFound => SenderView::NotFound,
            },
        })
        .collect()
}
