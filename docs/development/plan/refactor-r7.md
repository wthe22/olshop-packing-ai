# R7 — Shared test fixtures, merged tests, then the full review

Read first: [../code-rules.md](../code-rules.md) (binding; the *Tests* section is this step), [refactor.md](refactor.md)
(brief, choices). Effort L. Edits only test code and the files that hold it.

1. **`crates/testkit`** (dev-only, `publish = false`, a `[dev-dependencies]` entry of the crates that use it):
   the fixtures declared once per file today: building a `Line`, an `Order`, a `ReadOrder` and a `Category`
   (about ten copies in `engine/tests/*`, `pdf/src/batch`, `commands/*` tests), `temp_dir` (six copies),
   `vendor_dir` and `testdata` (four copies, `pdf/tests/common`). Only what three or more files use; no
   builder framework, plain functions. Add `crates/testkit` to the ignore list of `tools/check-coverage.mjs` (fixtures are not
   product code).
   The app crate's tests start several `PdfWorker`s in parallel and crash about once in twenty runs
   (access violation or illegal instruction inside PDFium); `crates/pdf/tests/common` already serialises
   PDFium use with a lock. Put that lock in `testkit` and use it in the app tests: a flaky gate is a defect.
   Add a scenario test of `app/src/lib/shell.svelte.ts` with a faked `invoke` (0.79 % covered, the largest
   gap in the front-end), and of `categoryEntry.ts`.
2. **Merge tiny tests** into scenarios at the public surface: one scenario per behaviour, many branches
   inside it. Candidates: `engine/tests/day_state.rs` (28 tests, 982 lines; over the 800 ceiling),
   `labels.rs` (20), `labels_reading.rs` (16), `plan_edits.rs` (17), `names.rs` (10), the `commands/` tests,
   `pdf/src/batch` tests. A pure function with a checkable answer becomes a table-driven test. Delete a test
   of a private function, of a single field's pass-through, of wording or layout, and one-refusal-per-test
   when a table does the same, **only** when `bash tools/check.sh --coverage` shows coverage not lower.
3. The front-end tests (`app/tests/*.test.mjs`, 53) get the same treatment where they test one field.
4. Fixtures under `testdata/` are not changed. A test that skips without `samples/` or `pdfium.dll` still
   skips, and the skip is reported once per file, not per test.
5. Lower the size baseline (`day_state.rs` under 800). Report: tests before and after (Rust, TypeScript),
   coverage before and after per language.

## The full review (the parent, after the step is committed)

Rules against the whole diff (`git diff <before-R0>..HEAD --stat` and a read of every new module's header
and public types): one job per module, no file over its ceiling, parameter count, no suppression, comments,
glossary words. The size and lint baselines: each entry left needs a reason or goes into the handoff as a
blocker. Coverage before ([../coverage.md](../coverage.md)) and after, per language. Complexity rated again
1 to 5. Findings are fixed in a final commit when small; otherwise listed in the handoff.
