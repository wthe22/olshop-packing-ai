//! A workspace's shops (workspaces.md, *Its shops*): `{workspace}/settings.toml`, the owner's list of
//! the shops whose labels belong in the workspace. Read when labels are read, never by the fold.

use std::fs;
use std::path::Path;

use packing_engine::record::SCHEMA;
use packing_engine::shops::WorkspaceShops;
use serde::{Deserialize, Serialize};

use crate::StoreError;
use crate::files::write_atomic;
use crate::workspaces::workspaces;

pub const WORKSPACE_SETTINGS: &str = "settings.toml";

#[derive(Serialize, Deserialize)]
struct Root {
    schema: String,
    #[serde(default)]
    shops: Vec<String>,
}

/// The shops `workspace` lists; none when it has no `settings.toml`.
pub fn workspace_shops(data_dir: &Path, workspace: &str) -> Result<Vec<String>, StoreError> {
    let path = data_dir.join(workspace).join(WORKSPACE_SETTINGS);
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(StoreError::io(&path, error)),
    };
    let root: Root = toml::from_str(&text).map_err(|error| StoreError::BadFile {
        path: path.clone(),
        reason: error.message().to_string(),
    })?;
    Ok(root.shops)
}

/// Write the shops of `workspace`, each name trimmed, and return them. An empty name, or a name
/// listed twice (spaces and upper and lower case ignored), is refused and nothing is written. The
/// caller holds the data folder's lock.
pub fn set_workspace_shops(
    data_dir: &Path,
    workspace: &str,
    shops: &[String],
) -> Result<Vec<String>, StoreError> {
    let shops: Vec<String> = shops.iter().map(|name| name.trim().to_string()).collect();
    for (index, name) in shops.iter().enumerate() {
        let refuse = |reason: &str| StoreError::BadName {
            name: name.clone(),
            reason: reason.to_string(),
        };
        if name.is_empty() {
            return Err(refuse("a shop needs a name"));
        }
        if shops[..index].iter().any(|other| same(other, name)) {
            return Err(refuse("that shop is listed twice"));
        }
    }
    let root = Root {
        schema: SCHEMA.to_string(),
        shops,
    };
    let text = toml::to_string(&root).expect("the shops are plain TOML");
    write_atomic(
        &data_dir.join(workspace).join(WORKSPACE_SETTINGS),
        text.as_bytes(),
    )?;
    Ok(root.shops)
}

/// The shops of every workspace but `workspace`, to say whose shop a label is.
pub fn other_workspace_shops(
    data_dir: &Path,
    workspace: &str,
) -> Result<Vec<WorkspaceShops>, StoreError> {
    workspaces(data_dir)?
        .into_iter()
        .filter(|other| other != workspace)
        .map(|other| {
            Ok(WorkspaceShops {
                shops: workspace_shops(data_dir, &other)?,
                workspace: other,
            })
        })
        .collect()
}

fn same(a: &str, b: &str) -> bool {
    let squash = |text: &str| text.split_whitespace().collect::<String>().to_lowercase();
    squash(a) == squash(b)
}
