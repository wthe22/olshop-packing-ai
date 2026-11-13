# Day

The start screen: what has been saved on the open date, for every account, and the way to the next batch.

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│ [☰] Packing v0.6                             [Search] [Accounts] [Sort rules] [Settings]│
├────────────────────────────────────────────────────────────────────────────────────────┤
│  Wed, 7 Oct 2026 (today) (2 batches) (729 orders) (5 group PDFs)  [Day summary] [+ New batch]│
│                                                                                        │
│  ┌ Batch 2  saved 09:12 · 2 files · 129 orders · 2 accounts ───────── [Amend] [Revert] │
│  │  2-A  Sepatu            88 orders    3 contents               [✓] [Print] [Open]  │ │
│  │  2-Z  Lainnya           41 orders   12 contents  Sent 09:14 ✕ [ ] [Print] [Open]  │ │
│  │  [Summary PDF ▾]                          Saved with 2 errors ▸   Warnings: 1 ▸    │ │
│  └───────────────────────────────────────────────────────────────────────────────────┘ │
│  ┌ Batch 1  saved 07:40 · 3 files · 600 orders · 1 account ──────────────────────────┐ │
│  │  1-A  Sepatu           172 orders    4 contents               [ ] [Print] [Open]  │ │
│  │  1-B  Spion & Knalpot   35 orders    4 contents    [ ]  ⚠ missing [Print] [Open]  │ │
│  │  1-Z  Lainnya          393 orders   38 contents               [ ] [Print] [Open]  │ │
│  │  Summary PDF [Open]       Warnings: none   Left out: 3 ▸           [Open folder]  │ │
│  └───────────────────────────────────────────────────────────────────────────────────┘ │
│  ┌ Reprints · 1 file · 3 orders ─────────────────────────────────────────────────────┐ │
│  │  R1   made 09:40     3 orders     3 pages            Orders ▾   [Print] [Open]    │ │
│  │       580000000000000042  JY0000001234  jnt  Sepatu ×1      Wed, 7 Oct · 2-A     │ │
│  │       580000000000000057  JY0000001235  jnt  Spion Beat ×2  Wed, 7 Oct · 2-B     │ │
│  │       580000000000000061  JY0000001240  jnt  Sepatu ×2      Tue, 6 Oct · 1-A     │ │
│  └───────────────────────────────────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

- **The open date** is the title, *Wed, 7 Oct 2026*, with a *today* pill when it is today; the title bar
  does not repeat it (R-UI-37). Its counts stand beside it as pills (no reprint, no pill for them).
- Newest batch on top, each a card whose shaded head is *Batch 2* in bold and its save time, files and
  orders, then how many accounts its orders are of (*2 accounts*). Each group PDF: `<batch>-<code>`, the group's name, its orders and their order
  contents (*88 orders*, *3 contents*); the code is a chip in the logo's blue. Each count is a column of its own, its number right-aligned in a
  fixed width and its word after it in the muted colour, so the numbers of a card stand under each other.
- **Print** opens the print preview of a group PDF or a reprint over the app's window,
  the one Chrome shows: printer, pages, copies, paper size, scale, margins, two-sided
  ([D-APP-26](../decisions/app-features.md)).
- **Sent hh:mm** stands before the tick of a group PDF from the moment its **Print** opens the preview: the
  app remembers that the print was sent. The preview does not tell the app whether Print or Cancel was
  pressed, so its ✕ takes the mark back, without a question, when the print was cancelled or failed.
- **The printed tick** beside each group PDF, as large as **Open**, is the second check: the labels came out
  and the group is done. It can be ticked with or without *Sent* (a small amend whose labels were all
  printed already is ticked at once); while ticked, *Sent* is not shown. It carries no word; its tooltip and
  screen-reader name say *Printed* and name the file.
- Both marks are saved at once as journal records
  ([05 After the save](../workflow/05-after-save.md#16-print-sent-and-printed)), so they survive a restart.
  They are notes only: nothing is enforced or blocked. A batch written again by an **Amend** starts with
  neither.
- **Open** opens the group PDF, or the batch's summary PDF of all accounts, in the PC's default PDF viewer;
  a print from there marks nothing. A summary PDF has no **Print** ([D-APP-29](../decisions/accounts.md)). Before **Print** or **Open**, a file that is not there any more (deleted in Explorer)
  is written again, checked like a **Restore**, with the dialog *Writing `<file>` again*; a summary PDF
  written again has the program's current layout. **Open folder** opens the date's `output\` folder in
  Explorer.
- **Summary PDF** is a menu, like **Day summary**, when the batch holds more than one account: *All
  accounts* opens the batch's summary PDF; an account writes that account's summary of the batch and opens it
  ([summary PDF](../labels/summary-pdf.md#one-account)), with the dialog *Writing the summary of Toko Alfa*; a
  finding of its check is shown and nothing is kept. A summary PDF found changed shows **Restore** in place
  of the menu.
- **Amend** and **Revert** appear only on the newest batch ([05 After the save](../workflow/05-after-save.md)):
  - Revert asks: "Undo batch 2?" — "Its 2 group PDFs and summary PDF are deleted. Throw away its printed
    labels." [Undo batch 2] [Cancel]
  - Amend asks: "Redo batch 2?" — "Its labels are read again with the sort rule default." [Redo batch 2]
    [Cancel], then reads the batch's inputs again, with a progress bar, and opens the Plan screen titled
    *Redo batch 2* ([Plan](plan.md)).
- **⚠ missing** / **⚠ changed** stands before the button of an output the open-time check found missing or
  with another size. A missing one keeps **Open**, which writes it again; a changed one has **Restore**
  instead, which writes it again over the changed file ([reprint › Restore](../app/reprint.md#restore)).
- **Warnings ▸** expands the batch's warnings (the same texts as on the Plan screen).
- **Left out: `<n>` ▸** appears when copies were left out: each order, its input and pages, the reason
  (*repeat in this batch* or *saved earlier*) and the copy that stays.
- **Saved with `<n>` errors ▸** appears on a batch saved anyway and lists the verifier's errors; its
  warnings are under **Warnings ▸**. They stay until the batch is reverted or amended.
- **Reprints**, a card under the batches when the date has any: one row per reprint like a group PDF's —
  `R<n>`, the time it was made, its orders and pages as count columns, **Print**, **Open** — and **Orders ▸**, which lists its
  orders as Search shows them (Order ID, tracking ID, courier, items, the date and group PDF it was saved
  in). An order whose batch was reverted since shows its Order ID and *saved `<date>` in batch `<b>`*.
  Reprints are made on [Search](search.md).
- **Day summary** writes the [day summary](../labels/day-summary.md) of the open date and opens it. When
  the date's live batches hold more than one account it is a menu: *All accounts*, then each account; while it
  is written the dialog *Writing the day summary* shows and the button is off, so it is written once. It is
  off while the date has no batch.
- An input that is missing or changed is a line above the batches, naming the file and the batches that
  use it ([Messages](messages.md)).
- A data folder that also holds data of an older version shows one line: *This folder also holds data of
  Packing v0.4 (`Toko Contoh\`, …). It is not read.* [Open folder]
  ([accounts](../app/accounts.md#data-of-older-versions)).
- A date with nothing saved shows "Nothing saved on this date yet." and **New batch**.
- After **Save batch**, the new batch is on top with a green line "Batch 3 saved: 3 group PDFs and the
  summary." for a few seconds.
