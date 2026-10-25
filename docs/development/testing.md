# Testing

| Level | What it covers | Command |
|---|---|---|
| Engine unit tests | Every reading and sorting rule on hand-made text runs | `cargo test --workspace` |
| Search | `crates/engine/tests/search.rs`: a tracking ID finds its one order, an item field matches when one line matches, `not` and *at least* behave as in a category, a day saved before search shows *items not stored* and is found by its IDs only, the fields survive a `state.json` round trip; the app reads the day folders for *This day* / *All days* and never sees an open draft's order | `cargo test --workspace` |
| Shared fixtures | `testdata/conditions.json` (parse, print, evaluate, error texts), `testdata/expected-labels.json`, `testdata/expected-picks.json`. Expected: the results written there; cases that use CSV fields expect the "needs the orders CSV" error | `cargo test --workspace` |
| Fixture writer | `crates/fixtures` writes `testdata/labels-slip.pdf`, `labels-plain.pdf` and their answer `expected-labels.json` from one set of made-up orders ([cli.md](cli.md#packing-fixtures)). Expected: writing them again gives the committed bytes | `cargo test --workspace` |
| PDF tests | `testdata/labels-slip.pdf` and `labels-plain.pdf`: read, copy, packing list. Their pages are drawn one character per text object, as the downloads are; the last page of `labels-plain.pdf` in whole strings, the path of the Shopee-style label. Expected: `expected-labels.json`, page counts, text found in the output | `cargo test --workspace` |
| Check before saving | The arithmetic on hand-made descriptions (`crates/engine/tests/saved_pdf_check.rs`: a doubled page, a dropped page, an order in two picks and in none, a wrong label, a swapped order) and the read-back on real files (`crates/pdf/tests/verify.rs`: a saved batch reads back clean; a tampered written file is caught). Each defect test fails when the check is removed | `cargo test --workspace` |
| Front-end flow tests | The logo prompt's flow (a courier name, *Not a courier*, *I don't know*, *Skip all*, Esc) and the counts it shows, against a small fake of the back end; the check before saving's window side (the findings dialog for errors and for warnings alone, the two stage texts of `save-progress`); and the *No picks yet* line's rule (the one catch-all entry, Q-UI-4). No window opens and nothing is written | `cd app && npm test` |
| Local baseline (skipped without `samples/`) | `crates/cli/tests/samples_baseline.rs`: `packing-cli dump` of each `samples/*.pdf` equals `work/baseline/<file>.json`, both confidential and never committed. A missing baseline file is written from the code of the day; a change meant to alter the reading deletes the files it alters, runs the test again and states the difference in its commit message | `cargo test --workspace` |
| By eye | Saved PDFs and packing lists from a sample run | a sample run ([cli.md](cli.md)), then compare the pages with the download (rendered side by side); the layouts are those of `crates/pdf/src/packing_list.rs` (the target is the [summary PDF](../software-design/labels/summary-pdf.md)) |

All Rust levels run from the repository root. Setup, run and build commands are in [app.md](app.md);
what each level covers in percent, and where no test reaches, is in [coverage.md](coverage.md).

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
