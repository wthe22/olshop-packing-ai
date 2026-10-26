# R10: the owner's review of v0.2

The owner tested v0.2 on 2026-10-26 and asked for the changes below. [code-rules.md](../code-rules.md) is
binding for every step and every subagent; each step's report states *size gate passed, no new lint
suppression, coverage not lower*. Test with the owner's rules: copy
`target/portable/Packing/packing-workflows.toml` beside the program being tested (its group codes `S1`,
`S2`, `SN` show R-UI-14). Never commit that file.

## The findings

| ID | What the owner sees | What the owner wants |
|---|---|---|
| R-UI-9 | The data-folder and first-workspace prompts are bare text and buttons on an empty window | A designed first start |
| R-UI-10 | The title bar looks rough | A clean title bar |
| R-UI-11 | Text boxes, lists, ticks and radio buttons stay white in dark mode | Every control follows the theme |
| R-UI-12 | The theme follows Windows only | A menu to choose Light, Dark or System |
| R-UI-13 | *About* is one line of plain text | A designed *About* |
| R-UI-14 | A saved batch's group code (`1-S1`) breaks over two lines on the Day card | The code on one line |
| R-UI-15 | — | Better UI and UX wherever possible |
| R-UI-16 | A group PDF's counts read *88 orders 3 order contents*: too long | *88 orders \| 3 contents* |
| R-UI-17 | On Search, *Select all results* is at the right; each row's tick is at the left | The select-all tick at the left, above the rows' ticks |
| R-UI-18 | Reprints look like last-minute work: one line at the foot of the Day screen, a button under the Search list | A designed reprint, on Search and on the Day screen |
| R-UI-19 | The program has Tauri's default icon | An icon of its own |
| R-APP-3 | The word *namespace* | *Workspace* on screen, in code and in the docs (confirmed again) |
| R-APP-4 | A first start offers the data folder `data` | `packing-data` |
| R-RULES-3 | No filter takes the orders whose courier was not named, or that have no tracking ID | A filter for an unknown courier and a missing tracking ID |
| R-LABEL-2 | The summary PDF prints the pick list first and the facts last | Facts first; then each group with its order contents (as the v0.1 packing list); then the pick list; each part on new pages |
| R-LABEL-3 | A summary exists per batch only | A summary of the whole day, made on request |

## Steps

| Step | What | Done when |
|---|---|---|
| R10.1 (done) | Theme: colour tokens in `app.css` for light and dark, `color-scheme` so the native controls follow, the theme menu in the title bar, `theme` in `packing-settings.toml` (`system` by default); the group code on one line | Every screen right in both themes; the choice survives a restart |
| R10.2 (done) | The look: title bar, panel head, first-start prompts, Settings and *About* as cards, the operator words (`starts with`, not `starts_with`) | Screenshots of each screen in both themes |
| R10.3 (done) | `is_empty` in the filter language (text fields); on screen *is unknown* for Courier and Tracking ID, *is empty* for the others, no value box | Parser, printer and evaluator tables; a sort rule groups the orders with no courier |
| R10.4 (done) | The summary PDF: facts pages, then the groups with their order contents, then the pick pages | V6 still passes; the page order pinned by a test |
| R10.5 (done) | *Namespace* → *workspace* everywhere; the journal's stored field keeps its name `namespace` (no migration) | `git grep -i namespace` finds only that field, this file, the plan's status and `docs/archive/` |
| R10.6 (done) | The default data folder `packing-data` (R-APP-4); a settings file that names another folder keeps it | The first-start test writes `data_dir = "packing-data"` |
| R10.7 | The Day card's counts *N orders \| N contents* (R-UI-16); Search's select-all tick in the head row of the tick column (R-UI-17) | The texts table; the screens |
| R10.8 | Reprints (R-UI-18): Search marks an order reprinted before and asks before printing it again; a click on a row ticks it; Enter in a box searches; the reprint bar stays in view; the Day screen's *Reprints* card lists each reprint with its orders | The index's reprint marks pinned by a test; the Search store's tests |
| R10.9 | The day summary (R-LABEL-3): **Day summary** on the Day screen writes `day summary.pdf` in the date's folder from the journal and opens it | Its content tables; a written file's page count |
| R10.10 | The app icon (R-UI-19): one SVG source, every size made by `npx tauri icon` | The program shows it in Explorer and the taskbar |
| R10.11 | The user guide's pictures again (`node tools/guide-pictures.mjs`), `make-portable.sh`, the handoff | The owner can test |
