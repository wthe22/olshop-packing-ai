//! The sets of the data folder: the groupings list, **+ New set** and **Delete set**.

use std::path::Path;

use packing_engine::groupings::{self, Grouping};
use serde::Serialize;

use crate::commands::day::settings_load;

/// One grouping of the data folder as the Categories screen lists it: its name and the file it is kept in, relative to the data folder.
#[derive(Serialize)]
pub struct GroupingView {
    pub(super) name: Grouping,
    pub(super) file: String,
}

/// The groupings of a data folder:
/// every grouping with its file, and the grouping used last, which the next **New batch**
/// preselects.
#[derive(Serialize)]
pub struct GroupingsList {
    pub(super) groupings: Vec<GroupingView>,
    /// The grouping used last, from the data folder's `settings.json`;
    /// `default` when the file has none or cannot be read.
    pub(super) last_grouping: Grouping,
}

pub(super) fn groupings_list_impl(folder: &Path) -> Result<GroupingsList, String> {
    let groupings = groupings::list(folder).map_err(|error| error.to_string())?;
    let last_grouping = settings_load(folder)
        .map(|settings| settings.grouping)
        .unwrap_or_else(|_| Grouping::default());
    Ok(GroupingsList {
        groupings: groupings
            .into_iter()
            .map(|entry| GroupingView {
                name: entry.name,
                file: entry.file,
            })
            .collect(),
        last_grouping,
    })
}
