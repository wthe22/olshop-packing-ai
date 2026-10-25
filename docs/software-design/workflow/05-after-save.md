# Steps 16–20: after the save

The [Day](../ui/day.md) screen shows the live batches of the open date, newest first, from the fold: each
group PDF with **Open** and its printed tick, the summary PDF, and *Left out ▸*, *Warnings ▸* and *Saved
with `<n>` errors ▸* where they apply. **Open** opens a file in the PC's PDF viewer for printing.

## 16. Printed

The tick beside a group PDF says that it has been printed. A click appends `printed_marked {batch, group,
printed}` and `sync_all`; the views are written again. It is a note only: nothing is enforced or blocked.
An amend writes the batch afresh, so its ticks start empty.

**Keeps:** I8.

## 17. Revert (newest live batch only)

After a confirmation that names the batch and its files:

1. Move the batch's outputs into `output/.partial/replaced/`. A file Windows refuses to move stops the revert:
   the files moved go back, and the message names the file to close.
2. Append `batch_reverted {batch}` and `sync_all`.
3. Delete `.partial/`; write the views; update the search index.

The batch's orders count as unsaved again, and the next batch takes its number. Its inputs stay in `input/`.

**Keeps:** I2, I6, I7 — only the newest batch, because later numbers and the guard depend on it.

## 18. Amend (newest live batch only)

For "a filter was wrong, do the last batch again":

1. Its inputs are read again from `input/`, in the recorded order, with the sort rule of the same name as
   it is now in `packing-workflows.toml`, or the batch's snapshot when no rule has that name
   ([D-RULES-2](../decisions/rules.md)). The guard (step 7) leaves the batch's own orders out of the
   saved set; each repeat's kept copy is preselected from the record.
2. **Back** shows the amend's file list: the owner may add files from anywhere and take files off; a file
   taken off leaves its orders unsaved, for a later batch ([D-APP-19](../decisions/app-features.md)).
3. **Save batch `<b>` again** runs steps 11–15 with `amends: true` and the same number; step 14 moves the
   old outputs aside first. **Cancel amend** writes nothing.

**Check:** printed labels may no longer match the amended files; the confirmation says so.

## 19. Search and reprint

- **Search** filters the in-memory index of the namespace, *This day* or *All days*, and shows each order
  with its trace ([reprint](../app/reprint.md#search)).
- **Reprint** writes one PDF of the ticked orders' pages: the sources from a fold of each order's date
  journal, the pages unchanged from `input/` in the original printing order, written through
  `output/.partial/` of today's date, verified (V2, V5), recorded as `reprint_made` in today's journal,
  then moved into `output/` ([reprint](../app/reprint.md#reprint)). A finding stops it; nothing is kept.

**Keeps:** I11. A reprint changes no batch and not the already-saved guard.

## 20. Restore

An output of a live batch found missing or changed when the date opened is written again on **Restore**: a
group PDF from `input/` by its record, the summary PDF from its record; verified (V2, V5 or V6), recorded as
`output_restored`, moved into `output/` ([reprint](../app/reprint.md#restore)).

**Keeps:** I5, I12 — a restored file is checked like the original.

## Failure model

Each of 16–20 is one append: before it returns the date is as before; after it the next open settles
`output/.partial/` by the journal ([journal](../data/journal.md#when-a-date-opens)).
