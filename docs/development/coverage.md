# Coverage

Coverage per language, measured by `tools/check-coverage.mjs` and held in
`tools/baselines/coverage.json`. The gate compares the Rust totals and per-crate figures and the
TypeScript totals, and fails when one is lower than the baseline by more than 0.1 point; a baseline
entry only shrinks. Run it with `bash tools/check.sh --coverage`.

The two measurements are `cargo llvm-cov --workspace --json --summary-only` (Rust; the instrumented
build goes to the folder in `CARGO_LLVM_COV_TARGET_DIR`, see [setup.md](setup.md)) and `node --test
--experimental-test-coverage` from `app/` (TypeScript). The numbers below are the state at commit
`490e48d`, measured 2026-10-19, before the refactor of [plan/refactor.md](plan/refactor.md).

## Rust

| Scope | Lines % | Regions % | Functions % |
|---|---|---|---|
| Total | 86.29 | 86.56 | 79.56 |
| `app/src-tauri` | 86.38 | 88.11 | 70.65 |
| `crates/cli` | 25.44 | 23.17 | 22.73 |
| `crates/engine` | 91.10 | 91.46 | 92.64 |
| `crates/pdf` | 87.97 | 86.99 | 82.10 |

Under 50 % of lines: `app/src-tauri/src/main.rs` (0 %, the window's six-line entry point) and
`crates/cli/src/main.rs` (25.44 %, the developer command line; only the batch steps the tests drive
run). No other file is under 50 %; the engine's library modules are between 92 % and 100 %. A Rust
test file is not part of the measurement.

## TypeScript

Only the files a test loads are instrumented, so a `.ts` file no test loads counts as 0 % (the run
adds `--test-coverage-include-all` for that).

| Scope | Lines % | Branches % | Functions % |
|---|---|---|---|
| Total | 72.43 | 98.88 | 66.67 |

Per file, lines %: `app/src/lib/logoPrompt.ts`, `printed.ts`, `saveCheck.ts`, `sets.ts` and
`unconfigured.ts` 100; `app/src/texts.ts` 99.56; `app/src/lib/api.ts` 50.34; `app/src/lib/conditions.ts`
22.07; `app/src/routes/+layout.ts` 0.

Under 50 % of lines: `app/src/lib/conditions.ts` (22.07 %, the condition-language evaluator, reached
only through its shared field table) and `app/src/routes/+layout.ts` (0 %, the five-line layout load
function).

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
