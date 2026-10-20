# Code rules

Binding for all code in this repository, written by the owner, an agent or a subagent: Rust, Svelte and
TypeScript. (`tools/analysis/` is the frozen Python reference and is out of scope.) The project is in heavy
development: build for what is decided now, add no compatibility layer or option for a change that might
come. Stored user data is the exception: a change to its shape is a migration.

## The seven rules

1. Every module, class and component has one clear responsibility. No god class, no tangled or mixed logic.
2. No big piles of code in a file. A big file is broken into smaller parts or simplified.
3. Do not over-generalise: no complex abstraction for cases that do not exist. A value that differs per user
   is a setting, a fixed value is a constant.
4. Do not over-engineer: no abstraction without a user. Fixed things stay hardcoded and simple.
5. Minimise duplication without over-engineering: extract on the third copy.
6. Comment only what is not obvious at a glance. No comment repeats the code. No ticket, decision, question or
   review ID, no task number and no history in a comment (that belongs in the commit message).
7. Use proper types for editor hints and completion. No overloaded meaning: no value that means two things
   depending on its content (a string that is an ID or a temporary ID, an `Option` that also means "not
   edited", an empty string that means none, a double `Option`, string keys where an enum belongs).

## How the rules are applied

**Functions.** No line limit: a function is as long as its one job needs. It is too much when it does several
separately named jobs in sequence, nests deeper than about four levels, or needs banner comments. Split by
job, not by length. At most **five parameters**; more means a missing type that groups them. A lint
suppression for "too many arguments" is not allowed.

**File size.** A ceiling that starts a split, not a target. Rust: **800 production lines** (the lines before
the first `#[cfg(test)]`); a Rust test file: **800** lines; a Svelte component or TypeScript module: **500**
lines. A ratchet enforces it: `tools/check-size.mjs` lists the files over the ceiling and
fails only for a file that is not in `tools/baselines/size.json` or that has grown. A small file with two
jobs is still wrong (rule 1).

**Lint ratchet.** `clippy.toml` sets the parameter limit (5) and the cognitive-complexity warning (25). The
parameter limit is a plain deny (`-D clippy::too_many_arguments` in `tools/check.sh`), so a suppression never
hides it. `tools/check-lint.mjs` counts, per file, the cognitive-complexity warning and every lint suppression
(`#[allow`, `svelte-ignore`, `@ts-ignore`, `eslint-disable`) and fails for a file not in
`tools/baselines/lint.json` or one that has grown. A baseline only shrinks; a new suppression is never added
to it.

**Types.** One type per concept, shared by the layers; the TypeScript interfaces mirror the Rust view types
by hand in `app/src/lib/api/` and change in the same commit. IDs are newtypes (`OrderId`, `TrackingId`,
`BatchNumber`, `PdfNumber`, `Grouping`), not `String` or `i64`. A kind or flag with a fixed set of values is
an enum. A parse of an engine's error text to branch is not allowed; a typed error is.

**Naming.** One word per concept, taken from the glossary below. A stored name (a key in `state.json`,
`settings.json`, `categories.toml`) stays as written even when the glossary word differs; the code maps it
with `serde(rename)`.

| Concept | Word in code | Not | Note |
|---|---|---|---|
| Named collection of categories | `Grouping` | set, rules set, picks set | stored key `set` stays |
| One entry of a grouping | `Category` | pick, entry, rule | stored layout name `pick` stays |
| The output PDF of one category | `SavedPdf` | output, result file | numbered through the day: `PdfNumber` |
| Consecutive identical orders in a saved PDF | `Run` | group, block | |
| One round of download and save | `Batch`, `BatchNumber` | job, round | |
| A batch read but not yet saved | `Draft` | pending, session | |
| One working day's folder and `state.json` | `Day` | date folder | |
| One platform order, its 18-digit ID | `Order`, `OrderId` | | |
| The courier's parcel number | `TrackingId` | AWB, waybill, resi | |
| One product line of an order | `Line` | item, row | the SKU ID identifies an item |
| The A4 sheet of runs | `PackingList` | picking list | |
| The owner's data folder | `DataFolder` | workspace, root | |
| A time stamp | `*_time` | `*_at`, `*_date` | ISO 8601 with offset |

**Comments.** `//!`/`///` say what a module or item is for and anything surprising (a PDFium quirk, a unit,
an ordering that matters). A pointer to a design file is by file name, never by number or ID. No banner
comments (`// ------ section`): a section that needs one is a module.

**Duplication.** The third copy is extracted. Test fixtures (a line, an order, a temp folder) live once in a
shared test module, not once per file.

## Tests

- The main kind is a **scenario test at the public surface**: an API call or a command, or a rendered screen
  (act, read what shows). One scenario covers many branches.
- **Unit tests only for fixed truths**: a pure function with a checkable answer (column letter `A` → 1, `Z` →
  26), written as a table.
- Not tested: a private function, a single field's pass-through, wording or layout, one refusal per test when a
  table does the same.
- **Coverage is the measure**, used to find gaps and dead code, never to chase a number. `cargo llvm-cov`
  for Rust, `node --test --experimental-test-coverage` for TypeScript; the baseline per language is in
  `tools/baselines/coverage.json` and read in [coverage.md](coverage.md). Coverage after a step is not lower
  than before it.
- A later bug gets one high-level scenario that fails before the fix.
- Tests that need `samples/` or `vendor/pdfium.dll` skip themselves when absent ([testing.md](testing.md)).

## The checks

`bash tools/check.sh` runs, in this order: `cargo fmt --all --check`, `cargo clippy --workspace
--all-targets -- -D warnings`, `cargo test --workspace`, `node tools/check-size.mjs`, `node
tools/check-lint.mjs`, `node tools/check-comments.mjs` (zero: no ID, task number, numbered design pointer or banner in a comment), `cd app && npm run check`, `cd app && npm test`. `bash tools/check.sh --coverage` adds
`node tools/check-coverage.mjs`. All must be green before a commit.

## Process rules for a refactor

- Every task file and every subagent instruction names this file. Every task report states: **size gate
  passed, no new lint suppression, coverage not lower**.
- One refactor step is one or more commits, made when the step's automatic checks are green.
- No hand review of each step. One minimal review after the step whose types the later steps build on (diff
  summary, check results, baseline and suppression lists, a spot read of the new types), and one full review
  at the end (the rules against the whole diff, the baseline and suppression lists, coverage before and
  after).
- The existing tests stay green. They are edited only where types change, where a small change moves an
  assertion, and where they are merged at the end.
- A refactor does not change what the program does: the screens, their order, what each save, import,
  migration and refusal does, what is stored. A small change nobody would hardly notice (a message's wording
  unified, a default made consistent, a field order, an ID's form) is allowed, is named in its commit message
  and is collected for the handoff. A change to a mechanism or a workflow is its own commit later.
