# Packing Checker — PC app

A Windows program (Rust + Tauri) for the shop's daily packing work (TikTok Shop / Tokopedia orders):
shipping-label PDFs with packing slip → one label PDF per pick (pages unchanged) and A4 packing lists.

- **PC app** (repository root): `crates/` (engine, PDF, developer CLI), `app/` (Tauri window with Svelte
  screens), `tools/` (PDFium download, portable build).
- **Analysis tools** (Python, `tools/analysis/`): helpers for looking inside the label PDFs and CSVs;
  frozen, not part of the product.

Documentation: [`docs/`](docs/README.md). Status and next step: [`docs/development/plan/`](docs/development/plan/README.md).
Start with [`docs/development/`](docs/development/README.md).

`samples/`, `work/` and `tmp/` contain real order data: confidential, git-ignored, never committed.
