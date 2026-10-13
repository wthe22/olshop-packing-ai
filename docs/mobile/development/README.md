# Development — mobile (Android app)

Setup and commands for the Android app. Rules, session start and the repository layout are in
[`../../development/README.md`](../../development/README.md); the phases (3–7) are in
[`../phases/`](../phases/) and [`../../development/plan.md`](../../development/plan.md). Not
started: phase 3 waits for the desktop phase-2 review.

| File | Content |
|---|---|
| [01-setup.md](01-setup.md) | Tools to install (Android Studio, phone, `adb`), one-time steps, setup log |

Design to read: [`../design/`](../design/README.md) — the old design, revised before phase 3
([`../design/02-design-questions.md`](../design/02-design-questions.md) D1).

## Commands (once set up)

| Task | Command (from the repository root, Git Bash) |
|---|---|
| App build (debug APK) | `cd android && ./gradlew assembleDebug` |
| App unit tests | `cd android && ./gradlew test` |
| Install on the phone | `cd android && ./gradlew installDebug` (phone on USB, debugging on) |
