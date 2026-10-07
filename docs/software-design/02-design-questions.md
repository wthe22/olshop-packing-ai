# 02 — Design Questions

Questions that need the owner's choice before the affected part is built. Add new ones as
`D1`, `D2`, … with the options; remove one once its answer is written into the other documents.

## D1 — PC script: picks instead of pack-group files (decided in part, design not yet rewritten)

The owner replaces "one label PDF per pack group" with a **pick** flow:

1. Input: the label PDFs of one download (up to 200 orders each).
2. A pick = a condition; the matching orders are saved as one PDF.
3. The next pick works on the orders that are left; repeat until the owner saves the rest as
   one PDF.

Decided:

| # | Point | Answer |
|---|---|---|
| 1 | How picks are given | Both: from the rules file by default (each entry = one pick, in order, the last = the rest); interactive when asked (script shows what is left, owner types a condition, saves, repeats, then saves the rest) |
| 2 | Page order inside a saved PDF | Re-ordered: labels with identical contents next to each other |
| 3 | Inputs | Label PDFs only should work; the orders CSV stays optional (the packing list needs item data). Warn when the label PDFs carry no product name and quantity (they can be left out at export on the seller centre) |
| 4 | A4 packing list | Owner's choice per run: one sheet per saved PDF, one sheet for the whole run with a section per saved PDF, or none |
| 5 | Several downloads per day | Normally each download holds only new labels. Guard: the script remembers every order saved that day; an order seen again is left out of the saved PDFs and listed in a warning (Order ID, tracking ID, when first saved) |
| 6 | Date and time in conditions | Orders from the CSV must be filterable by date, time of day and date+time, on the time fields that matter at packing time |
| 7 | "Identical contents" (re-ordering, runs) | Always product name + variation + quantity per line, so the result is the same with or without the CSV |
| 8 | Date/time fields | `paid_time`, `rts_time`, `created_time` (CSV) and `ship_by` (label `In transit by`, works without the CSV) |
| 9 | Date/time values | One field per time; the value's form decides: `paid_time = "2026-10-06"` (that day) · `paid_time < "14:00"` (time of day, any date) · `paid_time < "2026-10-06 14:00"` (exact moment) |
| 10 | Numbers | Saved PDFs numbered through the day (1, 2, 3 …); the runs of identical labels inside each numbered 01, 02 …; written `3-05` on the packing list |
| 11 | CSV and slips both given | The CSV supplies the items; where the slip says something different, the script warns |
| 12 | CSV-only field without a CSV | `sku_id`, `product_category`, `channel`, `paid_time`, `rts_time`, `created_time` have no value without the CSV: a pick using one stops the run with an error naming the pick and the field |
| 13 | Courier without the CSV | New field `tracking_id`. `courier` is also deduced from the label, not only from the tracking-ID pattern (e.g. the courier's name or web address printed on the label, the label's sort-code style); only J&T labels print the courier name as text in the sample (517 of 601 pages) |

Item data on the labels: the **Shipping label + Packing slip** export carries product name,
variation, Seller SKU and quantity per line, but no SKU ID
([business-process 02](../business-process/02-data-sources.md#label--packing-slip-export-option)).
Read by text position (column x-ranges); checked on the 600-order sample.

Open: what this means for the app (not touched until the PC script is settled).

Built before D1 and still usable: `orders.py`, `rules.py`, `labels.py` (Order ID per page, page
copying). `packing_list.py` is built for the pack-group layout and will need changes.
Not started: `batches.py` (1.5), command line (1.8).
