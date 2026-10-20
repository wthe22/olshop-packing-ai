//! The data folder's `settings.json`: the packing-list default and the grouping used last.
//!

use std::path::Path;

use packing_engine::day::{Layout, PackingListChoice, Scope};
use packing_engine::groupings::Grouping;
use serde::{Deserialize, Serialize};

use crate::SETTINGS_FILE;

/// The data folder's `settings.json`: the packing-list default and the grouping
/// used last. A missing key takes its default, so an older file still loads.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_scope")]
    pub scope: Scope,
    #[serde(default = "default_layouts")]
    pub layouts: Vec<Layout>,
    /// The grouping used last: the
    /// [New batch](../ui/new-batch.md) screen preselects it, and each batch records its own.
    /// Absent in an older file, which loads as `default`.
    #[serde(rename = "set", default)]
    pub grouping: Grouping,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            scope: default_scope(),
            layouts: default_layouts(),
            grouping: Grouping::default(),
        }
    }
}

impl Settings {
    /// The packing-list choice a batch starts with.
    pub(crate) fn packing_choice(&self) -> PackingListChoice {
        PackingListChoice {
            scope: self.scope,
            layouts: self.layouts.clone(),
        }
    }
}

fn default_scope() -> Scope {
    Scope::Whole
}

fn default_layouts() -> Vec<Layout> {
    vec![Layout::Pick]
}

/// The settings as the Settings screen sends and receives them: the stored values with the
/// grouping under its own name, so no screen carries the stored key `set`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SettingsView {
    pub(super) scope: Scope,
    pub(super) layouts: Vec<Layout>,
    pub(super) grouping: Grouping,
}

impl SettingsView {
    pub(super) fn of(settings: Settings) -> Self {
        Self {
            scope: settings.scope,
            layouts: settings.layouts,
            grouping: settings.grouping,
        }
    }

    pub(super) fn into_settings(self) -> Settings {
        Settings {
            scope: self.scope,
            layouts: self.layouts,
            grouping: self.grouping,
        }
    }
}

pub(crate) fn settings_load(data_folder: &Path) -> Result<Settings, String> {
    let path = data_folder.join(SETTINGS_FILE);
    if !path.is_file() {
        return Ok(Settings::default());
    }
    let text = std::fs::read_to_string(&path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    serde_json::from_str(&text).map_err(|error| format!("cannot read {}: {error}", path.display()))
}

pub(crate) fn settings_save(data_folder: &Path, settings: &Settings) -> Result<(), String> {
    let path = data_folder.join(SETTINGS_FILE);
    let mut text = serde_json::to_string_pretty(settings).map_err(|error| error.to_string())?;
    text.push('\n');
    std::fs::write(&path, text)
        .map_err(|error| format!("Cannot write to {}: {error}.", path.display()))
}

/// Remember the grouping used last in the data folder's `settings.json`, so the next **New batch** preselects it. Best-effort: a read is never stopped by a
/// settings problem, and a save of the same grouping writes nothing.
pub(crate) fn remember_last_grouping(data_folder: &Path, grouping: &Grouping) {
    let Ok(mut settings) = settings_load(data_folder) else {
        return;
    };
    if settings.grouping == *grouping {
        return;
    }
    settings.grouping = grouping.clone();
    let _ = settings_save(data_folder, &settings);
}
