# PC app

Run, build and test the PC app in the repository. The tools to install are in [setup.md](setup.md);
the developer command line is in [cli.md](cli.md).

## Run in development

From the repository root, Git Bash:

| Task | Command |
|---|---|
| Front-end checks (svelte-check) | `cd app && npm run check` |
| Front-end build (to `app/build/`) | `cd app && npm run build` |
| Open the window | `cd app && npm run tauri dev` |

## Gates and tests

| Gate | Command |
|---|---|
| All the checks, in order | `bash tools/check.sh` (`--coverage` adds the coverage compare) |
| Rust format | `cargo fmt --all --check` |
| Rust lints | `cargo clippy --workspace --all-targets -- -D warnings -D clippy::too_many_arguments -A clippy::cognitive_complexity` |
| Rust tests | `cargo test --workspace` |
| Front-end types | `cd app && npm run check` |
| Front-end tests | `cd app && npm test` |
| File size ratchet | `node tools/check-size.mjs` |
| Lint ratchet | `node tools/check-lint.mjs` |
| Comment gate | `node tools/check-comments.mjs` |
| Coverage compare | `node tools/check-coverage.mjs` |

Every change also meets the [code rules](code-rules.md): its checks, run together by `bash tools/check.sh`,
are listed there. The two ratchets read the counts in `tools/baselines/` and fail for a file that grew or
is new — the lint ratchet counts the cognitive-complexity warning and every suppression, while the parameter
limit (at most five) is denied by the clippy gate itself; the coverage numbers and what no test reaches are
in [coverage.md](coverage.md).

`cargo test --workspace` runs the engine, PDF and app tests; the test levels and their expected
sample numbers are in [testing.md](testing.md).

## Build the portable folder

| Task | Command |
|---|---|
| Download `pdfium.dll` | `bash tools/get-pdfium.sh` → `vendor/pdfium.dll` (git-ignored) |
| Build the program folder | `bash tools/make-portable.sh` → `target/portable/Packing/` |
| The same from Explorer or `cmd` | `tools\make-portable.bat` (double-click; `--no-pause` for scripts) |

The folder holds `Packing.exe` + `pdfium.dll` and no installer. Copy `target/portable/Packing/` to
the PC and start `Packing.exe` from a shortcut; to update, close the app and replace `Packing.exe`
and `pdfium.dll`, the data stays. See [program.md](../software-design/app/program.md).

## Owner's guide

How the owner uses the finished app: [../user-guide/README.md](../user-guide/README.md).
