# Steps 11–15: from the plan to a saved batch

**Save batch** runs on its own thread with its stage and a progress bar ([Plan](../ui/plan.md#saving)).
Nothing of the date changes until step 14.

## 11. Store the inputs

For each input of the batch that `input/` does not hold yet: copy it to `input/{sha256}.pdf.partial`,
`sync_all`, hash the copy again, and rename it to `input/{sha256}.pdf`. A copy whose hash differs stops the
save. From here on every page is taken from `input/`, never from the file the owner chose.

An input is never deleted. One stored by a save that is then cancelled is named by no record; it is
harmless, and the next save of the same file finds it in place.

**Keeps:** I9 — the bytes stored are the bytes hashed and read.

## 12. Write the outputs

Into `output/.partial/` (the open-time check has settled any earlier one):

1. **Group PDFs**, in plan order: each order's written copy, its pages copied unchanged from `input/` and
   together, in printing order (step 10).
2. **The summary PDF** from the plan: the pick pages and the facts page ([summary PDF](../labels/summary-pdf.md)).
3. **The record**: a `batch_saved` built in memory with the next `seq` — the inputs, the sort rule as used,
   the contents, the orders in printing order with their copies and groups, the copies left out, every
   output with its pages, size and SHA-256, and the read warnings ([records](../data/records.md)).

**Stops:** a full disk or a folder that cannot be written; `.partial/` is deleted and the date is as before.

**Keeps:** I5 — PDFium copies pages, it does not draw them.

## 13. Verify

`packing-verify` checks the record's bytes against the journal, `input/` and `output/.partial/`: V1–V6 of
[verification](../app/verification.md).

- **No finding**: go on.
- **A finding** (an error, or a warning alone) stops: the dialog lists errors and warnings apart.
  **Cancel — nothing is saved** deletes `.partial/` and keeps the draft. **Save anyway** adds the findings to
  the record's `errors` and `warnings` (nothing else in it changes) and goes on. The summary PDF, written
  before the verifier ran, does not list them; the Day card and the journal do.

**Keeps:** proves I1–I5 and I12 on the files themselves.

## 14. Commit

1. **Amend only**: move the batch's old outputs into `.partial/replaced/`. A file Windows refuses to move (a
   viewer has it open) stops the save: the files moved go back, and the message names the file to close.
2. If the journal ends in a torn tail, cut the file at the end of its last good line, and put a `recovered`
   record with the dropped bytes in front of the batch's record; on a new date, put `started` in front.
3. **Append** the line(s) and `sync_all`. This is the commit: before it returns nothing is saved, after it
   the batch is saved.

**Keeps:** I6, I7, I8 — one sealed append, numbered by the fold.

## 15. Move the outputs, write the views

1. Move every file of `.partial/` into `output/`, replacing a file of the same name; delete `.partial/`
   with `replaced/`. A move Windows refuses leaves the batch saved: the message names the file, and the
   next open moves it forward.
2. Write `orders.csv`, `order-contents.toml`, then `session.toml`, each to a temporary file renamed into
   place ([views](../data/views.md)).
3. Add the record to the search index. The Day screen shows the batch on top.

**Keeps:** I10.

## Failure model in one table

| Fails at | The date is |
|---|---|
| 3–10 (reading, plan) | As before: nothing was written |
| 11 (inputs) | As before, plus an input that no record names (harmless) |
| 12–13 (write, verify; **Cancel**) | As before: `.partial/` deleted, or settled back at the next open |
| 14, before the append returns | As before: at most a torn tail, cut and recorded at the next commit |
| 14 after the append, before 15's moves | Saved: the next open moves the outputs forward |
| 15, writing a view | Saved: the views are written again at the next open |
