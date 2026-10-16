# Day

The start screen: what has been saved for the day, and the way to the next batch.

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│ Packing  ·  ◀  Wed 7 Oct 2026  ▶  [Today]           [Search]  [Categories]  [Settings] │
├────────────────────────────────────────────────────────────────────────────────────────┤
│  Today: 2 batches · 729 orders · 5 saved PDFs                       [ + New batch ]    │
│                                                                                        │
│  ┌ Batch 2 · saved 09:12 · 2 files · 129 orders ─────────── [Amend] [Revert] ───────┐  │
│  │  4  A  Sepatu              88 orders   3 runs     [Open]                         │  │
│  │  5  Z  Lainnya             41 orders  12 runs     [Open]                         │  │
│  │  Packing list [Open]   Saved with 2 errors ▸   Warnings: 1 ▸  [Open folder]      │  │
│  └──────────────────────────────────────────────────────────────────────────────────┘  │
│  ┌ Batch 1 · saved 07:40 · 3 files · 600 orders ────────────────────────────────────┐  │
│  │  1  A  Sepatu             172 orders   4 runs     [Open]                         │  │
│  │  2  B  Spion & Knalpot     35 orders   4 runs     [Open]                         │  │
│  │  3  Z  Lainnya            393 orders  38 runs     [Open]                         │  │
│  │  Packing list [Open]   Warnings: none                       [Open folder]        │  │
│  └──────────────────────────────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

- Newest batch on top. Each saved PDF: number, pick code and name, orders, runs.
- **Open** opens the saved PDF, or the batch's packing list, in the PC's default PDF viewer,
  where the owner prints it.
  **Open folder** opens the batch folder in Explorer.
- **Amend** and **Revert** appear only on the newest batch ([revert and amend](../app/batch.md)):
  - Revert asks: "Undo batch 2? Its 2 saved PDFs, packing list and download copies are deleted, and its 129
    orders can be saved again. If you already printed them, throw the printed labels away."
    [Cancel] [Undo batch 2]
  - Amend asks: "Redo batch 2 with the current categories? Batch 2 was saved at 09:12. If you
    already printed it, print it again after saving." [Cancel] [Redo batch 2] → Plan screen.
- **Warnings ▸** expands the batch's warnings (same texts as on the Plan screen).
- **Saved with `<n>` errors ▸** appears on a batch saved anyway with the check's errors
  ([saved-pdf-check.md](../app/saved-pdf-check.md)) and expands them; **Warnings ▸** beside it
  expands the warnings. The two lists stay apart and stay on the card until the batch is reverted
  or amended.
- A day with no batches shows only "Nothing saved on this day yet." and **New batch**.
- After **Save batch**, the Day screen shows the new batch on top with a green line
  "Batch 3 saved: 3 PDFs, 1 packing list." for a few seconds.
