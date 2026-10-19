# Refactor to the code rules

Brings the code to [../code-rules.md](../code-rules.md) without changing what the program does. Run as
steps R3–R7 (R0 to R2 are done; the minimal review after R2 found nothing to fix), in order, each delegated to a subagent and checked by the parent. Every step file below names the
code rules; every report states **size gate passed, no new lint suppression, coverage not lower**.

| Step | Scope | Effort | File |
|---|---|---|---|
| R3 | The UI: split the big screens and `api.ts` | L | [refactor-r3.md](refactor-r3.md) |
| R4 | The data and storage layer: split by job, remove duplicated helpers | M | [refactor-r4.md](refactor-r4.md) |
| R5 | Split the functions that do several jobs or nest deeper than four levels | M | [refactor-r5.md](refactor-r5.md) |
| R6 | Comments: no IDs, no stale pointers, no banners, nothing that repeats the code | M | [refactor-r6.md](refactor-r6.md) |
| R7 | Shared test fixtures, tiny tests merged into scenarios and tables; then the **full review** | L | [refactor-r7.md](refactor-r7.md) |

The work starts at commit `91de98a` (the base of the final review's diff). A step edits only the files its file names; the steps run one after another, so no two edit the same file.

## Where the code stands before R0

Complexity 3 of 5. The engine crate is pure and has typed errors; the lint and test gates are green (270 Rust
tests, 53 front-end tests). What hurts: four files over the ceiling (`commands/batch.rs` 1147 production
lines, `rules.rs` 1110, `packing_list.rs` 1027, `pdf/batch.rs` 834) and three UI files over 500; a
grouping passed as a bare `set: String` through about 40 signatures; ID strings and bare `i64` numbers mixed
in one signature (`set_printed(day, batch, pdf, …)`); the layout enum written twice with converters; one
plan row type whose `index: None` means "generated row"; 8 functions with more than five parameters and
3 `#[allow(clippy::too_many_arguments)]`; 8 functions over 100 lines (`worker::serve` nests 7 deep);
92 comments citing decision, question or review IDs, 702 comments pointing at design files by a number
that no longer exists, 64 banner comments; the same test fixtures declared in about ten files. Kept as is:
the pure engine, the typed error enums, the tests on the real PDFs, the atomic write of `state.json`.

## Choices taken (defaults; the owner may overturn each)

- Ceilings: 800 production lines for Rust, 500 for a Svelte or TypeScript file, 800 for a Rust test file.
- `tools/analysis/` (the frozen Python reference) and `spike/` are out of scope.
- The gate scripts are Node (`tools/*.mjs`): Node is already needed for the app; no new dependency.
- Rust and TypeScript view types stay hand-mirrored (no generator); they change in the same commit.
- No DOM test harness for the Svelte screens in this refactor. The safety net for R3 is `svelte-check`, the
  existing front-end tests and the regression list in the handoff.
- Shared test fixtures become one dev-only crate `crates/testkit` (R7).
- Newtypes serialise transparently, so every stored file keeps its shape and no migration is needed.
- Tauri commands keep `Result<_, String>`: the IPC edge is the one place an error becomes text.
- A comment points at a design file by name, never by number or ID.
- Coverage compare tolerates 0.1 percentage point (rounding), nothing more.
- This run does not push (AGENTS.md says push; the run's instruction wins) and builds the portable folder
  once, at the end.

## How a session runs a step

1. `git status` and `git log --oneline -5`: another session may be editing; never touch its files.
2. Delegate the step to one subagent with the brief below.
3. Run `bash tools/check.sh --coverage` yourself. Do not review the step by hand. If a check fails, fix it or
   revert the step; never weaken a check or add to a baseline.
4. Commit when green (`Rn: <what>`; the message lists every small change a user could notice, under
   a line `Noticeable:`; the handoff collects them with `git log --grep Noticeable`), and delete the step's
   file and row.
5. After R2 do the minimal review; after R7 the full one; then the handoff in [README.md](README.md) and
   `bash tools/make-portable.sh`.

A step the rules make impossible is a blocker: write it under *Blockers* below and do not work around it.

### Brief for every subagent

> Repository: `D:\workspace\software\olshop-packing-ai` (Git Bash on Windows; read `AGENTS.md` first;
> `samples/`, `work/`, `tmp/` are confidential: never read, print or commit them). You do ONE refactor step.
> Read `docs/development/code-rules.md` (binding: the seven rules apply to code you write too) and your step
> file `docs/development/plan/refactor-rN.md` in full. Behaviour must not change: same screens, same
> commands, same stored data; existing tests stay green and are edited only as your step file allows. Do
> not ask questions: take the sensible default and state it in your report. Do not commit, push or touch files
> outside your step's list. Run `bash tools/check.sh` until green (from R1 on). Report: what changed (files),
> **size gate passed, no new lint suppression, coverage not lower** (with numbers), every small change a user
> could notice, anything you could not do and why.

## Blockers

None yet.
