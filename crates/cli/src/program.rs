//! What the batch steps read besides the labels: `packing-app.toml` beside the program for the
//! data folder, and `parser.toml` and `config.toml` in the data folder for the couriers, the
//! accounts and the sort rules. The command line writes none of them.

use std::fs;
use std::path::{Path, PathBuf};

use packing_engine::workflows::{
    BUNDLED_CONFIG, BUNDLED_PARSER, CONFIG_FILE, PARSER_FILE, Workflows, load_workflows,
};
use serde::Deserialize;

use crate::error::CliError;

const APP_FILE: &str = "packing-app.toml";

/// The data folder a program folder without an app file uses, as the app offers it.
const DEFAULT_DATA_DIR: &str = "packing-data";

#[derive(Default, Deserialize)]
struct AppFile {
    data_dir: Option<PathBuf>,
}

/// The data folder: `--data`, else the one `packing-app.toml` names, else `packing-data`, a
/// relative path taken from the program folder.
pub fn data_dir(program: &Path, chosen: Option<&Path>) -> Result<PathBuf, CliError> {
    if let Some(dir) = chosen {
        return Ok(dir.to_path_buf());
    }
    let app: AppFile = match read(&program.join(APP_FILE))? {
        Some(text) => toml::from_str(&text)
            .map_err(|error| CliError::Program(format!("{APP_FILE}: {error}")))?,
        None => AppFile::default(),
    };
    Ok(program.join(
        app.data_dir
            .as_deref()
            .unwrap_or(Path::new(DEFAULT_DATA_DIR)),
    ))
}

/// The couriers, accounts and sort rules of the data folder; a bundled copy stands in for a file
/// that is missing.
pub fn workflows(data: &Path) -> Result<Workflows, CliError> {
    let parser = read(&data.join(PARSER_FILE))?.unwrap_or_else(|| BUNDLED_PARSER.to_string());
    let config = read(&data.join(CONFIG_FILE))?.unwrap_or_else(|| BUNDLED_CONFIG.to_string());
    load_workflows(&parser, &config).map_err(|error| CliError::Program(error.to_string()))
}

/// The file's text; none when it is not there.
fn read(path: &Path) -> Result<Option<String>, CliError> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(CliError::Program(format!(
            "cannot read {}: {error}",
            path.display()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use testkit::TempDir;

    use super::*;

    #[test]
    fn without_files_the_defaults_and_the_bundled_copies_stand_in() {
        let program = TempDir::new("cli-program-none");
        assert_eq!(
            data_dir(program.path(), None).unwrap(),
            program.path().join("packing-data")
        );
        let chosen = Path::new("elsewhere");
        assert_eq!(data_dir(program.path(), Some(chosen)).unwrap(), chosen);
        let rules = workflows(program.path()).expect("bundled files");
        assert!(rules.sort_rule("default").is_some() && rules.accounts.is_empty());
    }

    #[test]
    fn the_app_file_names_the_data_folder_and_the_data_folder_the_accounts() {
        let program = TempDir::new("cli-program-files");
        fs::write(program.path().join(APP_FILE), "data_dir = \"data\"\n").unwrap();
        assert_eq!(
            data_dir(program.path(), None).unwrap(),
            program.path().join("data")
        );
        let config = format!(
            "{BUNDLED_CONFIG}[[accounts]]\nname = \"Toko Alfa\"\nshops = [\"Toko Alfa Official\"]\n"
        );
        fs::write(program.path().join(CONFIG_FILE), config).unwrap();
        let rules = workflows(program.path()).expect("files");
        assert_eq!(rules.accounts[0].shops, ["Toko Alfa Official"]);
    }

    #[test]
    fn a_broken_file_is_an_error() {
        let program = TempDir::new("cli-program-broken");
        fs::write(program.path().join(APP_FILE), "data_dir = 3\n").unwrap();
        fs::write(program.path().join(PARSER_FILE), "not a parser file").unwrap();
        assert!(data_dir(program.path(), None).is_err());
        assert!(workflows(program.path()).is_err());
    }
}
