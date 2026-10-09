# Software Design

What the two tools do and how they are built. Where it disagrees with
[`../business-process/`](../business-process/), reality wins and the design is fixed.

| File | Content |
|---|---|
| [01-requirements.md](01-requirements.md) | Concepts, order statuses, the PC script's pick rules, feature list of the PC script (P…) and the app (A…) |
| [02-design-questions.md](02-design-questions.md) | Open questions for the owner (each with options and what is built for now), and Android decisions not yet designed |
| [03-pc-script.md](03-pc-script.md) | The Python PC script (reference implementation): reading the labels and slips, picks, runs, numbering, the three packing-list layouts — rules the PC app reuses |
| [04-rules-file.md](04-rules-file.md) | `categories.toml` and the condition language shared by the script and the app |
| [05-app-architecture.md](05-app-architecture.md) | Android app: stack, modules, database, import, scan pipeline, session and settings files |
| [06-app-ui.md](06-app-ui.md) | Android app screens, layouts, scan messages, settings |
| [07-pc-app.md](07-pc-app.md) | PC app (Rust + Tauri): what changes from the script, one batch step by step, day folder and `state.json`, revert/amend, architecture, tests |
| [08-pc-app-ui.md](08-pc-app-ui.md) | PC app screens: Day, New batch, Plan, messages, Categories, Settings, first start |

## Conventions

- Timestamps are named `*_time` (`create_time`, `import_time`) and stored as ISO 8601 with the
  UTC offset (`2026-10-06T14:26:00+07:00`).
- Items are identified only by the platform `SKU ID`. Product name, variation and Seller SKU are
  display values that can change between two exports of the same day.
- Orders are identified by `Order ID`.
- Requirement IDs (`P3`, `A2`) are stable references; code comments and tests may cite them.
