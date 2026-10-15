# Reading labels and making picks

The rules by which the PC app ([app](../app/README.md)) reads a label download and turns it into
picks, saved PDFs and packing lists: the Order ID of a page, the packing slip, the courier, the
picks and their runs, the numbering and file names, and the look of the packing list. All order
data comes from the label and its packing slip. Requirements: `F-LABEL-1` to `F-LABEL-5` in
[requirements.md](../requirements.md); a **pick** selects orders from what is left and each
pick's orders are written as one **saved PDF** of label pages (the *Concepts* table in
[requirements.md](../requirements.md)).

## In this folder

- [reading.md](reading.md) — the per-order data, the Order ID from the label, multi-page orders
  and reading the packing slip.
- [courier.md](courier.md) — the courier deduced from the label.
- [picks.md](picks.md) — picks, a saved PDF, the numbering and the file names.
- [packing-list.md](packing-list.md) — the A4 packing list and its three layouts.
