# 01 — Setup (Android app)

Git and Git Bash are common to both tools: [`../../development/README.md`](../../development/README.md#common-setup).

## What is needed

| For | Tool | Notes |
|---|---|---|
| Android app | Android Studio (includes a JDK and the Android SDK manager) | Not installed yet. `winget install Google.AndroidStudio`, then first-run wizard with the default SDK |
| Android app | Android phone with USB debugging | Settings › About phone › tap *Build number* 7× → Developer options › USB debugging. Phone model and Android version go into the log below |
| Android app | `adb` | Comes with the SDK (`%LOCALAPPDATA%\Android\Sdk\platform-tools`); add to `PATH` |

## Steps — Android (phase 3)

1. Install Android Studio; run the setup wizard (standard install, accept SDK licences).
2. Open `android/` in Android Studio once so it downloads Gradle and the SDK platforms; after
   that, command-line builds with `./gradlew` work.
3. Connect the phone, accept the "Allow USB debugging" prompt, check with `adb devices`.

## Log

One line per one-time step actually done on this PC: date · what · exact command or click path.

| Date | Step |
|---|---|
