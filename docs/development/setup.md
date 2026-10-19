# Setup

Git and Git Bash are needed for both tools ([README.md](README.md)); the steps below are for a
fresh clone of the repository, run from its root in Git Bash.

## What is needed

| Tool | Why | Tested version |
|---|---|---|
| Git and Git Bash | Clone the repository and run the shell scripts (`tools/get-pdfium.sh`, `tools/make-portable.sh`) | — |
| Python | The analysis helpers only (`tools/analysis`) | 3.12 or newer; 3.14 at `C:\Python314` (on `PATH` as `python`) |
| Python packages `pypdf`, `fpdf2`, `pillow`, `xxhash`, `pytest` | Reading PDFs and running the analysis helpers' tests; declared in `tools/analysis/pyproject.toml` | installed by `pip install -e "tools/analysis[dev]"` |
| Project virtual environment `.venv` | Holds the analysis helpers' packages | in the repository root |
| Fonts Arial and Consolas | The A4 packing lists | come with Windows (`C:\Windows\Fonts`) |
| Rust (stable, MSVC toolchain) | The PC app and `packing-cli` | cargo/rustc 1.98.0 |
| Node.js and npm | The Svelte screens and the Tauri build | Node 26.7, npm 11.19 |
| Microsoft C++ Build Tools | Links the Tauri crate | at `C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools` |
| WebView2 runtime | The window (Edge WebView2, bundled with Windows 11) | at `C:\Program Files (x86)\Microsoft\EdgeWebView\Application` |
| `tauri-cli` | The Tauri build tool, run as `cargo tauri` | 2.12.1 (`cargo install tauri-cli --locked`), matching the `tauri` 2.12 of [architecture.md](../software-design/app/architecture.md) |
| `pdfium.dll` | PDFium: read the page text, copy pages, write the packing lists ([program.md](../software-design/app/program.md)) | `chromium/7881` from bblanchon/pdfium-binaries, matching `pdfium-render` 0.9.4; fetched by `bash tools/get-pdfium.sh` into `vendor/pdfium.dll` (git-ignored), SHA256 checked, with a Python fallback for the download because `curl` fails on GitHub release files on this PC |

## Steps

### Python analysis tools (`tools/analysis`)

```bash
cd /d/workspace/software/olshop-packing-ai    # the clone
/c/Python314/python.exe -m venv .venv      # the system Python, not another one on PATH
source .venv/Scripts/activate
python -m pip install --upgrade pip
python -m pip install -e "tools/analysis[dev]"     # pypdf, fpdf2, pytest
python -m pytest tools/analysis/tests
```

`-e` installs the package in editable mode, so `python -m packing` works from any folder while
the venv is active. The helpers' commands are in [cli.md](cli.md).

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
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

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
