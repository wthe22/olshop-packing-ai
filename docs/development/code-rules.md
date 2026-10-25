# Code rules

Binding for all code in this repository, written by the owner, an agent or a subagent: Rust, Svelte and
TypeScript. (`tools/analysis/` is the frozen Python reference and is out of scope.) The project is in heavy
development: build for what is decided now, add no compatibility layer or option for a change that might
come. Stored user data is the exception: a change to its shape is a migration.

Every rule states its reason. An exception to a rule is allowed only with a one-line reason in the commit
message that makes it.

## The seven rules

1. Every module, class and component has one clear responsibility. No god class, no tangled or mixed logic.
   *Why: a reader can check one job at a time, and a change touches one place.*
2. No big piles of code in a file. A big file is broken into smaller parts or simplified.
   *Why: a file that fits in one reading can be reviewed whole.*
3. Do not over-generalise: no complex abstraction for cases that do not exist. A value that differs per user
   is a setting, a fixed value is a constant. *Why: every abstraction is code to read and test, and a case
   that does not exist cannot tell what the abstraction should be.*
4. Do not over-engineer: no abstraction without a user. Fixed things stay hardcoded and simple.
   *Why: the same — and the simplest code is the easiest to prove right.*
5. Minimise duplication without over-engineering: extract on the third copy.
   *Why: two copies are cheaper than a wrong abstraction; three show what the shared part is.*
6. Comment only what is not obvious at a glance. No comment repeats the code. No ticket, decision, question or
   review ID, no task number and no history in a comment (that belongs in the commit message).
   *Why: a comment that repeats the code goes stale; history lives in git.*
