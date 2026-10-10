# 01 — Setup

## What is needed

| For | Tool | Notes |
|---|---|---|
| Both | Git, Git Bash | Installed |
| PC script | Python 3.12 or newer | Python 3.14 is installed at `C:\Python314` (on `PATH` as `python`) |
| PC script | Project virtual environment `.venv` in the repository root | Created in phase 1, task 1.1 |
| PC script | `pypdf`, `fpdf2`; for development `pytest` | Declared in `script/pyproject.toml` |
| PC script | Fonts Arial and Consolas | Come with Windows (`C:\Windows\Fonts`) |
| PC app | Rust (stable, MSVC toolchain) | Installed: cargo/rustc 1.98.0 |
| PC app | Node.js + npm | Installed: Node 26.7, npm 11.19 |
| PC app | Tauri prerequisites: Microsoft C++ Build Tools, WebView2 | Installed: Build Tools at `C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools` (links the Tauri crate); WebView2 runtime present (`C:\Program Files (x86)\Microsoft\EdgeWebView\Application`) |
| PC app | `tauri-cli` (the Tauri build tool, run as `cargo tauri`) | Installed: tauri-cli 2.12.1 (`cargo install tauri-cli --locked`), matching the `tauri` 2.12 stated in [07-pc-app.md](../software-design/07-pc-app.md) |
| PC app | `pdfium.dll` (bblanchon/pdfium-binaries, pinned) | `bash pc/tools/get-pdfium.sh` → `pc/vendor/pdfium.dll` (git-ignored). Tag `chromium/7881` (matches `pdfium-render` 0.9.4), SHA256 checked; falls back to Python for the download because `curl` fails on GitHub release files on this PC |
| App | Android Studio (includes a JDK and the Android SDK manager) | Not installed yet. `winget install Google.AndroidStudio`, then first-run wizard with the default SDK |
| App | Android phone with USB debugging | Settings › About phone › tap *Build number* 7× → Developer options › USB debugging. Phone model and Android version go into the log below |
| App | `adb` | Comes with the SDK (`%LOCALAPPDATA%\Android\Sdk\platform-tools`); add to `PATH` |

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

### Android (phase 3)

1. Install Android Studio; run the setup wizard (standard install, accept SDK licences).
2. Open `android/` in Android Studio once so it downloads Gradle and the SDK platforms; after
   that, command-line builds with `./gradlew` work.
3. Connect the phone, accept the "Allow USB debugging" prompt, check with `adb devices`.

## Log

One line per one-time step actually done on this PC: date · what · exact command or click path.

| Date | Step |
|---|---|
| 2026-10-06 | Git repository created, remote `origin` = `https://github.com/wthe22/olshop-packing-ai.git`, branch `main` |
| 2026-10-07 | `.venv` created: `/c/Python314/python.exe -m venv .venv` (Python 3.14.7), `python -m pip install --upgrade pip` (26.2.1), `python -m pip install -e "script[dev]"` → pypdf 6.19.0, fpdf2 2.8.9, pytest 9.1.1 |
| 2026-10-09 | `bash pc/tools/get-pdfium.sh` → `pc/vendor/pdfium.dll` from bblanchon/pdfium-binaries `chromium/7881` (`pdfium-win-x64.tgz`, 3,733,154 B, SHA256 `73cc0de6…77ac08`); `curl` failed (schannel, error 23), downloaded by the script's Python fallback |
| 2026-10-10 | `cargo install tauri-cli --locked` → tauri-cli 2.11.4 replaced by 2.12.1 at `C:\Users\wthe22\.cargo\bin\cargo-tauri.exe` (built from source, ~4½ min) |
| 2026-10-10 | PC app scaffold: in `pc/`, `npm create tauri-app@latest app -- --template svelte-ts --manager npm --identifier com.wthe22.olshop-packing --yes` → create-tauri-app 4.7.5 (Tauri 2, SvelteKit + TypeScript) |
| 2026-10-10 | `cd pc/app && npm install` → @tauri-apps/api 2.12.2, @tauri-apps/cli 2.12.1, svelte 5.57.2, @sveltejs/kit 2.70.3, vite 8.3.4, typescript 6.0.3, svelte-check 4.7.6 |
| 2026-10-10 | PC app versions confirmed: `rustc`/`cargo` 1.98.0, `node` 26.7.0, `npm` 11.19.0, `tauri-cli` 2.12.1 (table above) |
