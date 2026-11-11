//! The two files beside the program that the batch steps read: `packing-settings.toml` for the
//! data folder and the accounts, `packing-workflows.toml` for the sort rules. The command line
//! writes neither.

use std::fs;
use std::path::{Path, PathBuf};

use packing_engine::accounts::Account;
use packing_engine::workflows::{BUNDLED_WORKFLOWS, WORKFLOWS_FILE, Workflows, load_workflows};
use serde::Deserialize;

use crate::error::CliError;

const SETTINGS_FILE: &str = "packing-settings.toml";

/// The data folder a program folder without a settings file uses, as the app offers it.
const DEFAULT_DATA_DIR: &str = "packing-data";

#[derive(Default, Deserialize)]
pub struct Settings {
    data_dir: Option<PathBuf>,
    #[serde(default)]
    pub accounts: Vec<Account>,
}

impl Settings {
    /// The file in `program`; no accounts and the default data folder when it is missing.
    pub fn load(program: &Path) -> Result<Settings, CliError> {
        let path = program.join(SETTINGS_FILE);
        match fs::read_to_string(&path) {
            Ok(text) => toml::from_str(&text)
                .map_err(|error| CliError::Program(format!("{SETTINGS_FILE}: {error}"))),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Settings::default()),
            Err(error) => Err(CliError::Program(format!(
                "cannot read {}: {error}",
                path.display()
            ))),
        }
    }

    /// The data folder: `--data`, else the one the settings name, else `packing-data`, a relative
    /// path taken from the program folder.
    pub fn data_dir(&self, program: &Path, chosen: Option<&Path>) -> PathBuf {
        match chosen {
            Some(dir) => dir.to_path_buf(),
            None => program.join(
                self.data_dir
                    .as_deref()
                    .unwrap_or(Path::new(DEFAULT_DATA_DIR)),
            ),
        }
    }
}

/// The rules in `program`; the bundled copy when the file is missing.
pub fn workflows(program: &Path) -> Result<Workflows, CliError> {
    let path = program.join(WORKFLOWS_FILE);
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => BUNDLED_WORKFLOWS.to_string(),
        Err(error) => {
            return Err(CliError::Program(format!(
                "cannot read {}: {error}",
                path.display()
            )));
        }
    };
    load_workflows(&text).map_err(|error| CliError::Program(error.to_string()))
}

#[cfg(test)]
mod tests {
    use testkit::TempDir;

    use super::*;

    #[test]
    fn a_missing_program_folder_gives_the_defaults() {
        let program = TempDir::new("cli-program-none");
        let settings = Settings::load(program.path()).expect("settings");
        assert!(settings.accounts.is_empty());
        assert_eq!(
            settings.data_dir(program.path(), None),
            program.path().join("packing-data")
        );
        let chosen = Path::new("elsewhere");
        assert_eq!(settings.data_dir(program.path(), Some(chosen)), chosen);
        let rules = workflows(program.path()).expect("bundled rules");
        assert!(rules.sort_rule("default").is_some());
    }

    #[test]
    fn the_settings_name_the_data_folder_and_the_accounts() {
        let program = TempDir::new("cli-program-settings");
        fs::write(
            program.path().join(SETTINGS_FILE),
            "data_dir = \"data\"\n[[accounts]]\nname = \"Toko Alfa\"\nshops = [\"Toko Alfa Official\"]\n",
        )
        .unwrap();
        let settings = Settings::load(program.path()).expect("settings");
        assert_eq!(settings.accounts[0].shops, ["Toko Alfa Official"]);
        assert_eq!(
            settings.data_dir(program.path(), None),
            program.path().join("data")
        );
    }

    #[test]
    fn a_broken_file_is_an_error() {
        let program = TempDir::new("cli-program-broken");
        fs::write(program.path().join(SETTINGS_FILE), "accounts = 3\n").unwrap();
        fs::write(program.path().join(WORKFLOWS_FILE), "not a rules file").unwrap();
        assert!(Settings::load(program.path()).is_err());
        assert!(workflows(program.path()).is_err());
    }
}
