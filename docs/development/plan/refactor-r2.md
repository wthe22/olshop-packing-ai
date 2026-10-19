# R2 — The typed boundary (rule 7), the glossary, parameter groups

Read first: [../code-rules.md](../code-rules.md) (binding; its glossary table is the naming source),
[refactor.md](refactor.md) (brief, choices). Effort L. Every later step builds on these types. Stored files
keep their shape: every newtype serialises transparently and a renamed Rust field keeps its stored name with
`#[serde(rename)]`; a test round-trips a `state.json` and a `settings.json` written before this step.
Tests are edited only where a type or a signature changes.

1. **Newtypes** (Rust, then the TypeScript mirrors): `Grouping` (replaces `set: String`/`&str` in about 40
   signatures: `engine/src/sets.rs`, `day.rs`, `commands/*`, `pdf/batch`, the CLI; `DEFAULT_SET` becomes
   `Grouping::default()`), `OrderId`, `TrackingId`, `BatchNumber`, `PdfNumber` (the day's numbering; today a
   bare `i64` beside a batch `i64`, e.g. `set_printed(day, batch, pdf, printed)`). The IPC argument and field
   `set` becomes `grouping` on both sides (not stored).
2. **One enum per concept**: the layout and scope enums exist in `engine/src/day.rs` and
   `pdf/src/packing_list.rs` with converters (`layout_name`, `to_pdf_layout`, `parse_layout`, `parse_scope`):
   keep one, parse text only where the command line needs it. `SaveProgressEvent.stage: String` becomes the
   `SaveStage` enum. The twin `LogoAnswerInput`/`LogoAnswer` becomes one type if their shapes are equal.
3. **Plan row**: `PlanRow` uses `index: Option<usize>` (None = generated rest row) plus the flags
   `generated`, `is_rest`, `empty`, `can_*`. Replace by a kind enum (a plan entry with its index, the rest
   row, the generated `Uncategorised` row) and only the flags that are not derivable. Update `api.ts` and the
   Plan screen minimally; the screen looks and acts the same.
4. **Audit rule 7** over `crates/`, `app/src-tauri/src/` and `app/src/`: an empty string or `0`/`-1` that
   means none, an `Option` that means "not edited", a value that is an ID or something else, string keys where
   an enum belongs (including `kind` strings in `api.ts`), `Option<String>` fields (72 `Option<String>` or
   `String::new()` uses). Fix what is overloaded; leave a plain optional value that is just optional.
5. **Parameter groups**: no function with more than five parameters, no `#[allow(clippy::too_many_arguments)]`.
   Known: `pdf::batch::save` (10), `cli::save_and_report` (9), `packing_list::cell_right` (8) and `cell` (7),
   `commands::read_into_draft` (7), `cli::report` (6), `pdf::batch::read` (6), `packing_jobs` (6); also
   whatever `tools/check-lint.mjs` lists. Group them in a named type (`SaveRequest`, `Cell`, …), one type
   per concept. The `too_many_arguments` count in `tools/baselines/lint.json` goes to zero; then add
   `-D clippy::too_many_arguments` to the plain clippy command in `tools/check.sh` and drop it from the
   ratchet script.
6. **Glossary rename** in identifiers (comments follow in R6): `set` → `grouping`; `pick` → `category`
   (`PickResult` → `CategoryResult`, `pick_*` functions, local names); `*_at` → `*_time` (`set_printed_at`).
   Stored names stay (`set` key, the `pick` layout) with `serde(rename)`; do not rename a file or folder the
   owner sees. The user-facing word is already *Grouping* and *Category*.
7. A typed error where code branches on an engine's message text, if any exists in production code.

Done when: `bash tools/check.sh --coverage` is green, the `too_many_arguments` and suppression counts in the
lint baseline for Rust are zero, and the report lists each new type with the file it lives in.
