# Desktop — PC app and PC script

The Windows side of the packing work: the **PC app** (Rust + Tauri, `pc/`) that turns the
shipping-label PDFs with packing slip into picks — one saved label PDF per pick — and A4 packing
lists, and the Python **PC script** (`script/`), the reference implementation with the same rules.

| Folder | Contains |
|---|---|
| [`design/`](design/README.md) | What to build: requirements, open questions for the owner, the script, the app, its screens |
| [`development/`](development/README.md) | Setup log (`01-setup.md`) and commands |
| [`user-guide/`](user-guide/README.md) | How to use the finished PC app, step by step with pictures (for the owner) |
| [`phases/`](phases/) | One folder per phase: `plan.md` (tasks) and `review-N.md` (owner reviews). Phase 1 = PC script, phase 2 = PC app |

Status and the next step: [`../development/plan.md`](../development/plan.md).
