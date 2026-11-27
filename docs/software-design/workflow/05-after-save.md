# Steps 16–20: after the save

The [Day](../ui/day.md) screen shows the live batches of the open date, newest first: each
group PDF with **Open** and its printed tick, the summary PDF, and *Left out ▸*, *Warnings ▸* and *Saved
with `<n>` errors ▸* where they apply. **Open** opens a file in the PC's PDF viewer for printing.

## 16. Print, Sent and Printed

**Print** opens the print preview of an output over the app's window. For a group PDF it sets the group's
`press_print_time` as the preview opens; the preview does not say whether Print or Cancel was pressed, so the
*Sent* mark's ✕ clears it. The tick beside a group PDF says it has really been printed: a click sets
`is_printed`, with or without *Sent*. Each is a commit that writes `day.toml` only
([save](../data/save.md#a-commit)). Both are notes only: nothing is enforced or blocked
([D-APP-26](../decisions/app-features.md)). An amend writes the batch afresh, so both start empty.

**Keeps:** I8.

## 17. Revert (newest live batch only)

After a question that names the batch and its files and offers to save its label PDFs or discard them
([D-APP-31](../decisions/app-features.md)). *Save* opens the Windows folder picker; closing it reverts
nothing.

1. Saved: copy the batch's stored input files (its label PDFs) into a new folder
   `{date} batch {n} reverted {hh.mm}` of the folder picked (` (2)` after the name when it is taken). A
   copy that fails takes the folder away and stops the revert.
2. Move the batch's outputs into `.partial/replaced/`. A file Windows refuses to move stops the revert:
   the files moved go back, the copies of step 1 are taken away, and the message names the file to close.
3. Write the three files without the batch into `.partial/` and rename `.partial/day.toml` over `day.toml`
   ([save](../data/save.md#a-commit)); a failure takes the copies away.
4. Move the other files in and delete `.partial/`; delete the batch's account summaries; refresh the search index.

The batch's orders count as unsaved again, and the next batch takes its number. Its inputs stay in the date folder.
The copies are the owner's: nothing reads them again.

**Keeps:** I2, I6, I7 — only the newest batch, because later numbers and the guard depend on it.

## 18. Amend (newest live batch only)

For "a filter was wrong, do the last batch again":

1. Its inputs are read again from the date folder, in `day.toml`'s order, with the sort rule of the same name as
   it is now in `config.toml`, or the batch's snapshot when no rule has that name
   ([D-RULES-2](../decisions/rules.md)). The guard (step 7) leaves the batch's own orders out of the
   saved set; each repeat's kept copy is preselected from `day.toml`.
2. **Back** shows the amend's file list: the owner may add files from anywhere and take files off; a file
   taken off leaves its orders unsaved, for a later batch ([D-APP-19](../decisions/app-features.md)).
3. **Save batch** runs steps 11–15 with `amends: true` and the same number; step 14 moves the old
   outputs aside first. **Cancel amend** writes nothing.

**Check:** printed labels may no longer match the amended files; the batch's Sent time and printed tick start empty.

## 19. Search and reprint

- **Search** filters the in-memory index of every date over a range of dates or all of them, and shows
  each order with its trace ([reprint](../app/reprint.md#search)).
- **Reprint** writes one PDF of the ticked orders' pages: the sources read from each order's date files, the pages unchanged from the stored inputs in the original printing order, written through
  `.partial/` of today's date, verified (V2, V5), committed with the reprint added to today's `day.toml`,
  then moved into today's folder ([reprint](../app/reprint.md#reprint)). A finding stops it; nothing is kept.

**Keeps:** I11. A reprint changes no batch and not the already-saved guard.

## 20. Restore

An output of a live batch found missing when the date opened is written again on **Restore**, and one that
is not there is written again by **Open**: a group PDF from the stored inputs by the files, the summary PDF from them
in today's layout; verified (V2, V5 or V6), committed, moved into the date folder
([reprint](../app/reprint.md#restore)).

**Keeps:** I5, I12 — a restored file is checked like the original.

## 21. Read labels again

Offered by a changed or missing `orders.csv` or `order-contents.toml` ([D-APP-40](../decisions/date-files.md))
and by **Check data** ([D-APP-41](../decisions/date-files.md)). Asked first; the question names what will
change. It shows its progress (*Reading labels: batch 1, page 3 of 120*).

1. For each live batch, oldest first: read its stored inputs in `day.toml`'s order with today's
   `parser.toml`, then plan with the batch's own sort rule from `day.toml` and its left-out copies as chosen.
2. Verify the batch's group PDFs against the orders read (V2, V5): the PDFs stay as they are, so the reading
   must name their pages. A batch that does not match, or an input that is missing or changed, stops the
   step with its name, and nothing is written.
3. Commit `orders.csv`, `order-contents.toml` and `day.toml` (its content counts and picking lists); the
   Sent times, printed ticks and reprints stay.

**Keeps:** I8, I12 — every value comes from its label page, checked against what was printed.

## Failure model

Each of 16–21 is one commit: before the rename the date is as before; after it the next open settles
`.partial/` by whether it holds `day.toml` ([save](../data/save.md#when-a-date-opens)).
