# Plan

Only the unfinished work. A finished task is deleted from this folder in the commit that finishes it; a
finished phase moves to `archive/`. Phase and task numbers live only here. The code rules apply to every
task: a task report states *size gate passed, no new lint suppression, coverage not lower*.

## Status (2026-11-11)

- **Next work: v0.5, accounts replace workspaces — written, not designed or built.** The owner's words of
  2026-11-11 are in [v0.5-accounts.md](v0.5-accounts.md), the first unfinished task. The next agent designs
  it first (V5.1), then builds it in the order that file gives. The owner's open choices are Q-APP-12 to
  Q-APP-17, Q-UI-9, Q-UI-10 and Q-RULES-3 ([software-design/questions/](../../software-design/questions/README.md)),
  each with the default the work uses until the owner answers. Answered 2026-11-11: Q-APP-12, Q-APP-14 and
  Q-APP-16 and Q-APP-15 (A), and the owner's other answers are in the plan's *owner's answers* section. R6.4 and R9 stay open and are placed in that
  file's order. `target/portable/Packing/` is still the 0.4.2 build. Files left uncommitted by another
  session (`docs/development/README.md`, `docs/development/app.md`, `tools/dev.bat`) are not part of this.
- **Handoff of 2026-11-09 — the panel marks a workspace with unprinted groups today; version 0.4.2.** The
  owner's words in chat, built and pushed:
  - The workspace list of the panel shows **`· not printed yet`** after a workspace whose live batches of
    today have a group PDF not marked **Printed** ([panel](../../software-design/ui/panel.md)). The mark is
    read from each workspace's journal when the panel is read, so it clears on the next read after a tick.
  - Not in this change: the dates list of the open workspace has no mark, and a summary PDF does not count.
  - `target/portable/Packing/` is rebuilt. **The owner may test now**: tick Printed on every group PDF of
    today in one workspace, then open the workspace list; the mark goes when no group is left. A finding goes
    into [reviews/ui.md](../../reviews/ui.md) as the next `R-UI-<n>`.

