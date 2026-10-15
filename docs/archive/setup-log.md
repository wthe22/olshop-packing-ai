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
| 2026-10-13 | `script/` moved to `tools/analysis/` (analysis tools, not product code): `python -m pip install -e "tools/analysis[dev]"` re-pointed the editable install; `python -m pytest tools/analysis/tests` → 270 passed |
