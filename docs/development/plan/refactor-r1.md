# R1 — Split the files over the ceiling by job (pure moves)

Read first: [../code-rules.md](../code-rules.md) (binding), [refactor.md](refactor.md) (brief, choices).
Effort L. **Pure moves**: no signature, body, name or behaviour changes; visibility only widens to
`pub(super)`/`pub(crate)` where a move needs it. Public paths used elsewhere (`packing_engine::rules::…`,
`packing_pdf::batch::…`, the Tauri command paths in `app/src-tauri/src/lib.rs`) keep working because the
folder's `mod.rs` re-exports them. Tests are not edited, except `use` lines that the move makes necessary.
Run `bash tools/check.sh` after each file; lower the baselines with the scripts' `--update`.

One module per job, each under about 400 lines; the job names below are the target, adjust to what the code
shows.

1. `app/src-tauri/src/commands/batch.rs` (1147 production + 1349 test lines) → `commands/batch/`: `views.rs`
   (the serialisable view and input types), `plan_view.rs` (plan → view, plan edits), `read.rs` (check files,
   read with progress, cancel), `logo.rs` (logo views, pictures, base64, the answer and its remembering),
   `amend.rs` (amend start and revert), `save.rs` (save and its progress), `mod.rs` (the `#[tauri::command]`
   functions) and the tests in `tests/` (one file per scenario group, or `tests.rs`).
2. `crates/engine/src/rules.rs` (1110) → `rules/`: `error.rs`, `condition.rs` (the condition types and
   operators), `tokenizer.rs`, `parser.rs` (with the date and clock forms), `printer.rs`, `evaluate.rs`,
   `file.rs` (loading `categories.toml`, category headers, catch-all, `categorize`), `mod.rs`.
3. `crates/pdf/src/packing_list.rs` (1027) → `packing_list/`: `model.rs` (the data model: layout, lines,
   runs, sections), `fonts.rs` (font loading and segments), `measure.rs`, `builder.rs` (drawing ops and the
   three layouts, split again if over the ceiling), `mod.rs` (`write`, `title`).
4. `crates/pdf/src/batch.rs` (834 + 358 test lines) → `batch/`: `error.rs`, `check.rs` (check files, check
   saved), `read.rs` (`Draft`, `read`), `save.rs`, `jobs.rs` (the mapping into packing jobs and sections),
   `summary.rs`, `mod.rs`, tests in `tests.rs`.
5. Inline test blocks of more than 150 lines move to a sibling `tests.rs` (`#[cfg(test)] mod tests;`):
   `commands/day.rs` (424), `commands/rules.rs` (515), `commands/search.rs` (310).
6. Any other production file left over its ceiling.

Done when: no Rust production file is over 800 lines except those the size baseline still lists as a
blocker with a reason, `cargo test --workspace` runs the same number of tests as before (state the count before and after in the report), and
`bash tools/check.sh --coverage` is green.
