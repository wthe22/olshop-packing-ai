# Command line tools

`packing-cli` runs the batch steps of the PC app from the command line, for tests and sample runs,
and the Python analysis helpers inspect label PDFs and CSVs. Neither is part of the app the owner
uses ([app.md](app.md)); the Python helpers are frozen.

## `packing-cli`

From the repository root, Git Bash:

| Command | Does |
|---|---|
| `cargo run -p packing-cli -- prepare --labels <file>... [--rules <categories.toml>] [--data <dir>] [--day YYYY-MM-DD] [--packing-list whole\|per-pdf\|none] [--layout pick,full,summary] [--pdfium <dir>] [--quiet-ids]` | Read the label files, apply the picks and save one batch, like the app's Save |
| `cargo run -p packing-cli -- revert [--day <day>] [--data <dir>] [--pdfium <dir>] [--quiet-ids]` | Undo the last batch of the day |
| `cargo run -p packing-cli -- amend [--day <day>] [--data <dir>] [--rules <categories.toml>] [--pdfium <dir>] [--quiet-ids]` | Redo the last batch of the day in place |

| Flag | Meaning |
|---|---|
| `--labels <file>...` | One or more label PDFs of this batch (required for `prepare`) |
| `--rules <categories.toml>` | The picks, in order (default: `<data>/categories.toml`) |
| `--day YYYY-MM-DD` | The day folder (default: today) |
| `--data <dir>` | The data folder holding `categories.toml` and `labels/` (default: `.`) |
| `--packing-list <scope>` | `whole` \| `per-pdf` \| `none` (default: `whole`) |
| `--layout <list>` | Comma-separated `pick`, `full`, `summary` (default: `pick`) |
| `--pdfium <dir>` | The folder with `pdfium.dll` (default: next to the exe, else `vendor/`) |
| `--quiet-ids` | Print counts only; no order IDs, tracking IDs or names |

`cargo run -p packing-cli -- --help` prints the same usage. `prepare` writes the batch to
`<data>/labels/<day>/batch <b>/` — the saved PDFs, the packing lists, `summary.txt` and a copy of
the label files read ([data-folder.md](../software-design/app/data-folder.md)); the run's counts go
to the console.

## Python analysis helpers

The helpers live in `tools/analysis` and need the project virtual environment ([setup.md](setup.md)):

```bash
source .venv/Scripts/activate
python -m pytest tools/analysis/tests
```

| Command | Does |
|---|---|
| `python -m packing prepare --labels … [--csv …] --work work` | The whole pick flow from the command line (an analysis aid, not the product) |
| `python tools/analysis/helpers/extract_labels.py "samples/*Shipping label*.pdf" -o work/labels.csv` | Label PDFs → CSV of `order_id,tracking_id`, one row per order |
| `python tools/analysis/helpers/diff_csv.py A.csv B.csv [--missing f.csv --new f.csv --common f.csv]` | Compare two CSVs by Order ID; prints the counts and IDs and writes a file only per option |
