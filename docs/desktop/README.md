# Desktop — PC app

The Windows side of the packing work: the **PC app** (Rust + Tauri, `pc/`) that turns the
shipping-label PDFs with packing slip into picks — one saved label PDF per pick — and A4 packing
lists. The Python programs in [`tools/analysis/`](../../tools/analysis/README.md) (the first
implementation of the same rules, frozen) are an analysis aid for looking inside PDFs, not part of
the app.

| Folder | Contains |
|---|---|
| [`design/`](design/README.md) | What to build: requirements, open questions for the owner, the label and pick rules, the app, its screens |
| [`development/`](development/README.md) | Setup log (`01-setup.md`) and commands |
| [`user-guide/`](user-guide/README.md) | How to use the finished PC app, step by step with pictures (for the owner) |
| [`phases/`](phases/) | One folder per phase: `plan.md` (tasks) and `review-N.md` (owner reviews). Phase 1 = the Python script (done; now `tools/analysis`), phase 2 = PC app |

Status and the next step: [`../development/plan.md`](../development/plan.md).
