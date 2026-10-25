# Setup

Git and Git Bash are needed for both tools ([README.md](README.md)); the steps below are for a
fresh clone of the repository, run from its root in Git Bash.

## What is needed

| Tool | Why | Tested version |
|---|---|---|
| Git and Git Bash | Clone the repository and run the shell scripts (`tools/get-pdfium.sh`, `tools/make-portable.sh`) | — |
| Fonts Arial and Consolas | The A4 packing lists | come with Windows (`C:\Windows\Fonts`) |
| Rust (stable, MSVC toolchain) | The PC app and `packing-cli` | cargo/rustc 1.98.0 |
| Node.js and npm | The Svelte screens and the Tauri build | Node 26.12, npm 11.20 |
| Microsoft C++ Build Tools | Links the Tauri crate | at `C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools` |
| WebView2 runtime | The window (Edge WebView2, bundled with Windows 11) | at `C:\Program Files (x86)\Microsoft\EdgeWebView\Application` |
| `tauri-cli` | The Tauri build tool, run as `cargo tauri` | 2.12.1 (`cargo install tauri-cli --locked`), matching the `tauri` 2.12 of [architecture.md](../software-design/app/architecture.md) |
| `pdfium.dll` | PDFium: read the page text, copy pages, write the packing lists ([program.md](../software-design/app/program.md)) | `chromium/7881` from bblanchon/pdfium-binaries, matching `pdfium-render` 0.9.4; fetched by `bash tools/get-pdfium.sh` into `vendor/pdfium.dll` (git-ignored), SHA256 checked, with a PowerShell fallback for the download because `curl` fails on GitHub release files on this PC |
| `cargo-llvm-cov` | The Rust coverage measurement of `bash tools/check.sh --coverage` | 0.9.1 (`cargo install cargo-llvm-cov --locked`) |
| `llvm-tools-preview` (rustup component) | The `llvm-cov` and `llvm-profdata` binaries `cargo-llvm-cov` calls | `rustup component add llvm-tools-preview` |

## Steps

### PC app

From a fresh clone, with the tools in the table above installed:

```bash
bash tools/get-pdfium.sh     # downloads vendor/pdfium.dll (git-ignored)
cd app && npm install        # local node_modules (git-ignored)
npm run check                # svelte-check
npm run build                # front-end to app/build/
npm run tauri dev            # opens the window
```

The Rust workspace gates, run from the repository root (the full list, with the code-rule checks, is in
[code-rules.md](code-rules.md)):

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings -D clippy::too_many_arguments -A clippy::cognitive_complexity
cargo test --workspace
```

### Coverage tools

```bash
cargo install cargo-llvm-cov --locked       # cargo llvm-cov
rustup component add llvm-tools-preview     # llvm-cov, llvm-profdata
export CARGO_LLVM_COV_TARGET_DIR=C:/Users/wthe22/llvm-cov-target
```

The instrumented Rust build is about 5 GB, so `CARGO_LLVM_COV_TARGET_DIR` moves it to a drive with
room (`C:` on this PC, `target/` lives on `D:`). `tools/check-coverage.mjs` uses that folder when the
variable is set, so it is exported before `bash tools/check.sh --coverage`. The measured numbers and
what no test reaches are in [coverage.md](coverage.md).

### Build the program folder (portable)

The program folder (one folder with `Packing.exe` + `pdfium.dll`, no installer; see
[program.md](../software-design/app/program.md)) is produced by `tools/make-portable.sh`. It needs
`vendor/pdfium.dll` (`get-pdfium.sh`) and takes a few minutes (a release build; the first one is the
slowest):

```bash
bash tools/make-portable.sh          # → target/portable/Packing/
```

`tools/make-portable.bat` is the same thing for Windows Explorer and `cmd`: double-click it (or
`tools\make-portable.bat`) and it finds Git for Windows' `bash.exe` itself, runs the script and
waits for a key so the output stays readable. `make-portable.bat --no-pause` builds and returns (for
scripts, e.g. from another `.bat`).

Copy `target/portable/Packing/` to the PC (e.g. `D:\Packing\`) and start `Packing.exe` from a
shortcut. To update, close the app and replace `Packing.exe` and `pdfium.dll`; the data stays.
