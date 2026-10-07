# Software Design

What the two tools do and how they are built. Where it disagrees with
[`../business-process/`](../business-process/), reality wins and the design is fixed.

| File | Content |
|---|---|
| [01-requirements.md](01-requirements.md) | Concepts, order statuses, the PC script's pick rules, feature list of the PC script (P…) and the app (A…) |
| [02-design-questions.md](02-design-questions.md) | Questions still open for the owner (D1 the app, D2 interactive mode, D3 packing-list default) |
| [03-pc-script.md](03-pc-script.md) | The PC script: inputs, command, picks, saved-PDF numbering, the day state, the three packing-list layouts |
| [04-rules-file.md](04-rules-file.md) | `categories.toml` and the condition language shared by the script and the app |
| [05-app-architecture.md](05-app-architecture.md) | Android app: stack, modules, database, import, scan pipeline, session and settings files |
| [06-app-ui.md](06-app-ui.md) | Android app screens, layouts, scan messages, settings |

## Conventions

- Timestamps are named `*_time` (`create_time`, `import_time`) and stored as ISO 8601 with the
  UTC offset (`2026-10-06T14:26:00+07:00`).
- Items are identified only by the platform `SKU ID`. Product name, variation and Seller SKU are
  display values that can change between two exports of the same day.
- Orders are identified by `Order ID`.
- Requirement IDs (`P3`, `A2`) are stable references; code comments and tests may cite them.
