# R7 — Merged tests, then the full review

Read first: [../code-rules.md](../code-rules.md) (binding; the *Tests* section is this step), [refactor.md](refactor.md)
(brief, choices). Effort L. Edits only test code and the files that hold it. Part 1 (testkit) is done.

1. (done) `crates/testkit` holds the shared fixtures and the PDFium lock; use it in every test you touch.
2. **Merge tiny tests** into scenarios at the public surface: one scenario per behaviour, many branches
   inside it. Candidates: `engine/tests/day_state.rs` (28 tests, 938 lines; over the 800 ceiling),
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
