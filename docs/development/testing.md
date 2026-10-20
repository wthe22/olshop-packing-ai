# Testing

| Level | What it covers | Command |
|---|---|---|
| Engine unit tests | Every reading and sorting rule on hand-made text runs | `cargo test --workspace` |
| Search | `crates/engine/tests/search.rs`: a tracking ID finds its one order, an item field matches when one line matches, `not` and *at least* behave as in a category, a day saved before search shows *items not stored* and is found by its IDs only, the fields survive a `state.json` round trip; the app reads the day folders for *This day* / *All days* and never sees an open draft's order | `cargo test --workspace` |
| Shared fixtures | `testdata/conditions.json` (parse, print, evaluate, error texts), `testdata/expected-labels.json`, `testdata/expected-picks.json`. Expected: the results written there (the Python tools give the same); cases that use CSV fields expect the "needs the orders CSV" error in Rust | `cargo test --workspace` |
| PDF tests | `testdata/labels-slip.pdf` (made up, from `make_testdata.py`): read, copy, packing list. Expected: page counts, text found in the output | `cargo test --workspace` |
| Check before saving | The arithmetic on hand-made descriptions (`crates/engine/tests/saved_pdf_check.rs`: a doubled page, a dropped page, an order in two picks and in none, a wrong label, a swapped order) and the read-back on real files (`crates/pdf/tests/verify.rs`: a saved batch reads back clean; a tampered written file is caught). Each defect test fails when the check is removed | `cargo test --workspace` |
| Front-end flow tests | The logo prompt's flow (a courier name, *Not a courier*, *I don't know*, *Skip all*, Esc) and the counts it shows, against a small fake of the back end; the check before saving's window side (the findings dialog for errors and for warnings alone, the two stage texts of `save-progress`); and the *No picks yet* line's rule (the one catch-all entry, Q-UI-4). No window opens and nothing is written | `cd app && npm test` |
| Analysis tools | `tools/analysis` on `testdata/` and, when present, `samples/` | `python -m pytest tools/analysis/tests` ([setup.md](setup.md)) |
| Local sample test (skipped without `samples/`) | The three *Shipping label + Packing slip* sample files. Expected: 601 pages, 600 orders, one order on 2 pages, J&T 517 pages; the picks, runs and page counts equal what the Python tools (`tools/analysis`) give on the same files and rules | `cargo test --workspace` |
| By eye | Saved PDFs and packing lists from a sample run | a sample run ([cli.md](cli.md)), then compare the pages with the download (rendered side by side); the layout is as in [packing-list.md](../software-design/labels/packing-list.md) |

All Rust levels run from the repository root; the Python tests need the virtual environment
([setup.md](setup.md)). Setup, run and build commands are in [app.md](app.md); what each level covers
in percent, and where no test reaches, is in [coverage.md](coverage.md).

## Shared fixtures

`crates/testkit` holds the fixtures three or more test files declare today — a product line, an
order, a read order and a category, a fresh temp folder, and the `testdata/` and `vendor/` locators.
It is a `[dev-dependencies]` entry of the crates whose tests use it and is never built into the app;
its own coverage is left out of the report, since a fixture is not product code.

PDFium is not thread-safe, and the test harness runs the tests of one binary on several threads.
`testkit::pdfium_lock()` returns a guard that a test holds for its whole run whenever it binds
PDFium or starts a `PdfWorker` (the app's `app_state` helper hands the state and the guard back
together, the PDF tests wrap it in their `Bound`), so no two binds run beside each other.

The front-end tests load an `app/src` module through `app/tests/tsModule.mjs`: a resolver hook
completes the extensionless imports the bundler allows, and a fake replaces the Tauri bridge, so a
screen's own state and commands run with no window.
