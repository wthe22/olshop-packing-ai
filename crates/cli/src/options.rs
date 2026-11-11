use std::path::{Path, PathBuf};

use jiff::civil::Date;

use crate::error::CliError;

/// The parsed options shared by the commands.
#[derive(Debug)]
pub struct Options {
    pub labels: Vec<PathBuf>,
    pub day: Option<Date>,
    pub data: Option<PathBuf>,
    pub program: PathBuf,
    pub rule: String,
    pub account: Option<String>,
    pub pdfium: Option<PathBuf>,
    pub quiet_ids: bool,
    pub save_anyway: bool,
}

impl Options {
    pub fn parse(args: &[String]) -> Result<Options, CliError> {
        let mut parsed = Options {
            labels: Vec::new(),
            day: None,
            data: None,
            program: PathBuf::from("."),
            rule: "default".to_string(),
            account: None,
            pdfium: None,
            quiet_ids: false,
            save_anyway: false,
        };
        let mut index = 0;
        while index < args.len() {
            let flag = args[index].as_str();
            index += 1;
            match flag {
                "--labels" => collect_labels(args, &mut index, &mut parsed.labels),
                "--day" => parsed.day = Some(parse_day(&value(args, &mut index, flag)?)?),
                "--data" => parsed.data = Some(PathBuf::from(value(args, &mut index, flag)?)),
                "--program" => parsed.program = PathBuf::from(value(args, &mut index, flag)?),
                "--rule" => parsed.rule = value(args, &mut index, flag)?,
                "--account" => parsed.account = Some(value(args, &mut index, flag)?),
                "--pdfium" => parsed.pdfium = Some(PathBuf::from(value(args, &mut index, flag)?)),
                "--quiet-ids" => parsed.quiet_ids = true,
                "--save-anyway" => parsed.save_anyway = true,
                other => return Err(CliError::Usage(format!("unknown option \"{other}\""))),
            }
        }
        Ok(parsed)
    }

    pub fn day(&self) -> Date {
        self.day.unwrap_or_else(|| jiff::Zoned::now().date())
    }

    /// The label files in file-name order, which is download order.
    pub fn labels_in_order(&self) -> Result<Vec<PathBuf>, CliError> {
        if self.labels.is_empty() {
            return Err(CliError::Usage(
                "this command needs at least one --labels <file>".to_string(),
            ));
        }
        let mut labels = self.labels.clone();
        labels.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
        Ok(labels)
    }
}

fn parse_day(text: &str) -> Result<Date, CliError> {
    text.parse()
        .map_err(|_| CliError::Usage(format!("\"{text}\" is not a day; expected YYYY-MM-DD")))
}

fn value(args: &[String], index: &mut usize, flag: &str) -> Result<String, CliError> {
    let value = args
        .get(*index)
        .filter(|value| !value.starts_with("--"))
        .ok_or_else(|| CliError::Usage(format!("{flag} needs a value")))?;
    *index += 1;
    Ok(value.clone())
}

/// Every `--labels` value up to the next flag.
fn collect_labels(args: &[String], index: &mut usize, labels: &mut Vec<PathBuf>) {
    while *index < args.len() && !args[*index].starts_with("--") {
        labels.push(PathBuf::from(&args[*index]));
        *index += 1;
    }
}

/// The `pdfium.dll` folder: `--pdfium`, else the folder of the exe, else `vendor` next to the
/// crate (found at build time), so `cargo run` and the built binary both work.
pub fn pdfium_dir(override_dir: Option<&Path>) -> PathBuf {
    if let Some(dir) = override_dir {
        return dir.to_path_buf();
    }
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
        && dir.join("pdfium.dll").exists()
    {
        return dir.to_path_buf();
    }
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn parses_the_defaults() {
        let parsed = Options::parse(&args(&["--labels", "b.pdf", "a.pdf"])).expect("parse");
        assert_eq!(parsed.rule, "default");
        assert_eq!(parsed.program, PathBuf::from("."));
        assert!(parsed.day.is_none() && parsed.data.is_none() && parsed.account.is_none());
        let order = parsed.labels_in_order().expect("labels");
        assert_eq!(order, [PathBuf::from("a.pdf"), PathBuf::from("b.pdf")]);
    }

    #[test]
    fn parses_every_option() {
        let parsed = Options::parse(&args(&[
            "--day",
            "2026-10-07",
            "--data",
            "d",
            "--program",
            "p",
            "--rule",
            "r",
            "--account",
            "Toko Alfa",
            "--pdfium",
            "v",
            "--quiet-ids",
            "--save-anyway",
        ]))
        .expect("parse");
        assert_eq!(parsed.day, Some(jiff::civil::date(2026, 10, 7)));
        assert_eq!(parsed.data, Some(PathBuf::from("d")));
        assert_eq!(parsed.account.as_deref(), Some("Toko Alfa"));
        assert!(parsed.quiet_ids && parsed.save_anyway);
    }

    #[test]
    fn refuses_what_it_cannot_use() {
        for bad in [
            &["--day", "yesterday"][..],
            &["--data"],
            &["--nope"],
            &["--rule", "--quiet-ids"],
        ] {
            assert!(matches!(
                Options::parse(&args(bad)),
                Err(CliError::Usage(_))
            ));
        }
        assert!(Options::parse(&[]).unwrap().labels_in_order().is_err());
    }
}
