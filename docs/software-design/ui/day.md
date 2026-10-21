# Day

The start screen: what has been saved in the open session, and the way to the next batch. A day with
more than one session shows one session at a time; the [Sessions panel](sessions.md) switches.

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│ Packing v0.1  ·  [☰]  Wed 7 Oct 2026 · Main         [Search]  [Categories]  [Settings] │
├────────────────────────────────────────────────────────────────────────────────────────┤
│  This session: 2 batches · 729 orders · 5 saved PDFs                [ + New batch ]    │
│                                                                                        │
│  ┌ Batch 2 · saved 09:12 · 2 files · 129 orders ─────────── [Amend] [Revert] ───────┐  │
│  │  4  A  Sepatu              88 orders   3 runs     [✓]  [Open]                    │  │
│  │  5  Z  Lainnya             41 orders  12 runs     [ ]  [Open]                    │  │
│  │  Packing list [Open]   Saved with 2 errors ▸   Warnings: 1 ▸  [Open folder]      │  │
│  └──────────────────────────────────────────────────────────────────────────────────┘  │
│  ┌ Batch 1 · saved 07:40 · 3 files · 600 orders ────────────────────────────────────┐  │
│  │  1  A  Sepatu             172 orders   4 runs     [ ]  [Open]                    │  │
│  │  2  B  Spion & Knalpot     35 orders   4 runs     [ ]  [Open]                    │  │
│  │  3  Z  Lainnya            393 orders  38 runs     [ ]  [Open]                    │  │
│  │  Packing list [Open]   Warnings: none   Left out: 3 ▸       [Open folder]        │  │
│  └──────────────────────────────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

- Newest batch on top. Each saved PDF: number, pick code and name, orders, runs.
- The tick beside each saved-PDF row, next to **Open** and as large as it, is a note: the owner ticks
  it to say that this saved PDF has been printed. It carries no word — the tick alone says it — and
  its tooltip and screen-reader name say *Printed* and name the saved PDF. It is a note only —
  nothing is enforced, blocked or required, and an unticked row works as it does today. A tick is
  written into the session's `state.json` the moment it is clicked ([data-folder](../app/data-folder.md)),
  so it is still there after the app is closed and opened again. A batch written again by an **Amend**
  starts with its rows unticked, because the amend writes the batch afresh.
- **Open** opens the saved PDF, or the batch's packing list, in the PC's default PDF viewer,
  where the owner prints it.
  **Open folder** opens the batch folder in Explorer.
- **Amend** and **Revert** appear only on the newest batch ([revert and amend](../app/batch.md)):
  - Revert asks: "Undo batch 2? Its 2 saved PDFs, packing list and download copies are deleted, and its 129
    orders can be saved again. If you already printed them, throw the printed labels away."
    [Cancel] [Undo batch 2]
  - Amend asks: "Redo batch 2 with its grouping (default)? Batch 2 was saved at 09:12. If you
    already printed it, print it again after saving." [Cancel] [Redo batch 2] reads the batch's own
    label PDFs again with the grouping the batch used, recorded in its `state.json`
    ([New batch › The grouping](new-batch.md#the-grouping)); the window shows "Reading the batch's labels …  page 143 of
    257" with a progress bar, then opens the Plan screen, titled *Redo batch 2* ([Plan](plan.md)).
    That screen's **Back** shows the amend's selection page with the current selection listed
    ([New batch › The amend's selection page](new-batch.md#the-amends-selection-page)); **Read
    labels** there opens the Plan screen again.
- **Warnings ▸** expands the batch's warnings (same texts as on the Plan screen).
- **Left out: `<n>` ▸** appears when copies were left out and expands them: each order, its file and
  page, the reason (*repeat in this batch* or *saved earlier*) and the copy that stays
  ([duplicates](../app/duplicates.md#what-is-stored)).
- **Saved with `<n>` errors ▸** appears on a batch saved anyway with the check's errors
  ([saved-pdf-check.md](../app/saved-pdf-check.md)) and expands them; **Warnings ▸** beside it
  expands the warnings. The two lists stay apart and stay on the card until the batch is reverted
  or amended.
- A session with no batches shows only "Nothing saved in this session yet." and **New batch**.
- After **Save batch**, the Day screen shows the new batch on top with a green line
  "Batch 3 saved: 3 PDFs, 1 packing list." for a few seconds.
