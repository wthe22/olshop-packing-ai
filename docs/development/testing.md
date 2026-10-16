# Testing

| Level | What it covers | Command |
|---|---|---|
| Engine unit tests | Every reading and sorting rule on hand-made text runs | `cargo test --workspace` |
| Shared fixtures | `testdata/conditions.json` (parse, print, evaluate, error texts), `testdata/expected-labels.json`, `testdata/expected-picks.json`. Expected: the results written there (the Python tools give the same); cases that use CSV fields expect the "needs the orders CSV" error in Rust | `cargo test --workspace` |
| PDF tests | `testdata/labels-slip.pdf` (made up, from `make_testdata.py`): read, copy, packing list. Expected: page counts, text found in the output | `cargo test --workspace` |
| Check before saving | The arithmetic on hand-made descriptions (`crates/engine/tests/saved_pdf_check.rs`: a doubled page, a dropped page, an order in two picks and in none, a wrong label, a swapped order) and the read-back on real files (`crates/pdf/tests/verify.rs`: a saved batch reads back clean; a tampered written file is caught). Each defect test fails when the check is removed | `cargo test --workspace` |
| Front-end flow tests | The logo prompt's flow (a courier name, *Not a courier*, *I don't know*, *Skip all*, Esc) and the counts it shows, against a small fake of the back end; and the check before saving's window side (the findings dialog for errors and for warnings alone, the two stage texts of `save-progress`). No window opens and nothing is written | `cd app && npm test` |
| Analysis tools | `tools/analysis` on `testdata/` and, when present, `samples/` | `python -m pytest tools/analysis/tests` ([setup.md](setup.md)) |
| Local sample test (skipped without `samples/`) | The three *Shipping label + Packing slip* sample files. Expected: 601 pages, 600 orders, one order on 2 pages, J&T 517 pages; the picks, runs and page counts equal what the Python tools (`tools/analysis`) give on the same files and rules | `cargo test --workspace` |
| By eye | Saved PDFs and packing lists from a sample run | a sample run ([cli.md](cli.md)), then compare the pages with the download (rendered side by side); the layout is as in [packing-list.md](../software-design/labels/packing-list.md) |

All Rust levels run from the repository root; the Python tests need the virtual environment
([setup.md](setup.md)). Setup, run and build commands are in [app.md](app.md).
