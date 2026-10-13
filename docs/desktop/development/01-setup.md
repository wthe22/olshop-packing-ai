# 01 — Setup (PC app and PC script)

Git and Git Bash are common to both tools: [`../../development/README.md`](../../development/README.md#common-setup).

## What is needed

| For | Tool | Notes |
|---|---|---|
| PC script | Python 3.12 or newer | Python 3.14 is installed at `C:\Python314` (on `PATH` as `python`) |
| PC script | Project virtual environment `.venv` in the repository root | Created in phase 1, task 1.1 |
| PC script | `pypdf`, `fpdf2`; for development `pytest` | Declared in `script/pyproject.toml` |
| PC script | Fonts Arial and Consolas | Come with Windows (`C:\Windows\Fonts`) |
| PC app | Rust (stable, MSVC toolchain) | Installed: cargo/rustc 1.98.0 |
| PC app | Node.js + npm | Installed: Node 26.7, npm 11.19 |
| PC app | Tauri prerequisites: Microsoft C++ Build Tools, WebView2 | Installed: Build Tools at `C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools` (links the Tauri crate); WebView2 runtime present (`C:\Program Files (x86)\Microsoft\EdgeWebView\Application`) |
| PC app | `tauri-cli` (the Tauri build tool, run as `cargo tauri`) | Installed: tauri-cli 2.12.1 (`cargo install tauri-cli --locked`), matching the `tauri` 2.12 stated in [07-pc-app.md](../design/07-pc-app.md) |
| PC app | `pdfium.dll` (bblanchon/pdfium-binaries, pinned) | `bash pc/tools/get-pdfium.sh` → `pc/vendor/pdfium.dll` (git-ignored). Tag `chromium/7881` (matches `pdfium-render` 0.9.4), SHA256 checked; falls back to Python for the download because `curl` fails on GitHub release files on this PC |

## Steps

### Python (phase 1)

```bash
cd /d/workspace/software/olshop-packing-ai
/c/Python314/python.exe -m venv .venv      # the system Python, not another one on PATH
source .venv/Scripts/activate
python -m pip install --upgrade pip
python -m pip install -e "script[dev]"     # pypdf, fpdf2, pytest
python -m pytest script/tests
```

`-e` installs the package in editable mode, so `python -m packing` works from any folder while
the venv is active.

### PC app (phase 2)

From a fresh clone, with the tools in the table above installed:

```bash
cd pc
bash tools/get-pdfium.sh     # downloads pc/vendor/pdfium.dll (git-ignored)
cd app && npm install        # local node_modules (git-ignored)
npm run check                # svelte-check
npm run build                # front-end to pc/app/build/
npm run tauri dev            # opens the window
```

The Rust workspace gates, run from `pc/`:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

### Build the program folder (portable)

The program folder (one folder with `Packing.exe` + `pdfium.dll`, no installer; 07 › *Program
folder*) is built by `pc/tools/make-portable.sh`. It needs `pc/vendor/pdfium.dll`
(`get-pdfium.sh`) and takes a few minutes (a release build; the first one is the slowest):

```bash
bash pc/tools/make-portable.sh          # → pc/target/portable/Packing/
```

`pc/tools/make-portable.bat` is the same thing for Windows Explorer and `cmd`: double-click it (or
`pc\tools\make-portable.bat`) and it finds Git for Windows' `bash.exe` itself, runs the script and
waits for a key so the output stays readable. `make-portable.bat --no-pause` builds and returns (for
scripts, e.g. from another `.bat`).

Copy `pc/target/portable/Packing/` to the PC (e.g. `D:\Packing\`) and start `Packing.exe` from a
shortcut. To update, close the app and replace `Packing.exe` and `pdfium.dll`; the data stays.

## Log

One line per one-time step actually done on this PC: date · what · exact command or click path.

| Date | Step |
|---|---|
| 2026-10-07 | `.venv` created: `/c/Python314/python.exe -m venv .venv` (Python 3.14.7), `python -m pip install --upgrade pip` (26.2.1), `python -m pip install -e "script[dev]"` → pypdf 6.19.0, fpdf2 2.8.9, pytest 9.1.1 |
| 2026-10-09 | `bash pc/tools/get-pdfium.sh` → `pc/vendor/pdfium.dll` from bblanchon/pdfium-binaries `chromium/7881` (`pdfium-win-x64.tgz`, 3,733,154 B, SHA256 `73cc0de6…77ac08`); `curl` failed (schannel, error 23), downloaded by the script's Python fallback |
| 2026-10-10 | `cargo install tauri-cli --locked` → tauri-cli 2.11.4 replaced by 2.12.1 at `C:\Users\wthe22\.cargo\bin\cargo-tauri.exe` (built from source, ~4½ min) |
| 2026-10-10 | PC app scaffold: in `pc/`, `npm create tauri-app@latest app -- --template svelte-ts --manager npm --identifier com.wthe22.olshop-packing --yes` → create-tauri-app 4.7.5 (Tauri 2, SvelteKit + TypeScript) |
| 2026-10-10 | `cd pc/app && npm install` → @tauri-apps/api 2.12.2, @tauri-apps/cli 2.12.1, svelte 5.57.2, @sveltejs/kit 2.70.3, vite 8.3.4, typescript 6.0.3, svelte-check 4.7.6 |
| 2026-10-10 | PC app versions confirmed: `rustc`/`cargo` 1.98.0, `node` 26.7.0, `npm` 11.19.0, `tauri-cli` 2.12.1 (table above) |
| 2026-10-10 | Task 2.8 packages: `cd pc/app && npm install @tauri-apps/plugin-dialog` → @tauri-apps/plugin-dialog 2.8.1; `pc/app/src-tauri/Cargo.toml` gained `tauri-plugin-dialog = "2"` (resolved 2.8.1) plus the workspace path deps `packing-engine`, `packing-pdf` and `jiff`; the dialog plugin is registered in `lib.rs` and `dialog:default` added to `capabilities/default.json` |
| 2026-10-10 | Task 2.13 portable build: `bash pc/tools/make-portable.sh` → `pc/target/portable/Packing/` (`Packing.exe` 7,792,128 B, `pdfium.dll` 7,211,520 B; release build 4m11s cold, 2m58s warm). Tested from a copy outside the repository at `%TEMP%\portable-test\Packing` via a `.lnk` shortcut: the first start asked for the data folder, *Use this folder* wrote the built-in `categories.toml` (482 B), one batch on `testdata/labels-slip.pdf` → 24 pages, 23 orders, 3 saved PDFs (`1 A Sepatu ×7`, `2 B Spion & Knalpot ×8`, `3 Z Lainnya ×8`) + `packing-list.pdf` (1.4 MB) in `labels/2026-10-10/batch 1/` |
| 2026-10-11 | Analysis helper (not needed to build or run the project): `.venv/Scripts/python.exe -m pip install pypdfium2` → pypdfium2 5.14.0, used to render sample label pages to PNG under `tmp/probe/img/` while comparing the SiCepat, J&T Express, J&T Cargo and IDX label layouts |
| 2026-10-11 | Analysis helper: `.venv/Scripts/python.exe -m pip install xxhash` → xxhash 4.0.1, used to compute the logo fingerprints (XXH3-64) in `tmp/probe/logos/` and to check them against Rust's `xxhash-rust` (both match the known XXH3-64 vector `2d06800538d394c2`, so the script and the app share one expected-value fixture) |
