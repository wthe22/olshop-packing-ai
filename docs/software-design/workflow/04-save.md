# Steps 11–15: from the plan to a saved batch

**Save batch** runs on its own thread with its stage and a progress bar ([Plan](../ui/plan.md#saving)).
Nothing of the date changes until step 14.

## 11. Store the inputs

For each input of the batch that the date folder does not hold yet: copy it to `{name}.partial` there,
`sync_all`, hash the copy again, and rename it to `{name}`, where `{name}` is
`{batch} Input {pdf} {short hash}.pdf` ([D-APP-10](../decisions/app.md)). A copy whose hash differs stops the
save. From here on every page is taken from the stored inputs, never from the file the owner chose.

An input is never deleted. One stored by a save that is then cancelled is named by no file; it is
harmless, and the next save of the same file finds it in place.

**Keeps:** I9 — the bytes stored are the bytes hashed and read.

## 12. Write the outputs and the files

Into the date folder's `.partial/` (the open-time check has settled any earlier one):

1. **Group PDFs**, in plan order: each order's written copy, its pages copied unchanged from the stored inputs and
   together, in printing order (step 10).
2. **The summary PDF** of all the batch's accounts from the plan ([summary PDF](../labels/summary-pdf.md)).
3. **The three files** as they will be ([files](../data/files.md)): `orders.csv` and `order-contents.toml`
   with the live batches' orders and contents, and `day.toml` with the batch (the inputs, the sort rule as
   used, the groups, the copies left out, the read warnings) and its hashes of the other two.

**Stops:** a full disk or a folder that cannot be written; `.partial/` is deleted and the date is as before.

**Keeps:** I5 — PDFium copies pages, it does not draw them.

## 13. Verify

`packing-verify` checks the three files in `.partial/` against the date's current files, the stored inputs
and the outputs there: V2–V6 of [verification](../app/verification.md).

- **No finding**: go on.
- **A finding** (an error, or a warning alone) stops: the dialog lists errors and warnings apart.
  **Cancel — nothing is saved** deletes `.partial/` and keeps the draft. **Save anyway** adds the findings to
  the batch's `errors` and `warnings` in `.partial/day.toml` (nothing else in it changes) and goes on. The
  summary PDF, written before the verifier ran, does not list them; the Day card and `day.toml` do.

**Keeps:** proves I1–I5 and I12 on the files themselves.

## 14. Commit ([save](../data/save.md#a-commit))

1. **Amend only**: move the batch's old outputs into `.partial/replaced/`. A file Windows refuses to move (a
   viewer has it open) stops the save: the files moved go back, and the message names the file to close.
2. **Rename `.partial/day.toml` over `day.toml`.** This is the commit: before it nothing is saved, after it
   the batch is saved.

**Keeps:** I6, I7, I8 — one rename, the files written whole.

## 15. Move the rest in

1. Move every other file of `.partial/` (the outputs, `orders.csv`, `order-contents.toml`) into the date
   folder, replacing a file of the same name; delete `.partial/` with `replaced/`. A move Windows refuses
   leaves the batch saved: the message names the file, and the next open moves it forward.
2. Refresh the date in the search index. The Day screen shows the batch on top.

**Keeps:** I10.

## Failure model in one table

| Fails at | The date is |
|---|---|
| 3–10 (reading, plan) | As before: nothing was written |
| 11 (inputs) | As before, plus an input that `day.toml` does not name (harmless) |
| 12–13 (write, verify; **Cancel**) | As before: `.partial/` deleted, or settled back at the next open |
| 14, before the rename | As before: `.partial/` holds `day.toml`, so the next open settles it back |
| 14 after the rename, before 15's moves | Saved: `.partial/` holds no `day.toml`, so the next open moves the rest forward |
