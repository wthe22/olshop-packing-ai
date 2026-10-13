# Mobile — Design

What the Android app does and how it is built. **This is the old design** (batches from the orders
CSV, pack groups, scan filters on batch/group); it is revised as its own checking workflow before
phase 3 ([02-design-questions.md](02-design-questions.md) D1). Conventions and the rules file shared
with the PC tools are in [`../../shared/`](../../shared/README.md). Where a design disagrees with
[`../../business-process/`](../../business-process/README.md), reality wins and the design is fixed.

| File | Content |
|---|---|
| [01-requirements.md](01-requirements.md) | App concepts, order statuses, feature list A1–A7 |
| [02-design-questions.md](02-design-questions.md) | Owner decisions not yet designed, and D1 (the app revision) |
| [`../../shared/04-rules-file.md`](../../shared/04-rules-file.md) | `categories.toml` and the condition language (shared with the PC tools) |
| [05-app-architecture.md](05-app-architecture.md) | Stack, modules, database, import, scan pipeline, session and settings files |
| [06-app-ui.md](06-app-ui.md) | Screens, layouts, scan messages, settings |

The file numbers (01–08) are shared with the desktop design
([`../../desktop/design/`](../../desktop/design/README.md): 01, 02, 03, 07, 08), so `05 › Database` in
a code comment or a task means the mobile file.
