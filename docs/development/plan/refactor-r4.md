# R4 — The data and storage layer: split by job, remove duplicated helpers

Read first: [../code-rules.md](../code-rules.md) (binding), [refactor.md](refactor.md) (brief, choices).
Effort M. Edits the engine crate, `app/src-tauri/src/commands/{day,rules,search}` and the storage helpers of
`crates/pdf`. Behaviour and stored shapes do not change; moves are pure, tests are not edited except imports
and types.

1. `crates/engine/src/day.rs` (595) → `day/`: the `state.json` model and its atomic read/write, the duplicate
   guard and file fingerprints, the batch save and the batch folder delete, `DayError`. One job per file.
2. `crates/engine/src/orders.rs` (597) → `orders/`: the order and line model, the reading warnings and
   errors, the unseen-logo types and answers, `read_batch`. `label.rs` (659) → the courier table (loading,
   names) apart from the deduction (tracking-ID form, logo lookup). `rules_edit.rs` (642): the editable
   document apart from the live counts if it holds both. `verify.rs`, `sets.rs`, `couriers_edit.rs`, `plan.rs`
   and `search.rs`: read each; split only a file with two jobs.
3. `app/src-tauri/src/commands/day.rs`: the settings file (load, save, remember the last grouping) is not the
   day overview and the printed tick; give it its own module. `commands/rules.rs` and `commands/search.rs`:
   one job per file, the `*_impl` functions that only wrap another call removed when the command can call the
   engine directly (keep an `_impl` only where a test needs it without a Tauri `State`).
4. **Duplicates, extracted on the third copy**: `file_name` (`engine/orders.rs`, `pdf/batch`, `pdf/read.rs`),
   the order index by ID (`pdf/batch` and `commands/batch`) and the split of orders per run (`commands/batch`
   and `engine/plan.rs`), `modified_time`, any write-to-a-temporary-file-then-replace helper written more than
   twice (`day.rs` `replace`, the `toml_edit` writers), the PDFium page lookup repeated in `pdf/read.rs`.
   Two copies stay two; say so in the report.
5. Remove dead code found on the way (a function or parameter nothing calls; `cargo llvm-cov` at 0 % on a
   production function is the lead). List each removal in the report.
6. Update `docs/software-design/` only if a module name written there changed (a document names a file or
   module); keep the facts.

Done when: `bash tools/check.sh --coverage` is green, no file in the engine crate or `commands/` is over its
ceiling, and the report lists the duplicates extracted and the ones left at two copies.
