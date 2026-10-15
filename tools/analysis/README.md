# tools/analysis — Python tools for looking at the packing files

Small Python programs for **inspecting and debugging** the shipping-label PDFs, packing slips and
orders CSVs, and for checking the PC app against an independent implementation. They are **not
part of the product**: the PC app (`crates/`, `app/`) does not use them, nothing here is shipped or run in
daily work, and the code is **frozen** — the PC app is the truth for behaviour, so when the two
differ, the app's design (`docs/software-design/`) decides.

Meant for coding agents (and the owner) who need to look inside a PDF or compare numbers.
`samples/` and `work/` hold real data: write output under `work/` or `tmp/` (git-ignored) and
never copy real IDs, names or addresses into the repository.

| Path | What it is |
|---|---|
| `helpers/extract_labels.py` | Label PDFs → CSV of `order_id,tracking_id`, one row per order |
| `helpers/diff_csv.py` | Compare two such CSVs by Order ID (counts and IDs; files only per option) |
| `helpers/courier_clues.py` | Per page: the courier evidence — each image's **encoded-bytes hash** and its **appearance hash** (mask applied, 64×64, thresholded), the tracking-ID candidates, the header-strip tag and the printed clauses (`FastTrack`, `TT Order ID`, …). The measurement behind `docs/software-design/labels/courier.md` and Q-LABEL-1 |
| `packing/` | The readers the PC app was ported from: `labels.py` (Order ID per page, label fields, courier), `slip.py` (packing-slip table by text position), `orders.py` (CSV), `rules.py` (`categories.toml` and conditions), `picks.py`, `packing_list.py`; `python -m packing prepare …` runs the whole pick flow from the command line |
| `tests/` | `pytest`, on `testdata/` (made up, committed) and `samples/` (local; skipped when absent) |
| `prototype/` | Throwaway A4 packing-list layout prototype, reference only |

## Use

From the repository root, Git Bash, with the project virtual environment (setup:
[`docs/development/setup.md`](../../docs/development/setup.md)):

```bash
source .venv/Scripts/activate
python -m pip install -e "tools/analysis[dev]"     # once; pypdf, fpdf2, pytest
python -m pytest tools/analysis/tests
python tools/analysis/helpers/extract_labels.py "samples/*Shipping label*.pdf" -o work/labels.csv
python tools/analysis/helpers/diff_csv.py A.csv B.csv
python tools/analysis/helpers/courier_clues.py "samples/*.pdf" --summary   # needs pillow, xxhash
python -m packing prepare --labels <label.pdf> [--csv <orders.csv>] --work work    # see --help
```

Other one-off helpers (`pypdfium2` to render a page to PNG, `xxhash` to compute the logo hash)
are installed into the same venv when needed and logged in the setup file; their scratch output
goes under `tmp/`.
