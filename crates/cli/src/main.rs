//! Developer CLI: the batch steps of the PC app from the command line, for tests and sample
//! runs (`packing-cli` calls the same `packing-batch` steps, not for daily use).

mod accounts;
mod commands;
mod dump;
mod error;
mod options;
mod program;

use error::CliError;
use options::Options;

const USAGE: &str = "\
packing-cli — developer command line for the packing app (not for daily use)

USAGE:
  packing-cli prepare --labels <file>... [options]
  packing-cli revert  [options]
  packing-cli amend   [options]
  packing-cli dump    --labels <file>... [--pdfium <dir>]

OPTIONS:
  --labels <file>...      One or more label PDFs of this batch (prepare, dump)
  --day YYYY-MM-DD        The date to work on (default: today)
  --data <dir>            The data folder, holding one folder per date, parser.toml (couriers)
                          and config.toml (accounts, sort rules); a missing file means the
                          bundled copy (default: the `data_dir` of packing-app.toml, else
                          packing-data, both inside --program)
  --program <dir>         The folder with packing-app.toml (default: .)
  --rule <name>           The sort rule of a new batch (default: default); an amend keeps the
                          rule its batch was made with
  --account <name>        Put every order whose shop no account lists, and every file with
                          labels that show no sender, under this account for this run only;
                          without it such an order stops the command, naming the shops
  --save-anyway           Save a batch the verifier found something in, recording the findings
  --pdfium <dir>          The folder with pdfium.dll (default: next to the exe, else vendor)
  --quiet-ids             Print counts only; no order IDs, tracking IDs, shops or names

prepare reads the files and saves a batch into <data>/<day>/ (journal.jsonl, input/, output/).
revert undoes the newest batch of the date. amend redoes it from its stored inputs. dump saves
nothing: it prints what reading the files gives as JSON, read with the bundled rules.
";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(command) = args.first().map(String::as_str) else {
        return print!("{USAGE}");
    };
    let result = match command {
        "--help" | "-h" => return print!("{USAGE}"),
        "prepare" => run(&args[1..], commands::prepare),
        "revert" => run(&args[1..], commands::revert_last),
        "amend" => run(&args[1..], commands::amend_last),
        "dump" => run(&args[1..], dump::run),
        other => Err(CliError::Usage(format!(
            "unknown command \"{other}\"; expected prepare, revert, amend or dump"
        ))),
    };
    if let Err(error) = result {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run(args: &[String], command: fn(&Options) -> Result<(), CliError>) -> Result<(), CliError> {
    command(&Options::parse(args)?)
}
