# Plan

The heart of the app: which orders go into which saved PDF. It opens with the rules file
applied in order (a proposal). For a normal day
the owner checks the counts and presses **Save batch**.

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│  Batch 3 · 2 files · 257 pages · 256 orders · 0 already saved          [Back]          │
│  Warnings: 2 ▸                                                                         │
├────────────────────────────────────────────────────────────────────────────────────────┤
│  Use  #  Pick                         Orders  Runs   File name                         │
│  [✓]  6  A  Sepatu                       130     3   [6 A Sepatu ×130            ]  ↑ ↓ ▸│
│          name contains "sepatu"                                                        │
│  [✓]  7  B  Spion & Knalpot               20     4   [7 B Spion & Knalpot ×20    ]  ↑ ↓ ▸│
│          not name contains "sepatu" and (name contains "spion" or name …)             │
│  [✓]  8  Z  Lainnya (the rest)           106    31   [8 Z Lainnya ×106           ]      ▸│
│                                                                                        │
│  [+ Add a pick for this batch]                       [Edit categories]                 │
├────────────────────────────────────────────────────────────────────────────────────────┤
│  Packing list:  ( ) one per PDF  (•) one for the batch  ( ) none                       │
│                 [ ] full   [ ] summary   [✓] pick                                      │
│                                                                    [ Save batch ]      │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

- **Rows** are the picks in order; each takes its orders from what the rows above left. The
  numbers `#` are the saved-PDF numbers this batch will get (continuing the day).
- **Use** unticked = skip this pick for this batch only (its orders fall to the rows below).
  The rules file is not changed. Unticking the rest row itself is an open question
  ([Q-UI-1](../questions/ui.md)).
- **↑ ↓** move a pick for this batch only. The rest row is always last: the last
  `categories.toml` entry when it has no condition (here *Z Lainnya*), else `? Uncategorised`.
- **+ Add a pick for this batch** opens a small form: code (default `X`), name, condition (the
  boxes editor of [Categories](categories.md), with the count it would take at that position).
  It is added above the rest row and can be moved. Not saved to the rules file; the form has a
  **Also add to categories** tick for that.
- A pick that takes 0 orders is shown grey with "0 orders — no PDF" and gets no number.
- **File name**: suggested `<n> <code> <name> ×<orders>`; the owner may change the whole name,
  number included (keeping the number in front keeps the files in printing order). The packing
  list shows the number `#` whatever the name. Two saved PDFs of the batch cannot get the same
  name (upper and lower case count as the same, as in Windows): the second shows "Same name as PDF 6" and **Save batch** waits until one is changed.
- **▸** expands a pick to its runs — what the packing list will show:

  ```
  │  6-01  Sepatu Standar Samping Motor ×1                                     96 orders │
  │  6-02  Sepatu Standar Samping Motor ×2                                     26 orders │
  │  6-03  Sepatu Standar Samping Motor ×3                                      8 orders │
  ```
- **Warnings ▸** lists the read warnings in plain words (see *Messages*), e.g. couriers not
  recognised and orders already saved for the day with their batch and time.
- Every change (tick, move, add, edit categories) recounts all rows immediately.
- **Save batch** writes the batch ([one batch](../app/batch.md)) and returns to the Day screen;
  the write and the check before saving run on their own thread
  ([saved-pdf-check.md](../app/saved-pdf-check.md)) — see *Saving*. A new batch's bottom row
  holds **Save batch** only; there is no discard button there.
- **Back** (top right) returns to the PDF selection page ([New batch](new-batch.md)) with the
  files still listed, without asking, and writes nothing. The per-batch changes (ticked, moved
  or added picks, file names) are lost when **Read labels** is pressed again: reading starts
  again from the files ([New batch › Reading](new-batch.md#reading)). Only **Cancel** on the
  selection page discards the draft.
- Amend uses the same screen, titled "Redo batch 2", with **Save batch 2 again** and a bottom
  **Cancel amend**. **Back** (top right) returns to the amend's selection page with the files still
  listed ([New batch › The amend's selection page](new-batch.md#the-amends-selection-page));
  **Cancel amend** leaves the batch as it was and returns to the Day screen. Neither asks, because
  an amend writes nothing before **Save batch 2 again**.
- When the plan is the one catch-all entry (no picks configured), a line above the rows reads
  **No picks yet: all orders go into one PDF set** ([Q-UI-4](../questions/ui.md)).

## Saving

While *Save batch* runs, the stage and a progress bar stand above the packing-list options
([saved-pdf-check.md](../app/saved-pdf-check.md): *Shown while it runs*):

```
│  Writing the saved PDFs …  page 143 of 257                                             │
│  ██████████████████████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░            │
```

The check before saving names what it caught in plain words — the errors and the warnings in two
lists, never one — and offers **Save anyway** and **Cancel — nothing is saved**:

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│  The saved files do not match the plan.                                                │
│                                                                                        │
│  Errors: 2                                                                             │
│  · PDF 2 (2 B Spion & Knalpot ×35.pdf): the page of order 580000000000000001 is        │
│    missing (0 of 1 written).                                                           │
│  · order 580000000000000042 is in two saved PDFs (2, 3); it must be in one.            │
│                                                                                        │
│  Warnings: 1                                                                           │
│  · PDF 1 (1 A Sepatu ×130.pdf): every page is there, but not in the run order.         │
│                                                                                        │
│  [ Save anyway ]   [ Cancel — nothing is saved ]                                       │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

- **Save anyway** saves the batch and keeps the two lists on its card and in `state.json`
  ([saved-pdf-check.md](../app/saved-pdf-check.md)); **Cancel — nothing is saved** closes the
  dialog and keeps the draft, so the Plan screen with the read labels is still there.
- The check runs on every save, whatever the size of the batch ([D-APP-2](../decisions/app.md)).
