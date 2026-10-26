# R10: the owner's review of v0.2

The owner tested v0.2 on 2026-10-26 and asked for the changes below. [code-rules.md](../code-rules.md) is
binding for every step and every subagent; each step's report states *size gate passed, no new lint
suppression, coverage not lower*. Test with the owner's rules: copy
`target/portable/Packing/packing-workflows.toml` beside the program being tested (its group codes `S1`,
`S2`, `SN` show R-UI-14). Never commit that file.

## The findings

| ID | What the owner sees | What the owner wants |
|---|---|---|
| R-UI-9 | The data-folder and first-namespace prompts are bare text and buttons on an empty window | A designed first start |
| R-UI-10 | The title bar looks rough | A clean title bar |
| R-UI-11 | Text boxes, lists, ticks and radio buttons stay white in dark mode | Every control follows the theme |
| R-UI-12 | The theme follows Windows only | A menu to choose Light, Dark or System |
| R-UI-13 | *About* is one line of plain text | A designed *About* |
| R-UI-14 | A saved batch's group code (`1-S1`) breaks over two lines on the Day card | The code on one line |
| R-UI-15 | — | Better UI and UX wherever possible |
| R-APP-3 | The word *namespace* | *Workspace* on screen, in code and in the docs |
| R-RULES-3 | No filter takes the orders whose courier was not named, or that have no tracking ID | A filter for an unknown courier and a missing tracking ID |
| R-LABEL-2 | The summary PDF prints the pick list first and the facts last | Facts first; then each group with its order contents (as the v0.1 packing list); then the pick list; each part on new pages |

## Steps

| Step | What | Done when |
|---|---|---|
| R10.1 (done) | Theme: colour tokens in `app.css` for light and dark, `color-scheme` so the native controls follow, the theme menu in the title bar, `theme` in `packing-settings.toml` (`system` by default); the group code on one line | Every screen right in both themes; the choice survives a restart |
| R10.2 (done) | The look: title bar, panel head, first-start prompts, Settings and *About* as cards, the operator words (`starts with`, not `starts_with`) | Screenshots of each screen in both themes |
| R10.3 (done) | `is_empty` in the filter language (text fields); on screen *is unknown* for Courier and Tracking ID, *is empty* for the others, no value box | Parser, printer and evaluator tables; a sort rule groups the orders with no courier |
| R10.4 (done) | The summary PDF: facts pages, then the groups with their order contents, then the pick pages | V6 still passes; the page order pinned by a test |
| R10.5 | *Namespace* → *workspace* everywhere; the journal's stored field keeps its name `namespace` (no migration) | `git grep -i namespace` finds only that field and `docs/archive/` |
| R10.6 | The user guide's pictures again (`node tools/guide-pictures.mjs`), `make-portable.sh`, the handoff | The owner can test |
