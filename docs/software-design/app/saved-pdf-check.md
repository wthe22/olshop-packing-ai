# Check before saving

Wrong code could copy a label page twice, drop one, put an order in two saved PDFs or in none, or
write the pages in the wrong order. Without a check that is found only at the printer.

After the saved PDFs and the packing lists are written and before `batch <b>.partial/` is renamed
([batch.md](batch.md), step 6), the app reads its own output back and compares it with the input pages and the plan. It
uses the same page reader as the reading step of the batch, so there is no second parser. The
check runs on every save, whatever the size of the batch ([D-APP-2](../decisions/app.md)).

## What is compared

Two things: **every input page appears exactly once** — written once, or left out on purpose, never
twice and never lost (1:1, never 1:N, never 1:0) — and the counts printed on each packing list equal
the plan's counts.

### The page ledger

An **input page** is a page of the files read, `(file, page)`. Each ends in one of two places:

| Status | Meaning |
|---|---|
| **written** | The plan puts it at one position of one saved PDF |
| **left out** | A `left_out` record covers it: a repeat, or an order saved earlier ([duplicates](duplicates.md)) |

The ledger adds up: **pages read = pages written + pages left out**, and the pages written are the
pages of the saved PDFs. The reader checks the first half when it groups pages into copies (a page
that fits no copy stops the batch). The check before saving checks the second half, on the files:

- The expected side is made from the **input pages and the `left_out` records**, not from the plan's
  own order list, so a plan that doubles or drops a page cannot approve itself.
- Each saved PDF is read back page by page and every page gets its **page fingerprint**
  ([reading.md](../labels/reading.md#page-identity)). For each fingerprint *F* the app counts the input
  pages to write with *F* (planned) and the saved pages with *F* (found): found equal to planned is
  1:1; found above planned is **1:N**; found below is **1:0**. A left-out copy has the same fingerprint
  as the copy kept, so a left-out copy written anyway is 1:N too.
- The page at each position of each saved PDF must be the one the plan put there; all pages right but
  in another order is the warning below.
- The Order ID is only the label of a message; the identity of a page is its fingerprint
  ([Q-LABEL-10](../questions/duplicates.md)).

## Two severities, never one list

| Severity | Findings |
|---|---|
| **Error** | **1:0** an input page is in no saved PDF. **1:N** an input page is written more than once, in one or in two saved PDFs. A saved page matches no input page. An order is split over two saved PDFs. A written page count differs from the plan. Packing-list counts differ from the plan |
| **Warning** (lower severity) | All pages are there exactly once, but the **order** inside a saved PDF differs from the run order |

The messages name the input page and the order: *page 12 of `labels_1.pdf` (order 580000000000000042)
is written 2 times: PDF 2 page 4, PDF 3 page 9.* — *page 13 of `labels_1.pdf` (order …) is in no saved
PDF.* The dialog, the Day screen and `state.json` keep the two lists apart. `summary.txt` and the Plan
header carry the ledger line: *1000 pages read = 986 saved + 14 left out (14 repeats, 0 saved earlier)*.

## Save anyway, and the findings stay visible

Any finding stops the batch — an error and a warning alike: a swapped page order (a warning only)
is written only after **Save anyway**, as an error is
([Q-APP-1](../questions/app.md)).

- The dialog names the findings in plain words and offers **Save anyway** and **Cancel — nothing is
  saved**.
- **Cancel** deletes the temporary folder, leaves `state.json` untouched and keeps the draft (the
  Plan screen with the read labels). This is what the all-or-nothing save does on any error.
- **Save anyway** writes both lists into the batch's `state.json` entry (`errors` and `warnings`;
  absent in a day folder written without them, which loads as empty lists), prints them in
  `summary.txt`, and the Day screen shows them on the batch card (the errors under *Saved with 2
  errors ▸*, with *Warnings ▸* beside it) until the batch is reverted or amended. It is never a
  message that scrolls away.

## Shown while it runs

The check takes 1–2 s on a large day (600 pages, 5 saved PDFs). *Save batch* is therefore a
command that runs on its own thread and emits a progress event, as the reading step does with
`read-progress`:

| Event | Payload | Shown |
|---|---|---|
| `save-progress` | `{stage, page, pages}`; stage *Writing* or *Checking* | The stage and a progress bar on the Plan screen |

The window never looks frozen while writing or checking.
