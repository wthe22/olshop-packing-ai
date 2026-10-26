# Open questions: labels

## Q-LABEL-12 — What the day summary holds

**Words.** The owner asked for a summary of the whole day (R-LABEL-3). The batch's
[summary PDF](../labels/summary-pdf.md) already says what one batch is, its groups with their order
contents and what to pick. A day summary could repeat all of that for the day, or hold only what a batch's
summary cannot: totals over the batches, the couriers' counts, what is not printed yet, the reprints.

**Example.** A day of 2 batches, 729 orders, 5 group PDFs and 1 reprint.

| Option | What the owner sees |
|---|---|
| **1. The day's facts and picks** (**Default**) | Page 1: the couriers' counts (`jnt · 412 orders`), each batch with its group PDFs and their printed tick (`2-Z Lainnya · 41 orders \| 12 contents · not printed`), the reprints with their orders. From a new page: the pick list of the whole day. About 2–3 pages |
| 2. Option 1 and every group's order contents | As 1, then each batch's groups with their order contents, as on the summary PDFs. About as many pages as all the day's summary PDFs together |
| 3. The pick list only | The units per item over the day, nothing else. One or two pages |
| 4. On screen, not a PDF | A *Day summary* panel on the Day screen with the same facts as 1; nothing to print |

**What the answer changes:** [day-summary.md](../labels/day-summary.md) and the PDF writer; with option 4,
the Day screen instead of a file.
