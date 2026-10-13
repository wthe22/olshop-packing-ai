# Development — desktop (PC app)

Setup and commands for the PC app and the Python analysis tools. Rules, session start and the repository layout are in
[`../../development/README.md`](../../development/README.md); the phases are in
[`../phases/`](../phases/) (1 = the Python script, done, now `tools/analysis`; 2 = PC app) and
[`../../development/plan.md`](../../development/plan.md).

| File | Content |
|---|---|
| [01-setup.md](01-setup.md) | Tools to install, one-time steps, and the log of what has been done on this PC |

Design to read: [`../design/`](../design/README.md); `categories.toml` format:
[`../../shared/04-rules-file.md`](../../shared/04-rules-file.md). The Python programs in
[`tools/analysis/`](../../../tools/analysis/README.md) are an analysis aid (frozen, not part of the
product).

## Commands (once set up)

| Task | Command (from the repository root, Git Bash) |
|---|---|
| Activate Python env | `source .venv/Scripts/activate` |
| Analysis tools: tests | `python -m pytest tools/analysis/tests` |
| Analysis tools: pick flow from the command line (not the product) | `python -m packing prepare --labels … [--csv …] --work work` |
| Label PDFs → CSV | `python tools/analysis/helpers/extract_labels.py "samples/*Shipping label*.pdf" -o work/labels.csv` |
| Diff two CSVs | `python tools/analysis/helpers/diff_csv.py A.csv B.csv [--missing f.csv --new f.csv --common f.csv]` (by Order ID; prints the counts and IDs, writes a file only per option) |
| PC app: Rust tests | `cd pc && cargo test` |
| Developer CLI (batch steps for tests/sample runs) | `cd pc && cargo run -p packing-cli -- prepare --labels <file>... [--rules <categories.toml>] [--data <dir>] [--day YYYY-MM-DD] [--packing-list whole\|per-pdf\|none] [--layout pick,full,summary]` — also `revert` / `amend`; `--quiet-ids` prints counts only. Writes `<data>/labels/<day>/batch <b>/` (07 › *Day folder*) |
| PC app: run in development | `cd pc/app && npm run tauri dev` |
| PC app: portable folder | `pc/tools/make-portable.sh` → `pc/target/portable/Packing/`; on Windows, `pc\tools\make-portable.bat` (double-click; no Git Bash window needed) |
| Use the PC app | [../user-guide/01-pc-app.md](../user-guide/01-pc-app.md) (owner guide: setup, a day's batches, print) |
