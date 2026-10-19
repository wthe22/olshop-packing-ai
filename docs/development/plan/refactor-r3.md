# R3 — The UI: split the big screens and `api.ts`

Read first: [../code-rules.md](../code-rules.md) (binding), [refactor.md](refactor.md) (brief, choices).
Effort L. Edits only `app/src/` (and `app/tests/` where an import path moves). Every screen looks and acts as
before: same texts (`app/src/texts.ts`), same buttons, same order of steps, same commands. The safety net is
`npm run check` (types: props must match) and `npm test`; there is no rendered test, so move code, do not
rewrite it. Types come from R2 (`Grouping`, the plan row kind); do not redo them.

1. `app/src/lib/PlanScreen.svelte` (804): one component per job: the plan rows with their expanded runs, the
   *Add a category for this batch* form, the packing-list options, the save with its progress and findings
   dialog; the screen keeps the state they share and the commands. Remove the four
   `svelte-ignore state_referenced_locally` suppressions by deriving the initial values in a way Svelte accepts
   (not by adding another suppression).
2. `app/src/lib/CategoriesScreen.svelte` (732): the grouping bar (choose, new, delete), the category list
   and its row editor, the live counts, the text view; the screen keeps the state they share.
3. `app/src/lib/NewBatchScreen.svelte` (509): the file list and the read progress apart from the grouping
   control and the logo prompt hand-off.
4. `app/src/routes/+page.svelte` (467; script 385): separate the screen routing (which screen shows) from
   the data it loads and the dialogs; plain `.ts` modules for logic, components for markup.
5. `app/src/lib/api.ts` (445) → `app/src/lib/api/` one file per area (start and settings, day, new batch and
   plan, categories and groupings, search), each a typed wrapper over `invoke` with its interfaces beside it.
   Call sites import the area they use; no barrel file unless three or more call sites need several areas.
6. `app/src/lib/texts.ts` (458) stays one file while under 500 lines; split by screen if it passes it.
7. `app/src/lib/Frame.svelte`, `SettingsScreen.svelte`, `DayScreen.svelte` (345) and the rest: no change
   unless a file has two jobs.
8. No file under `app/src` over 500 lines. Lower the size baseline (`node tools/check-size.mjs --update`).

Done when: `bash tools/check.sh --coverage` is green (svelte-check 0 errors, 0 warnings; the 53 front-end tests
pass), the lint baseline's `svelte-ignore` count is zero, and the report lists the new components with their
one-line jobs.
