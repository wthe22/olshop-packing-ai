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

Open, until the owner adds a label PDF exported **with product name and quantity** to
`samples/` (the current samples have none):

- which fields a condition can use without the CSV (what the label text holds and how reliably
  it can be read);
- where item data comes from when both the CSV and item text on the labels are present;
- what replaces batch and group numbers (01-requirements shared rules 3–7), and what that means
  for the app (not touched until the PC script is settled).

Built before D1 and still usable: `orders.py`, `rules.py`, `labels.py` (Order ID per page, page
copying). `packing_list.py` is built for the pack-group layout and will need changes.
Not started: `batches.py` (1.5), command line (1.8).
