# Packing Checker

Tools for the shop's daily packing work (TikTok Shop / Tokopedia orders):

- **PC app** (Rust + Tauri, `pc/`): shipping-label PDFs with packing slip → one label PDF per
  pick (pages unchanged) and A4 packing lists, in a Windows program. Designed (phase 2).
- **PC script** (Python, `script/`): the same rules from the command line; built in phase 1,
  now the reference implementation for the PC app.
- **Android app** (Kotlin, `android/`): checks every packed parcel by scanning its label with
  the phone camera; sessions, batches, marks with undo, export/import.

Status: see [`docs/development/02-plan.md`](docs/development/02-plan.md). Start with [`docs/development/`](docs/development/).

Documentation: [`docs/`](docs/README.md).

`samples/` and `work/` contain real order data: confidential, git-ignored, never committed.
