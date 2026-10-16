# What happens in one batch

1. **Choose the files** with **Add files…** or by dropping them on the window
   ([New batch](../ui/new-batch.md)). Each file's fingerprint (size + XXH3-64)
   is checked against `state.json`: a file already used in an earlier batch the same day is marked
   *used in batch 1* (it can still be read; its orders are then left out as already saved).
2. **Read** the files in the order shown (default: by file name, which is download order),
   page by page, with a progress bar and a Cancel button. Per page: the Order ID, the
   label-level fields, the slip lines. Pages are grouped into orders ([reading rules](../labels/reading.md)).
   Every order gets its **download position** (1, 2, 3, … over all files of the batch); runs
   keep this order inside a run.
3. **Check**: no-slip pages stop the batch ([Inputs](inputs.md)); `Qty Total` differences, unknown
   couriers and missing tracking IDs are warnings.
4. **Duplicate guard**: the orders already saved the same day are taken out and listed (Order ID,
   tracking ID, batch, saved-PDF number, time; the time is the batch's `save_time`).
5. **Plan**: the `categories.toml` entries in file order, each taking its orders from what is
   left; the leftovers form the last pick, the rest. The owner may change the plan for this
   batch only: skip a pick, move it up or down, add a one-off condition at any position,
   rename the file. Every change recounts the whole plan at once (all in memory, no file
   written).
6. **Save** (one button):
   1. Number the saved PDFs from the day's next number (after the last batch's last number).
   2. Re-order each pick's orders into runs.
   3. Write everything into a temporary folder `labels/<day>/batch <b>.partial/`: the saved
      PDFs (pages copied unchanged), the packing lists, `summary.txt`, and a copy of the label
      files read, in `download/` (amend reads them back).
   4. Check before saving: the saved PDFs and packing lists are read back and compared with the
      plan; see [saved-pdf-check.md](saved-pdf-check.md).
   5. Rename the folder to `batch <b>` and add the batch to `state.json`.

   If anything fails before step 5, the temporary folder is deleted and the day is as before
   — there is never a half-written batch.

## Revert and amend (only the last batch)

Like git: the last batch can be undone (**revert**) or redone in place (**amend**). An earlier
batch cannot, because later batches' numbers and duplicate guard depend on it.

**Revert last batch** — after a confirmation that names the batch and its files:

1. Delete the folder `batch <b>` (all its saved PDFs, packing lists and summary).
2. Remove the batch and its orders from `state.json`.

The orders are treated as unsaved again, and the next batch reuses the numbers. If a file is open
in a PDF viewer, Windows refuses the delete: the app says which file to close and changes
nothing.

**Amend last batch** — for "I fixed a category, do the last batch again":

1. The **selection page** opens ([New batch](../ui/new-batch.md#the-amends-selection-page)): the
   batch's label PDFs are listed, already selected, read from its `download/` copy in the order
   stored in `state.json`. The owner adds PDFs ([Q-APP-2](../questions/app.md)) and takes PDFs off
   the list ([Q-APP-3](../questions/app.md)).
2. **Read labels** reads the selected files: the batch's own pages again in the same order, an
   added file as in a new batch. The Plan screen opens with their orders and the current
   `categories.toml`; the duplicate guard ignores the batch's own orders. **Back** on the Plan
   screen returns to the selection page with the files still listed.
3. **Save batch `<b>` again** writes a new `batch <b>.partial/` with the same first number, a
   `download/` copy of the files read and the same packing-list choice, takes the place of the old
   folder, and writes the batch into `state.json`. **Cancel amend** (bottom, beside the save
   button) and the selection page's **Cancel** leave the old batch untouched and return to the Day
   screen; neither asks, because an amend writes nothing before **Save batch `<b>` again**.

Amend after printing: the printed labels may not match the new files. The confirmation says so
("Batch 2 was saved at 09:12. If you already printed it, print it again after amending.").

## Packing list

- The three layouts ([Packing list](../labels/packing-list.md)), A4, fonts Arial and Consolas from
  `C:\Windows\Fonts`.
- Chosen per batch on the Plan screen: scope `per PDF` / `whole batch` / `none`, layouts any of
  `full`, `summary`, `pick`. Default: `whole batch`, `pick`. The default can be changed in Settings.
- Header as in [Packing list](../labels/packing-list.md) with the batch added:
  `Packing list · 2026-10-07 · batch 1 · 2  B  Spion & Knalpot`.
- **File size.** About 1.4 MB for one page, 1.6 MB for five because
  PDFium writes each whole font into the file. Accepted; the printed result is the same.