7. Use proper types for editor hints and completion. No overloaded meaning: no value that means two things
   depending on its content (a string that is an ID or a temporary ID, an `Option` that also means "not
   edited", an empty string that means none, a double `Option`, string keys where an enum belongs).
   *Why: the compiler then checks what a reader would otherwise have to remember.*

## How the rules are applied

**Functions.** No line limit: a function is as long as its one job needs. It is too much when it does several
separately named jobs in sequence, nests deeper than about four levels, or needs banner comments. Split by
job, not by length. At most **five parameters**; more means a missing type that groups them. A lint
suppression for "too many arguments" is not allowed.

**File size.** A ceiling that starts a split, not a target. Rust: **800 production lines** (the lines before
the first `#[cfg(test)]`); a Rust test file: **800** lines; a Svelte component or TypeScript module: **500**
lines. `tools/check-size.mjs` fails for a file over the ceiling that is not in `tools/baselines/size.json` or
that has grown. A small file with two jobs is still wrong (rule 1).

**Lint ratchet.** `clippy.toml` sets the parameter limit (5) and the cognitive-complexity warning (25). The
parameter limit is a plain deny (`-D clippy::too_many_arguments` in `tools/check.sh`), so a suppression never
hides it. `tools/check-lint.mjs` counts, per file, the cognitive-complexity warning and every lint suppression
(`#[allow`, `svelte-ignore`, `@ts-ignore`, `eslint-disable`) and fails for a file not in
`tools/baselines/lint.json` or one that has grown. A baseline only shrinks; a new suppression is never added.

**Types.** One type per concept, shared by the layers; the TypeScript interfaces mirror the Rust view types by
hand in `app/src/lib/api/` and change in the same commit. IDs are newtypes, not `String` or an integer. A kind
or flag with a fixed set of values is an enum. A parse of an error's text to branch is not allowed; a typed
error is. `packing-verify` never imports a type of the crates it checks
([architecture](../software-design/app/architecture.md#who-may-use-whom)).

**Naming.** One word per concept, from the glossary below. A stored name (a field of the journal or of a
TOML file) is the design's word; the code maps it with `serde(rename)` where they differ. Code that still
uses a v0.1 word takes the new one when it is rewritten.

| Concept | Word in code | Not |
|---|---|---|
| One shop owner's folder | `Namespace` | session, shop |
| One date of a namespace, its journal folded | `Day` | session, date folder |
| The one truth of a day | `Journal`, one `Record` per line | log, state |
| A file the app writes from the journal | `View` | cache, export |
| A named list of label groups | `SortRule` | grouping, set |
| One entry of a sort rule | `LabelGroupRule`; its result in a batch `LabelGroup` | category, pick |
| The output PDF of one label group | `GroupPdf` | saved PDF, output |
| The one A4 file of a batch | `SummaryPdf` | packing list, picking list |
| Consecutive orders with one packing key | `Run` | block |
| One round of download and save | `Batch`, `BatchId` | job, round |
| A label download, stored by content | `InputPdf`, `PdfId` | file, download |
| A batch read but not yet saved | `Draft` | pending |
| One platform order | `Order`, `OrderId` | |
| The courier's parcel number | `TrackingId` | AWB, waybill, resi |
| One product line of an order | `Line` | item, row |
| The owner's data folder | `DataFolder` | workspace, root |
| A time stamp | `*_time` | `*_at` |

**Comments.** `//!`/`///` say what a module or item is for and anything surprising (a PDFium quirk, a unit,
an ordering that matters). A pointer to a design file is by file name, never by number or ID. No banner
comments (`// ------ section`): a section that needs one is a module.

**Duplication.** The third copy is extracted. Test fixtures (a line, an order, a temp folder) live once in a
shared test module, not once per file.

## Tests

*Why: a test is worth its cost when it pins an answer someone checked, at a surface that does not move when
the code inside is rearranged.*

- **The backbone: known-answer fixtures** through read → plan → save → verify. A fixture PDF and its expected
  answer (orders, groups, runs, the record) are written together by the fixture writer, as the real downloads
  are made (one character per text object). One fixture covers many branches.
- **Tables for fixed truths**: a pure function with a checkable answer (a regex, a sort order, a file name),
  written as one table, not one test per row.
- **Broken outputs the verifier must catch**: a few saved sets with a doubled, a missing, a swapped and a
  foreign page; each test fails when its check is removed.
- **An end-to-end test** (the window, the whole app) only where it proves what nothing smaller can.
- Not tested: a private function, a single field's pass-through, wording or layout.
- **Coverage is the measure**, used to find gaps and dead code, never to chase a number. `cargo llvm-cov` for
  Rust, `node --test --experimental-test-coverage` for TypeScript; the baseline per language is in
  `tools/baselines/coverage.json` and read in [coverage.md](coverage.md). Coverage after a step is not lower
  than before it.
- A later bug gets one high-level scenario that fails before the fix.
- Tests that need `samples/` or `vendor/pdfium.dll` skip themselves when absent ([testing.md](testing.md)).

## The checks

- **While working**, the quick check: `cargo test -p <crate>` for the crate being changed, `cd app && npm test`
  for the screens.
- **Before a commit**, the full check: `bash tools/check.sh --coverage` runs, in this order,
  `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`
  (the slow and `samples/` tests included), `node tools/check-size.mjs`, `node tools/check-lint.mjs`,
  `node tools/check-comments.mjs` (zero: no ID, task number, numbered design pointer or banner in a comment),
  `cd app && npm run check`, `cd app && npm test`, `node tools/check-coverage.mjs`. All must be green.

## Process rules for a refactor

- Every task file and every subagent instruction names this file. Every task report states: **size gate
  passed, no new lint suppression, coverage not lower**.
- One refactor step is one or more commits, made when the step's checks are green.
- No hand review of each step. One minimal review after the step whose types the later steps build on, and one
  full review at the end (the rules against the whole diff, the baseline and suppression lists, coverage).
- The existing tests stay green. They are edited only where types change, where a small change moves an
  assertion, and where they are merged at the end.
- A refactor does not change what the program does. A small change nobody would notice (a message's wording
  unified, a field order) is allowed, is named in its commit message and is collected for the handoff. A
  change to a mechanism or a workflow is its own commit.

## Choices taken in the first refactor

- `tools/analysis/` (the frozen Python reference) and `spike/` are out of scope of the gates.
- The gate scripts are Node (`tools/*.mjs`): Node is already needed for the app.
- Rust and TypeScript view types stay hand-mirrored (no generator); they change in the same commit.
- No DOM test harness for the Svelte screens: the guards are `svelte-check`, the front-end scenario tests (the
  shell store with a faked `invoke`) and the owner's regression list.
- Tauri commands keep `Result<_, String>`: the bridge to the window is the one place an error becomes text.
- The coverage compare tolerates 0.1 percentage point (rounding), nothing more ([coverage.md](coverage.md)).
