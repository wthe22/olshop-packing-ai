# Packing Checker

Tools for the shop's daily packing work (TikTok Shop / Tokopedia orders):

- **PC script** (Python, `script/`): orders CSV + shipping-label PDFs → one label PDF per pack
  group (pages unchanged) and an A4 packing list.
- **Android app** (Kotlin, `android/`): checks every packed parcel by scanning its label with
  the phone camera; sessions, batches, marks with undo, export/import.

Status: designed, not implemented. Start with [`docs/development/`](docs/development/).

Documentation: [`docs/`](docs/README.md).

`samples/` and `work/` contain real order data: confidential, git-ignored, never committed.
