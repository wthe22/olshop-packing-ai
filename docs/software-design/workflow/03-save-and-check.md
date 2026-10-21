# Steps 10–14: from the plan to a saved batch

## 10. Write the batch

**Save batch** runs on its own thread with a progress bar ([Plan](../ui/plan.md)); nothing of the
session changes until step 12. Everything goes into `batch <b>.partial/`, a stale one deleted first:

1. **Saved PDFs.** For each pick in plan order, its orders in run order (step 9), each order's
   pages copied **unchanged** and together, in the order they were read ([picks](../labels/picks.md)).
   Which page of which file is written comes from the copy that stayed (step 7).
2. **Packing lists** as chosen: scope (per PDF, whole batch, none) and layouts (full, summary, pick)
   ([packing-list](../labels/packing-list.md)).
3. **`summary.txt`**: the counts, warnings and check findings, and the ledger line *pages read = saved + left
   out* ([saved-pdf-check](../app/saved-pdf-check.md)).
4. **`download/`**: a copy of every label file read, under its own name, so an amend works when the
   originals in Downloads are gone ([data-folder](../app/data-folder.md)).

**Keeps:** I5. **Stops:** a file open in another program, a full disk or a folder that cannot be
written (Messages); the temporary folder is deleted and the session is as before.

## 11. The check before saving

The app reads its own saved PDFs and packing lists back with the same page reader
([saved-pdf-check](../app/saved-pdf-check.md)) and compares them with the **input pages and the
copies left out**, then with the plan:

1. Every saved page gets its fingerprint. Each input page to write must be found exactly once;
   found twice is **1:N**, not found is **1:0**; a saved page that is no input page is an error.
2. Each saved page is at the position the plan gave it; pages all present but in another order is a
   warning only.
3. No order is split over two saved PDFs; page counts and the packing-list counts equal the plan's.
4. The ledger adds up: **pages read = pages written + pages left out**.

An error stops the batch until the owner chooses **Save anyway**, after which the two lists (errors,
warnings) stay on the Day card. The check runs on every save, whatever the size of the batch.

**Keeps:** I4 — it is the proof of I1–I5 on the files themselves, not on the plan.

**Check:** the check reads the real saved files, so it also finds a page that the writer copied
wrongly; it cannot find an order the reader never saw (a label missing from the download).

## 12. Commit

1. The folder `batch <b>.partial/` is renamed to `batch <b>`.
2. The batch is added to `state.json` (its files and their fingerprints, its saved PDFs, the packing
   list choice, the grouping, the warnings, the errors and the copies left out), and each saved order
   to `saved[]`. The file is written whole (to `state.json.tmp`, then replaced).

**Keeps:** I7 — before the rename nothing changed; a crash between 1 and 2 leaves a batch folder with
no entry in `state.json`, which the next save of that number replaces.

## 13. The Day screen, revert and amend

- The Day screen shows the batches of the open session from `state.json`, newest first; **Open** opens
  a saved PDF or the packing list in the PC's PDF viewer for printing; the **Printed** tick is a
  note only.
- **Revert** (newest batch only): the batch folder is deleted and its entry and its orders removed
  from `state.json`, so its orders can be saved again.
- **Amend** (newest batch only): the batch's own `download/` files are read again with the
  grouping the batch used; the owner's earlier choices for repeats come back preselected; the Plan
  opens as *Redo batch b*; **Save batch** writes the batch again under the same number and the same
  first saved-PDF number, replacing the old folder (step 10–12).
- Only the newest batch, because later numbers and the already-saved guard depend on it.

**Check:** the amend replaces the old batch folder at step 12; the files of the old batch are deleted
when the new one is renamed in.

## Failure model in one table

| Fails at | Session is |
|---|---|
| 2–7 (reading, repeats) | As before: nothing was written |
| 10 (writing) | As before: the temporary folder is deleted |
| 11 (a finding) | As before until **Save anyway**; **Cancel** keeps the draft |
| 12, before the rename | As before |
| 12, after the rename, before `state.json` | A batch folder without an entry; the next save of that number replaces it |
