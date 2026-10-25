# Command line tools

`packing-cli` runs the batch steps of the PC app from the command line, for tests and sample runs;
`packing-fixtures` writes the made-up label PDFs of `testdata/`. Neither is part of the app the owner
uses ([app.md](app.md)).

## `packing-cli`

From the repository root, Git Bash:

| Command | Does |
|---|---|
| `cargo run -p packing-cli -- prepare --labels <file>... [--rules <categories.toml>] [--couriers <couriers.toml>] [--data <dir>] [--day YYYY-MM-DD] [--packing-list whole\|per-pdf\|none] [--layout pick,full,summary] [--pdfium <dir>] [--quiet-ids]` | Read the label files, apply the picks and save one batch, like the app's Save |
| `cargo run -p packing-cli -- revert [--day <day>] [--data <dir>] [--pdfium <dir>] [--quiet-ids]` | Undo the last batch of the day |
| `cargo run -p packing-cli -- amend [--day <day>] [--data <dir>] [--rules <categories.toml>] [--couriers <couriers.toml>] [--pdfium <dir>] [--quiet-ids]` | Redo the last batch of the day in place |
| `cargo run -p packing-cli -- dump --labels <file>... [--couriers <couriers.toml>] [--pdfium <dir>]` | Read the files as the first batch of an empty day and print what that gives as JSON (the files, the counts, every order, the read warnings); saves nothing. Without `--couriers`, the bundled courier list |

| Flag | Meaning |
|---|---|
| `--labels <file>...` | One or more label PDFs of this batch (required for `prepare` and `dump`) |
| `--rules <categories.toml>` | The picks, in order (default: `<data>/categories.toml`; without the file, the one catch-all pick) |
| `--couriers <couriers.toml>` | The courier data (default: `<data>/couriers.toml`, else the bundled list) |
| `--day YYYY-MM-DD` | The day folder (default: today) |
| `--data <dir>` | The data folder holding `categories.toml`, `couriers.toml` and `labels/` (default: `.`) |
| `--packing-list <scope>` | `whole` \| `per-pdf` \| `none` (default: `whole`) |
| `--layout <list>` | Comma-separated `pick`, `full`, `summary` (default: `pick`) |
| `--pdfium <dir>` | The folder with `pdfium.dll` (default: next to the exe, else `vendor/`) |
| `--quiet-ids` | Print counts only; no order IDs, tracking IDs or names |

`cargo run -p packing-cli -- --help` prints the same usage. `prepare` writes the batch to
`<data>/labels/<day>/batch <b>/` — the saved PDFs, the packing lists, `summary.txt` and a copy of
the label files read (the v0.1 day folder; the target layout is [data/](../software-design/data/README.md)); the run's counts go
to the console. `dump` prints real order data when given a sample file: its output goes to `work/` only.

## `packing-fixtures`

`cargo run -p packing-fixtures [dir]` writes `labels-slip.pdf`, `labels-plain.pdf` and their answer
`expected-labels.json` into `dir` (default: `testdata/`). The orders are Rust data in
`crates/fixtures/src/orders.rs`. A change to them, or to how the pages are drawn, is committed together
with the files this command writes; until then the crate's test fails ([testing.md](testing.md)).
