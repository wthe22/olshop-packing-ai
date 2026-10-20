# Coverage

Coverage per language, measured by `tools/check-coverage.mjs` and held in
`tools/baselines/coverage.json`. The gate compares the Rust totals and per-crate figures and the
TypeScript totals, and fails when one is lower than the baseline by more than 0.1 point; a baseline
entry only shrinks. Run it with `bash tools/check.sh --coverage`.

The two measurements are `cargo llvm-cov --workspace --json --summary-only` (Rust; the instrumented
build goes to the folder in `CARGO_LLVM_COV_TARGET_DIR`, see [setup.md](setup.md)) and `node --test
--experimental-test-coverage` from `app/` (TypeScript). The numbers below are the state at commit
`81fb900`, measured 2026-10-19, after the first refactor step (the code split into modules) and before the
rest of [plan/refactor.md](plan/refactor.md).

`cargo llvm-cov` leaves out a file named `tests.rs` or under a `tests/` folder. The first baseline, taken
when the big files still held their tests inline, counted that test code as covered: Rust total 86.29 lines,
86.56 regions, 79.56 functions (`app/src-tauri` 86.38 / 88.11 / 70.65, `crates/pdf` 87.97 / 86.99 / 82.10).
The split moved the tests into `tests.rs` files and changed no production line (the engine crate, which had
no inline tests, is unchanged to the digit), so the table below is the production code only and is the
baseline every later step is compared with.

## Rust

| Scope | Lines % | Regions % | Functions % |
|---|---|---|---|
| Total | 82.70 | 82.14 | 75.79 |
| `app/src-tauri` | 71.14 | 70.50 | 51.65 |
| `crates/cli` | 25.44 | 23.17 | 22.73 |
| `crates/engine` | 91.10 | 91.46 | 92.64 |
| `crates/pdf` | 86.41 | 85.52 | 80.84 |

Under 50 % of lines: `app/src-tauri/src/main.rs` (0 %, the window's six-line entry point) and
`crates/cli/src/main.rs` (25.44 %, the developer command line; only the batch steps the tests drive
run). No other file is under 50 %; the engine's library modules are between 92 % and 100 %. A Rust
test file is not part of the measurement.

## TypeScript

Only the files a test loads are instrumented, so a `.ts` file no test loads counts as 0 % (the run
adds `--test-coverage-include-all` for that).

| Scope | Lines % | Branches % | Functions % |
|---|---|---|---|
| Total | 54.72 | 98.88 | 66.67 |

The first baseline (72.43 lines) was taken while the screens' logic sat inside `.svelte` files, which no
run measures. Splitting the shell out of `+page.svelte` (`app/src/lib/shell.svelte.ts`, 381 lines) and the
API wrapper into `app/src/lib/api/` moved about 600 lines of untested code into `.ts` files, which are
measured: the lines went from unmeasured to uncovered, and no tested line was lost. The figures above are
the production code as it is now and are the baseline for the later steps. Raising them is the job of the
scenario tests (the shell store with a faked `invoke`).

Per file, lines %: `app/src/lib/groupings.ts`, `logoPrompt.ts`, `printed.ts`, `saveCheck.ts` and
`unconfigured.ts` 100; `app/src/texts.ts` 99.56; `app/src/lib/api/` 32 to 59; `categoryEntry.ts` 28.85;
`conditions.ts` 22.07; `shell.svelte.ts` 0.79; `app/src/routes/+layout.ts` 0.

## What no test reaches

- The 14 Svelte components under `app/src`: the screens (`DayScreen`, `NewBatchScreen`, `PlanScreen`,
  `CategoriesScreen`, `SearchScreen`, `SettingsScreen`), the dialogs and frames (`Frame`, `Overlay`,
  `SaveCheckDialog`, `BoxesEditor`, `LogoPrompt`, `StartPrompt`) and `app/src/routes/+layout.svelte`
  and `+page.svelte`. No test renders a component in this refactor; `svelte-check` and the front-end
  tests are the only guard ([plan/refactor.md](plan/refactor.md), "Choices taken").
- `app/src-tauri/src/main.rs`: it only starts the window.

The front-end flow tests reach the logic behind the screens through the shared `.ts` modules
(`logoPrompt.ts`, `saveCheck.ts`, `printed.ts`, `sets.ts`, `unconfigured.ts`), which is why those are
at 100 % while the components that call them are at 0 %. `conditions.ts` and `api.ts` are loaded by a
test for their types only, so most of their statements never run.
