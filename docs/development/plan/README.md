# Plan

Only the unfinished work. A finished task is deleted from this folder in the commit that finishes it; a
finished phase moves to `archive/`. Phase and task numbers live only here. The code rules apply to every
task: a task report states *size gate passed, no new lint suppression, coverage not lower*.

## Status (2026-10-27)

- **Handoff of the session of 2026-10-27 — the owner tests the summaries, the Day screen, Grab and
  Anteraja, and an amend that adds a file; then the work is R6.4 and R9.** Round four of the owner's
  review, from the owner's words in chat, is built and pushed: the summary PDF's first page draws the rows
  of the group pages, names the couriers, and lists no groups and no instant orders
  ([D-LABEL-19](../../software-design/decisions/label.md); the verifier reads each group's counts from its
  bar); the day summary draws each batch as its Day card with *contents* and no Printed ticks
  ([D-LABEL-20](../../software-design/decisions/label.md)); the Day screen's counts stand in columns; an
  item's `×` no longer covers the space before it when the same item is printed twice; numbers right-
  aligned by their advance, so a `1` lines up. Grab is named by its tag `Sameday`, Anteraja by its logo and
  `TSA-` + 11 digits, and an instant label's channel strip is never asked about
  ([D-LABEL-21](../../software-design/decisions/label.md)). An amend's selection page no longer marks the
  batch's own files *used in batch n* and lists a file added again by its bytes once.
  `target/portable/Packing/` is rebuilt.
  - **The owner's `packing-workflows.toml`** there was corrected: the logo the owner had named Grab
    (`f2cf7019646c955b`) is the `TikTok Shop` strip on every TikTok instant label, so it named no Gojek or
    Grab label; it is now under `not_courier` with `tokopedia PLUS` (`330b79a3b29d4646`), Grab has
    `instant_tags = ["Sameday"]` and Anteraja `tracking_id = 'TSA-\d{11}'`. The file before is kept as
    `work/packing-workflows.before-2026-10-27.toml`.
  - **The owner may test now**: a batch's summary PDF (first page, group pages), **Day summary**, the Day
    screen's columns; a Grab instant label and an Anteraja label read with their courier; **Amend** with
    a label file of that batch added again.
  - No question is open.
  - A finding goes into [reviews/ui.md](../../reviews/ui.md), [reviews/label.md](../../reviews/label.md),
    [reviews/rules.md](../../reviews/rules.md) or [reviews/app.md](../../reviews/app.md) as the next
    `R-<SCOPE>-n` (R-UI-21, R-LABEL-9, R-RULES-4, R-APP-7); the next session turns each into a task.
  - **Next without the owner:** **R6.4** ([R6-app-journal.md](R6-app-journal.md)), then **R9** (the full
    check, an independent review of the whole v0.2 diff, the coverage baseline, the handoff). An instant
    label's unknown tag is not asked about yet (W-9 of [workflow-audit.md](workflow-audit.md)).
  - The guide's pictures are the real program on the `testdata/` labels (`node tools/guide-pictures.mjs`
    drives a fresh copy over WebView2's DevTools port); the same run is a quick smoke test of the build.

- **The work is the v0.2 rework**: [v0.2-rework.md](v0.2-rework.md) holds the decisions agreed with the owner
  and the order of work. **R1 to R5 are done**: [software-design/](../../software-design/README.md)
  describes v0.2; the project has no Python; the label fixtures are written by `crates/fixtures` one
  character per text object, as the downloads are; today's reading of the owner's eleven sample files is
  pinned in `work/baseline/` on this PC ([testing.md](../testing.md), *Local baseline*); `crates/store`
  holds the journal, the lock, the inputs, the views, the open-time checks and the workspaces; and
  `packing-engine` holds the v0.2 chain — `packing-workflows.toml`, copies with the courier clue,
  repeats and the copies left out, the plan with order contents, the `batch_saved` record — and the 2.32
  fix; `crates/verify` checks a batch's outputs by page digest with `lopdf` alone (every page of the
  samples that PDFium copies keeps its digest). **R6 is under way** ([R6-app-journal.md](R6-app-journal.md)):
  R6.1 to R6.3 are done, R6.4 follows the owner's test of v0.2. **The program the owner runs is v0.2.0**: it
  saves through `crates/batch` and the journal into `{data_dir}/{workspace}/{date}/` and writes the summary
  PDF. A task gets its own file in this folder when it starts; until then its row in the order of work is
  the task.
- **The owner checked the v0.1 program** (2026-10-26): it works as before, including the full product name
  read without a space inside a word; the owner said to go ahead with R6. R6 is the first task that changes
  the program the owner runs: the app saves through the journal into `{data_dir}/{workspace}/{date}/` and no
  longer shows the v0.1 data (`labels\`, `state.json`), which stays on disk unread
  ([D-APP-17](../../software-design/decisions/app-features.md)).
- **Not blocked on the owner otherwise.** The owner may read the design at any time, starting with
  [the workflow](../../software-design/workflow/README.md) (each step against one page) and the points below;
  a finding goes into [reviews/app.md](../../reviews/app.md) under R-APP-2 and changes the design before the
  task that builds it.
- **The owner confirmed** (2026-10-25) the points R1 had to settle: the ship-by deadline is a date only
  (D-APP-21); orders are grouped by name, variation and quantity, the Seller SKU kept only for reference —
  names and SKUs can change during the day, and the order inside a group matters less than the grouping
  (D-APP-20); a `courier` filter compares the code; an amend uses the rule of the batch's name, else its
  snapshot (D-RULES-2); a continuation page must follow its label page (D-LABEL-12).
- **The owner chose *order content*** (2026-10-25) for the orders with the same items inside a group PDF
  ([D-LABEL-15](../../software-design/decisions/label.md)): the Seller SKU and the printed `Qty Total` moved
  from the content to the order, and the block number (`2-A-05`) is gone. The owner judges on the running
  screens (R8) whether the word fits; a finding goes into [reviews/ui.md](../../reviews/ui.md).
- **Kept for the rework**: [workflow-audit.md](workflow-audit.md), where the built program differs from the
  design; the v0.2 crates close its rows, and the program follows when R5 and R6 put them to use.
- The program folder `target/portable/Packing/` is rebuilt after each task; a rebuild replaces only
  `Packing.exe` and `pdfium.dll`, so the owner's settings, rules and data there stay.
  The owner's checklist for v0.2 is [R8-checklist.md](R8-checklist.md).

## Where to work

A **local** session on the owner's PC (Windows, Git Bash): it builds, runs `tools/make-portable.sh` and
commits. A cloud session cannot run commands on the PC; it can only read and write files there.

Outside Windows the packing-list tests fail, because the packing list loads its fonts from
`C:\Windows\Fonts` (ten tests of `packing-app`); that is not a regression.

## Reading for a new session

In this order.

1. [../../../AGENTS.md](../../../AGENTS.md) and [../README.md](../README.md): rules, commit and push, layout.
2. [v0.2-rework.md](v0.2-rework.md), then the task's own file when it has one.
3. [../../software-design/workflow/README.md](../../software-design/workflow/README.md) (the steps, the
   invariants I1–I12 and the verification matrix), then [data/](../../software-design/data/README.md) and the
   design files the task names.
4. [../code-rules.md](../code-rules.md) and [../testing.md](../testing.md).
5. [../../business-process/02-data-sources.md](../../business-process/02-data-sources.md) before touching
   any PDF code.
6. Code, after the documents: `crates/` and `app/`; then `git status` and `git log --oneline -5`.

`samples/` and `work/` are confidential: never commit them or copy real data from them.
