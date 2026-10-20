# Coverage

Coverage per language, measured by `tools/check-coverage.mjs` and held in
`tools/baselines/coverage.json`. The gate compares the Rust totals and per-crate figures and the
TypeScript totals, and fails when one is lower than the baseline by more than 0.1 point; a baseline
entry only shrinks. Run it with `bash tools/check.sh --coverage`.

The two measurements are `cargo llvm-cov --workspace --json --summary-only` (Rust; the instrumented
build goes to the folder in `CARGO_LLVM_COV_TARGET_DIR`, see [setup.md](setup.md)) and `node --test
--experimental-test-coverage` from `app/` (TypeScript). The numbers below are the state at commit
`81fb900`, measured 2026-10-19, after the first refactor step (the code split into modules) and before the
rest of the first refactor.

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
| Total | 97.04 | 94.42 | 75.17 |

The first baseline (72.43 lines) was taken while the screens' logic sat inside `.svelte` files, which no
run measures; splitting the shell out of `+page.svelte` moved it into `.ts` files, which are measured, and
the scenario tests of the shell store (a faked `invoke`) then covered it (97.04 lines). Branches went from
98.88 to 94.42 because the earlier figure counted a file no test loads as having no branches; the branches
left uncovered are `$derived` initialisers that only the Svelte compiler evaluates. Per file, lines %: all
`.ts` modules are at 91 to 100 except `app/src/lib/api/search.ts` (52) and `app/src/routes/+layout.ts` (0, the
five-line layout load).

## What no test reaches

- The 14 Svelte components under `app/src`: the screens (`DayScreen`, `NewBatchScreen`, `PlanScreen`,
  `CategoriesScreen`, `SearchScreen`, `SettingsScreen`), the dialogs and frames (`Frame`, `Overlay`,
  `SaveCheckDialog`, `BoxesEditor`, `LogoPrompt`, `StartPrompt`) and `app/src/routes/+layout.svelte`
  and `+page.svelte`. No test renders a component in this refactor; `svelte-check` and the front-end
  tests are the only guard ([code-rules.md](code-rules.md), "Choices taken").
- `app/src-tauri/src/main.rs`: it only starts the window.

The front-end flow tests reach the logic behind the screens through the shared `.ts` modules
(`logoPrompt.ts`, `saveCheck.ts`, `printed.ts`, `sets.ts`, `unconfigured.ts`), which is why those are
at 100 % while the components that call them are at 0 %. `conditions.ts` and `api.ts` are loaded by a
test for their types only, so most of their statements never run.
