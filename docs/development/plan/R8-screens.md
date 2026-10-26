# R8: the screens on v0.2

Started 2026-10-26, after R7 (the search index, reprint and restore in the back end).
[code-rules.md](../code-rules.md) is binding for every step and every subagent; each step's report states
*size gate passed, no new lint suppression, coverage not lower*.

**Proves it:** the owner's checklist (step R8.6) on the program folder.

Read first: [ui/](../../software-design/ui/README.md) (the window frame and each screen below),
[reprint](../../software-design/app/reprint.md), [namespaces](../../software-design/app/namespaces.md),
[duplicates](../../software-design/app/duplicates.md), [rules file](../../software-design/rules-file/README.md),
[program](../../software-design/app/program.md) (the commands).

## Steps

| Step | What | Done when |
|---|---|---|
| R8.1 (done) | The panel and the title bar: the namespace list, **+ New namespace**, the dates by month with their orders, today, a damaged date, **Open folder**, hide and show (`Ctrl+B`), read again on focus; the day stepper goes; the title bar names the namespace and the date (**Search** and **Sort rules** join it with their steps) | The shell's tests open another date and namespace through the panel; a draft asks first |
| R8.2 (done) | The Day card: **Restore** on a missing or changed output, **Left out ▸** listing each copy left out, **Reprints** with **Open** | The app's test restores through the command; the front-end tests show the rows |
| R8.3 (done) | Search on the index: the boxes as filter text, *This day* / *All days*, *Loading…*, the rows with their tick and trace line, **Select all results**, **Reprint `<n>` orders** and its line with **Open** | A front-end test searches, ticks and reprints against the fake back end |
| R8.4 (done) | Repeated orders: the read's repeats as groups and files, `resolve_repeats`, the screen after the logo prompt, the Plan header's *copies left out ▸* and **Change…**, the group chips on the file list | The app's test chooses *Keep the last copies* on the repeat fixture and saves; the verifier passes |
| R8.5 (done) | Sort rules on `packing-workflows.toml`: the label sort rules edited through `toml_edit` (comments kept, an unchanged filter keeps its text), `rules_*`, `rule_new` / `rule_rename` / `rule_delete`, the screen with the boxes, the text view and *Edit file as text*; the v0.1 words (*grouping*, *category*, *set*) leave the front end and the app (the v0.1 engine modules go with R6.4) | A rename and a new rule written into the file keep its comments; the Plan's **Edit sort rule** rebuilds the plan |
| R8.6 | The user guide for v0.2 and its pictures (made with `testdata/` labels); the owner's checklist, from the regression list of the phase 2 plan | Every screen of the guide matches the running app |

## Notes for the steps

- The window's TypeScript types mirror the Rust view types by hand and change in the same commit.
- Search and the Sort rules screen share the boxes editor; its tree goes through the engine's parser and
  printer (`filter_to_tree` / `tree_to_filter`), so the window has no second parser.
- The pictures: run the app (`npm run tauri dev`) on a fresh program folder with the `testdata/` label
  PDFs, so no real data shows.
