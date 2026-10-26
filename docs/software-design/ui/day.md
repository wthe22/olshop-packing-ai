# Day

The start screen: what has been saved on the open date of the open namespace, and the way to the next batch.

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│ Packing v0.2  ·  [☰]  Toko Contoh · Wed 7 Oct 2026     [Search]  [Sort rules] [Settings]│
├────────────────────────────────────────────────────────────────────────────────────────┤
│  This day: 2 batches · 729 orders · 5 group PDFs · 1 reprint             [ + New batch ]│
│                                                                                        │
│  ┌ Batch 2 · saved 09:12 · 2 files · 129 orders · rule default ── [Amend] [Revert] ──┐ │
│  │  2-A  Sepatu           88 orders   3 order contents   [✓]  [Open]                 │ │
│  │  2-Z  Lainnya          41 orders  12 order contents   [ ]  [Open]                 │ │
│  │  Summary [Open]   Saved with 2 errors ▸   Warnings: 1 ▸        [Open folder]      │ │
│  └───────────────────────────────────────────────────────────────────────────────────┘ │
│  ┌ Batch 1 · saved 07:40 · 3 files · 600 orders · rule default ──────────────────────┐ │
│  │  1-A  Sepatu          172 orders   4 order contents   [ ]  [Open]                 │ │
│  │  1-B  Spion & Knalpot  35 orders   4 order contents   [ ]  ⚠ missing  [Restore]   │ │
│  │  1-Z  Lainnya         393 orders  38 order contents   [ ]  [Open]                 │ │
│  │  Summary [Open]   Warnings: none   Left out: 3 ▸                [Open folder]     │ │
│  └───────────────────────────────────────────────────────────────────────────────────┘ │
│  Reprints: reprint 1 x3 (09:40) [Open]                                                 │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

- Newest batch on top. Each group PDF: `<batch>-<code>`, the group's name, orders, order contents.
- **The printed tick** beside each group PDF, as large as **Open**, says it has been printed. It carries no
  word; its tooltip and screen-reader name say *Printed* and name the file. A click is saved at once as a
  journal record ([05 After the save](../workflow/05-after-save.md#16-printed)), so it survives a restart. It
  is a note only: nothing is enforced or blocked. A batch written again by an **Amend** starts unticked.
- **Open** opens the group PDF, or the batch's summary PDF, in the PC's default PDF viewer, where the owner
  prints it. **Open folder** opens the date's `output\` folder in Explorer.
- **Amend** and **Revert** appear only on the newest batch ([05 After the save](../workflow/05-after-save.md)):
  - Revert asks: "Undo batch 2? Its 2 group PDFs and its summary are deleted, and its 129 orders can be
    saved again. If you already printed them, throw the printed labels away." [Cancel] [Undo batch 2]
  - Amend asks: "Redo batch 2 with the sort rule default? Batch 2 was saved at 09:12. If you already
    printed it, print it again after saving." [Cancel] [Redo batch 2] reads the batch's inputs again, with a
    progress bar, then opens the Plan screen titled *Redo batch 2* ([Plan](plan.md)).
- **⚠ missing** / **⚠ changed** replaces **Open** on an output the open-time check found missing or with
  another size; **Restore** writes it again ([reprint › Restore](../app/reprint.md#restore)).
- **Warnings ▸** expands the batch's warnings (the same texts as on the Plan screen).
- **Left out: `<n>` ▸** appears when copies were left out: each order, its input and pages, the reason
  (*repeat in this batch* or *saved earlier*) and the copy that stays.
- **Saved with `<n>` errors ▸** appears on a batch saved anyway and lists the verifier's errors; its
  warnings are under **Warnings ▸**. They stay until the batch is reverted or amended.
- **Reprints** lists the reprints made on this date, with **Open** ([Search](search.md)).
- An input that is missing or changed is a line above the batches, naming the file and the batches that
  use it ([Messages](messages.md)).
- A data folder that also holds v0.1 data shows one line: *This folder also holds data of Packing v0.1
  (`labels\`). It is not read.* [Open folder] ([namespaces](../app/namespaces.md#a-version-01-data-folder)).
- A date with nothing saved shows "Nothing saved on this day yet." and **New batch**.
- After **Save batch**, the new batch is on top with a green line "Batch 3 saved: 3 group PDFs and the
  summary." for a few seconds.
