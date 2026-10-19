# R0 — Tools and baselines

Read first: [../code-rules.md](../code-rules.md) (binding), [refactor.md](refactor.md) (brief, choices).
Effort M. Edits: new files under `tools/`, `clippy.toml`, `docs/development/coverage.md`, and the docs named
below. No source file of the program changes in this step.

Build, all as Node scripts (`.mjs`, no new npm dependency) run from the repository root:

1. **`tools/check-size.mjs`** + `tools/baselines/size.json`. Measures: Rust production lines (before the
   first `#[cfg(test)]` line) with ceiling 800; a Rust file under a `tests/` folder: all lines, ceiling 800;
   `.svelte` and `.ts`/`.mjs` under `app/src` and `app/tests`: all lines, ceiling 500. Skips `target`,
   `node_modules`, `vendor`, `samples`, `work`, `tmp`, `spike`, `tools/analysis`, `app/build`, `.svelte-kit`,
   `docs`. The baseline lists each file over its ceiling with its line count. Fails for a file over the ceiling
   that is not in the baseline or whose count grew; prints the files that shrank or fell under the ceiling and
   tells to lower the baseline (`--update` rewrites it and may only lower or remove entries).
2. **`clippy.toml`**: `too-many-arguments-threshold = 5`, `cognitive-complexity-threshold = 25`.
   **`tools/check-lint.mjs`** + `tools/baselines/lint.json`: runs clippy over the workspace with
   `--message-format=json` and `-W clippy::too_many_arguments -W clippy::cognitive_complexity`, and scans
   sources for suppressions (`#[allow`, `#![allow`, `svelte-ignore`, `@ts-ignore`, `@ts-expect-error`,
   `eslint-disable`). Counts per file and per lint kind; same ratchet rule as the size script (a new file or a
   grown count fails; `--update` only lowers). The plain gate `cargo clippy --workspace --all-targets -- -D
   warnings` keeps its meaning: add `-A clippy::too_many_arguments -A clippy::cognitive_complexity` to it in
   `tools/check.sh` so the two ratcheted lints are counted only by the script.
3. **`tools/check-coverage.mjs`** + `tools/baselines/coverage.json`. Rust: `cargo llvm-cov --workspace
   --json --summary-only` (lines, regions, functions, in percent, per crate and total; `cargo-llvm-cov` and
   `llvm-tools-preview` are installed on this PC). TypeScript: `node --test --experimental-test-coverage`
   from `app/` (the line percent of the files under `app/src`). Compare mode (default) fails when any
   baseline number is lower than recorded by more than 0.1 point; `--record` writes the baseline (used in
   this step only; later steps never record).
4. **`tools/check.sh`**: the checks listed in code-rules.md, in that order, stopping at the first failure;
   `--coverage` adds the coverage compare. Run from any folder.
5. Record the **coverage BEFORE** with `--record`, then write `docs/development/coverage.md` (one table per
   language: total and per crate, the files at under 50 % lines, the files at 0 %, what no test reaches
   (the Svelte screens and `+page.svelte` have no rendered test) and the date and commit it was measured at).
6. Docs: [../setup.md](../setup.md) gets the two coverage tool installs in its tool table and steps;
   [../app.md](../app.md) Gates table gets `bash tools/check.sh [--coverage]` and the two ratchet scripts;
   [../testing.md](../testing.md) gets one line pointing at coverage.md. Keep each file within about 150 lines.

Done when: `bash tools/check.sh --coverage` is green on the unchanged program, both baselines list today's
offenders exactly (the three `#[allow(clippy::too_many_arguments)]`, the four Rust files and three UI files over
their ceilings, the test files over 800), and coverage.md holds the BEFORE numbers. Test the scripts by
making a temporary offender (a 900-line file, a new `#[allow]`) and seeing the gate fail, then remove it.