- **Handoff of 2026-11-06 — a workspace's shops; Print in the app; version 0.4.0.** The owner's words in
  chat, built and pushed:
  - **D-APP-25**: a workspace lists its shops (Settings › Workspaces › **Shops**, kept in
    `{workspace}/settings.toml`). A read whose labels show another sender, or none, asks first:
    **Back**, or **Read anyway**. Names are compared whole, spaces and case ignored, so the Shopee
    spelling is listed beside the TikTok one
    ([workspaces › Its shops](../../software-design/app/workspaces.md#its-shops)). A workspace with no shop
    listed asks nothing.
  - **D-APP-26**: **Print** on a group PDF, a summary PDF or a reprint opens the print preview Chrome shows
    over the app's window. A group PDF is then marked *Sent hh:mm*; its ✕ takes the mark back when the
    print was cancelled or failed. The **Printed** tick is the second check and can be ticked with or
    without *Sent* ([Day](../../software-design/ui/day.md)). The preview does not tell the app whether
    Print or Cancel was pressed.
  - **Update every copy together**: the journal now holds `print_sent` records, and an older copy of the
    program reads such a date as damaged ([records](../../software-design/data/records.md)).
  - **The owner may test now** with `target/portable/Packing/Packing.exe`: list the shops of each
    workspace, then read one day's real downloads, a file of another shop among them. Print a group PDF, cancel once (✕ the mark), print again, tick
    Printed. A finding goes into [reviews/app.md](../../reviews/app.md) as R-APP-10, a screen finding into
    [reviews/ui.md](../../reviews/ui.md) as R-UI-39, a sender read wrong into
    [reviews/label.md](../../reviews/label.md) as R-LABEL-13.

- **Handoff of 2026-11-02 — several copies on one data folder; version 0.3.3.** The owner's words in
  chat, built and pushed:
  - **R-APP-9**: a second copy of the program no longer stops at *Packing is already open*. The data
    folder's lock is held only while a copy writes; another copy's write waits for it, a minute at most
    ([D-APP-9](../../software-design/decisions/app.md), [data › One writer](../../software-design/data/README.md#one-writer)).
  - Each copy looks every 3 seconds for what the others wrote: the Day screen, the panel and Search
    follow. A write reads the date's journal again first; a save whose orders another copy saved after
    the labels were read stops with *read the labels again*; **Revert** and **Amend** refuse a batch that
    is no longer the newest; a workspace another copy renamed or deleted is followed
    ([program › Several copies](../../software-design/app/program.md#several-copies)).
  - Version **0.3.3**. The commit also holds the earlier session's 0.3.2 work that was left uncommitted
    (the whole version shown, *About*'s build time, the buttons without `…`).
  - **The owner may test now**: start `target/portable/Packing/Packing.exe` twice, or on two PCs with one
    shared data folder. Save a batch in one: the other shows it within a few seconds. Tick Printed in
    both, save in both at once, rename the workspace in one. A finding goes into
    [reviews/app.md](../../reviews/app.md) as R-APP-10.
  - Not covered: `packing-workflows.toml` is beside the program, not in the data folder. Copies started
    from one program folder share it, and the last save of a sort rule wins.

- **Handoff of 2026-11-02 — Shopee orders: the tracking ID's letter, `total_quantity` on every slip.**
  The owner's words in chat, built and pushed:
  - **R-LABEL-12**: an SPX tracking ID is `SPXID` + 11 digits + a capital letter, and the letter was
    dropped. The bundled form is `SPXID\d{11}[A-Z]`; the `No. Resi` layout's `Batas Kirim` date is read
    too ([labels/courier](../../software-design/labels/courier.md)). The owner's files are in `samples/`
    as `2026-11-02 spx eco.pdf` and `2026-11-02 spx std.pdf`.
  - **R-RULES-7**: a filter's `total_quantity` is the units of the order's lines on every slip. It was
    the printed `Qty Total`, which the Shopee-style slip does not print, so `total_quantity = 2` never
    matched there and the order fell to the next sepatu group
    ([conditions](../../software-design/rules-file/conditions.md)).
  - **The owner's step first**: the program reads the owner's own `packing-workflows.toml` beside
    `Packing.exe`, never the bundled copy, and does not rewrite it (D-APP-3). In its
    `courier_code = "spx"` entry the owner changes `tracking_id = 'SPXID\d{11}'` to
    `tracking_id = 'SPXID\d{11}[A-Z]'`, saves and starts the program again. Without that line the
    tracking ID still loses its letter; the `total_quantity` fix needs no change to the file.
  - **The owner may test now**: read the two 2026-11-02 Shopee PDFs with `main-v2.0`: the sepatu
    orders of 2 go to *Sepatu Hitam x2*, and each tracking ID ends in its letter. A batch already saved
    keeps what it read until it is amended. A finding goes into [reviews/label.md](../../reviews/label.md)
    or [reviews/rules.md](../../reviews/rules.md) as R-LABEL-13 or R-RULES-8.
  - `target/portable/Packing/` is built from the working tree, which also holds another session's
    uncommitted work (as the build of 2026-11-01 did); that work stays uncommitted.

- **Handoff of 2026-10-30 — Search's dates and rows, the date as the Day screen's title.** The owner's
  words in chat, built and pushed:
  - **R-UI-37**: no *This day* anywhere. Search's Dates are *Viewed day* (the date the Day screen shows,
    the default; not listed when it is today), *Today*, *Yesterday* and the rest
    ([D-UI-3](../../software-design/decisions/ui.md)). The Day screen's title is the date itself with a
    *today* pill; the title bar names only the workspace ([ui/day](../../software-design/ui/day.md)).
  - **R-UI-38**: a Search row is *Order* (the Order ID over the courier and tracking ID), *Items*, *Label*
    (the date and group code over the group's name and page); its details show only the batch's save
    time and the input PDF, its file name boxed with a file icon, no courier clue
    ([D-UI-12](../../software-design/decisions/ui.md)).
  - **The owner may test now**: Search on both screens, a row's arrow, the Dates list on an older date
    and on today. A finding goes into [reviews/ui.md](../../reviews/ui.md) as R-UI-39.

- **Handoff of 2026-10-30 — the sort rules: the rule used last, a rule switched off, a new page.** The
  owner's words in chat, built and pushed:
  - **R-RULES-5**: New batch preselects the rule of the workspace's newest batch **on whichever date**, so
    a new day starts on the rule used last; **Back** from the Plan screen keeps the rule just read
    ([D-RULES-1](../../software-design/decisions/rules.md)). Nothing new is stored: the journals say it.
  - **R-RULES-6**: a sort rule is **On** or **Off** (`enabled = false` in `packing-workflows.toml`); New
    batch lists only the rules that are on; the only rule that is on cannot be switched off or deleted
    ([D-RULES-5](../../software-design/decisions/rules.md)).
  - **The Sort rules page** in the look of Settings and the Plan screen
    ([ui/sort-rules](../../software-design/ui/sort-rules.md)): a tab per rule (*last used*, *off*), the
    rule as one card — its head the switch, **Rename…** in place and **Delete rule**, a row per group with
    its code as the blue chip, **Boxes | Text**, the count under *In batch n* and icon buttons — and a foot
    that stays in view with *Unsaved changes*, **Cancel** and **Save**; another tab asks before it drops
    unsaved changes. The doubled *no filter: takes the rest* is one tick.
  - **The owner may test now**: [R8-checklist.md](R8-checklist.md) item 10. A finding goes into
    [reviews/rules.md](../../reviews/rules.md) or [reviews/ui.md](../../reviews/ui.md) as the next
    `R-<SCOPE>-n` (R-RULES-7, R-UI-36).

- **Handoff of 2026-10-29 — the owner tests round eight; then the work is R6.4 and R9.** Round eight of
  the owner's review, from the owner's words in chat, is built and pushed:
  - **Search** shows one compact **row** per order again: the Order ID over the tracking ID; the items,
    with the rest of the row; the group PDF's code, name and **page** over the courier and the date;
    **Open**, the trace arrow and the *Reprinted* mark ([D-UI-12](../../software-design/decisions/ui.md)).
    The ID and group PDF columns are only as wide as their longest entry. In a window much less than full
    width the items take a line of their own under the IDs and the dates and **Search** a line under the
    filter; the note of a reprint just written sits in the reprint bar's line.
  - **The day summary** lists the day's order contents with the most orders first (the pick pages keep
    their order) and, after the couriers, each batch's input PDFs with their pages and SHA-256 and the
    batch's ledger, as a batch's summary PDF does
    ([D-LABEL-23](../../software-design/decisions/summaries.md)).
  - In a window under 56 rem the title bar's **Search**, **Sort rules** and **Settings** keep their icons
    only, so the bar no longer wraps ([ui/README](../../software-design/ui/README.md#window-frame)).
  - Round seven's look ([D-UI-11](../../software-design/decisions/ui.md)) and *Tracking ID contains* stay.
  - **Found while checking, not fixed**: the PDF viewer of WebView2 (Edge's) draws the `·` of the summary
    PDFs and the day summary as `¥`; the text layer reads `·`. `·` is drawn with the non-CID font
    (`packing_list/fonts.rs`), and the verifier reads the header's `·` line as text, so a fix touches both.
  - Checked on screenshots of the built program at 1536 × 800, 1093 × 580 (the owner's two screens at
    125 %) and at 960, 768 and 600 points wide, in both themes, with long made-up product names.
  - `target/portable/Packing/` is rebuilt. **The owner may test now**: Search on the small screen and in a
    window less than full width (a long product name, ticking rows, **Select all**, **Reprint**), and
    **Day summary**'s first and last pages; then [R8-checklist.md](R8-checklist.md) items 9 and 11 if not
    tried.
  - No question is open.
  - A finding goes into [reviews/ui.md](../../reviews/ui.md), [reviews/label.md](../../reviews/label.md),
    [reviews/rules.md](../../reviews/rules.md) or [reviews/app.md](../../reviews/app.md) as the next
    `R-<SCOPE>-n` (R-UI-36, R-LABEL-12, R-RULES-7, R-APP-9); the next session turns each into a task.
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
  R6.1 to R6.3 are done, R6.4 follows the owner's test of v0.2. **The program the owner runs is v0.3.3**: it
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
