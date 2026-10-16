# Reviews

The owner's feedback on the running app, and the owner's ideas. One file per scope, never one per review
round:

| File | Scope |
|---|---|
| `label.md` | LABEL: reading labels and slips, couriers, picks, the packing list |
| `rules.md` | RULES: `categories.toml` and the condition language |
| `app.md` | APP: batches, the data folder, revert and amend, the program folder |
| `ui.md` | UI: the window and every screen |

A file exists only while it holds an open item. An item is `R-<SCOPE>-n` (for example `R-UI-8`), written
as the present state and the target: what the owner sees, what the owner wants. A resolved item is deleted;
its answer is written into [../software-design/](../software-design/README.md) and the work into
[../development/plan/](../development/plan/README.md). Numbers are never reused.

Open items: [app.md](app.md) (R-APP-1), [ui.md](ui.md) (R-UI-1, R-UI-2).
