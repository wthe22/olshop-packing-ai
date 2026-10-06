# 01 — Setup

## What is needed

| For | Tool | Notes |
|---|---|---|
| Both | Git, Git Bash | Installed |
| PC script | Python 3.12 or newer | Python 3.14 is installed at `C:\Python314` (on `PATH` as `python`) |
| PC script | Project virtual environment `.venv` in the repository root | Created in phase 1, task 1.1 |
| PC script | `pypdf`, `fpdf2`; for development `pytest` | Declared in `script/pyproject.toml` |
| PC script | Fonts Arial and Consolas | Come with Windows (`C:\Windows\Fonts`) |
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

### Android (phase 2)

1. Install Android Studio; run the setup wizard (standard install, accept SDK licences).
2. Open `android/` in Android Studio once so it downloads Gradle and the SDK platforms; after
   that, command-line builds with `./gradlew` work.
3. Connect the phone, accept the "Allow USB debugging" prompt, check with `adb devices`.

## Log

One line per one-time step actually done on this PC: date · what · exact command or click path.

| Date | Step |
|---|---|
| 2026-10-06 | Git repository created, remote `origin` = `https://github.com/wthe22/olshop-packing-ai.git`, branch `main` |
