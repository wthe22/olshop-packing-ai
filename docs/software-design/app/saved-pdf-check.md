# Check before saving

Wrong code could copy a label page twice, drop one, put an order in two saved PDFs or in none, or
write the pages in the wrong order. Without a check that is found only at the printer.

After the saved PDFs and the packing lists are written and before `batch <b>.partial/` is renamed
([batch.md](batch.md), step 6), the app reads its own output back and compares it with the plan. It
uses the same page reader as the reading step of the batch, so there is no second parser. The
check runs on every save, whatever the size of the batch ([D-APP-2](../decisions/app.md)).

## What is compared

Every order appears exactly once, in one saved PDF, page for page and in order. The counts printed on
each packing list equal the plan's counts.

## Two severities, never one list

| Severity | Findings |
|---|---|
| **Error** | A page is missing. A page is twice in one PDF. An order is in two saved PDFs. An order is in no PDF. A page holds another order's label. A written page count differs from the plan. Packing-list counts differ from the plan |
| **Warning** (lower severity) | All pages are there exactly once, but the **order** inside a saved PDF differs from the run order |

The dialog, the Day screen and `state.json` keep the two lists apart.

## Save anyway, and the findings stay visible

- The dialog names the findings in plain words and offers **Save anyway** and **Cancel — nothing is
  saved**.
- **Cancel** deletes the temporary folder, leaves `state.json` untouched and keeps the draft (the
  Plan screen with the read labels). This is what the all-or-nothing save does on any error.
- **Save anyway** writes both lists into the batch's `state.json` entry (`errors` and `warnings`;
  absent in a day folder written without them, which loads as empty lists), prints them in
  `summary.txt`, and the Day screen shows them on the batch card (*Saved with 2 errors ▸*) until the
  batch is reverted or amended. It is never a message that scrolls away.

## Shown while it runs

The check takes 1–2 s on a large day (600 pages, 5 saved PDFs). *Save batch* is therefore a
command that runs on its own thread and emits a progress event, as the reading step does with
`read-progress`:

| Event | Payload | Shown |
|---|---|---|
| `save-progress` | `{stage, page, pages}`; stage *Writing* or *Checking* | The stage and a progress bar on the Plan screen |

The window never looks frozen while writing or checking.
