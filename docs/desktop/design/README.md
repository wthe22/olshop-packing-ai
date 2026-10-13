# Desktop — Design

What the PC app (Rust + Tauri, `pc/`) does and how it is built. Conventions and the rules file shared with the
Android app are in [`../../shared/`](../../shared/README.md). Where a design disagrees with
[`../../business-process/`](../../business-process/README.md), reality wins and the design is fixed.

| File | Content |
|---|---|
| [01-requirements.md](01-requirements.md) | Concepts, the pick flow (P…) and the app's features (U…) |
| [02-design-questions.md](02-design-questions.md) | Open questions for the owner (each with options and what is built for now) |
| [03-labels-and-picks.md](03-labels-and-picks.md) | The rules the app uses: reading the labels and slips, courier deduction, picks, runs, numbering, the three packing-list layouts |
| [`../../shared/04-rules-file.md`](../../shared/04-rules-file.md) | `categories.toml` and the condition language (shared with the Android app) |
| [07-pc-app.md](07-pc-app.md) | PC app (Rust + Tauri): what the app decides, one batch step by step, day folder and `state.json`, revert/amend, architecture, tests |
| [08-pc-app-ui.md](08-pc-app-ui.md) | PC app screens: Day, New batch, Plan, messages, Categories, Settings, first start |

The file numbers (01–08) are shared with the Android design ([`../../mobile/design/`](../../mobile/design/README.md):
01, 02, 05, 06), so `07 › Day folder` in a code comment or a task means the desktop file.
