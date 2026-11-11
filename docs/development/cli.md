# Command line tools

`packing-cli` runs the batch steps of the PC app from the command line, for tests and sample runs;
`packing-fixtures` writes the made-up label PDFs of `testdata/`. Neither is part of the app the owner
uses ([app.md](app.md)).

## `packing-cli`

From the repository root, Git Bash. It calls the same `packing-batch` steps as the app, on the same
data folder (one folder per date, [data/](../software-design/data/README.md)).

| Command | Does |
|---|---|
| `cargo run -p packing-cli -- prepare --labels <file>... [options]` | Read the label files and save one batch into `<data>/<day>/` (`journal.jsonl`, `input/`, `output/`), like the app's Save |
| `cargo run -p packing-cli -- revert [options]` | Undo the newest batch of the date |
| `cargo run -p packing-cli -- amend [options]` | Redo the newest batch from its stored inputs, with the sort rule it was made with |
| `cargo run -p packing-cli -- dump --labels <file>... [--pdfium <dir>]` | Read the files with the bundled rules and print what that gives as JSON (the files, the counts, every copy, the unknown header images, the read warnings); saves nothing |

| Option | Meaning |
|---|---|
| `--labels <file>...` | One or more label PDFs of this batch (`prepare`, `dump`); read in file-name order |
| `--day YYYY-MM-DD` | The date (default: today) |
| `--data <dir>` | The data folder (default: the `data_dir` of `packing-settings.toml`, else `packing-data`, inside `--program`) |
| `--program <dir>` | The folder with `packing-settings.toml` (the accounts) and `packing-workflows.toml` (the sort rules); a missing file means no accounts and the bundled rules; the command writes neither (default: `.`) |
| `--rule <name>` | The sort rule of a new batch (default: `default`) |
| `--account <name>` | Put every order whose shop no account lists, and every file whose labels show no sender, under this account for this run only. Without it such an order stops the command and the message names the shops |
| `--save-anyway` | Save a batch the verifier found something in, recording the findings; without it the findings stop the save |
| `--pdfium <dir>` | The folder with `pdfium.dll` (default: next to the exe, else `vendor/`) |
| `--quiet-ids` | Print counts only; no order IDs, tracking IDs, shops or names |

`cargo run -p packing-cli -- --help` prints the same usage. `dump` prints real order data when given a
sample file: its output goes to `work/` only.

## `packing-fixtures`

`cargo run -p packing-fixtures [dir]` writes into `dir` (default: `testdata/`) `labels-slip.pdf` and
`labels-plain.pdf` with their answer `expected-labels.json`, and `labels-repeat-1.pdf` and
`labels-repeat-2.pdf` — an order printed twice in a row, an order in both — with `expected-repeat.json`. The orders are Rust data in
`crates/fixtures/src/orders.rs`. A change to them, or to how the pages are drawn, is committed together
with the files this command writes; until then the crate's test fails ([testing.md](testing.md)).
