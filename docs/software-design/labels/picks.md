# Picks

- **Default**: the categories of the grouping the batch uses ([rules file](../rules-file/README.md)) in
  file order, each one pick. The last entry may have no `when` and takes the rest. A pick takes the
  matching orders **from what is left** after the earlier picks. A grouping with no file — a fresh data
  folder, or a folder that has none — has one catch-all entry that takes every order
  ([inputs](../app/inputs.md)).
- **Leftovers**: orders matching no pick stay in "the rest" and are saved as a final PDF.
- **Duplicate guard**: an order already saved earlier the same day (the day's `state.json`,
  [data-folder](../app/data-folder.md)) is left out of the saved PDFs and listed in a warning
  with its Order ID, tracking ID and the time of its first save.

## A saved PDF

- **Re-ordering**: inside a saved PDF the orders are grouped into **runs** of identical contents
  and printed run by run. The contents key is per line `product name` + `variation` + `quantity`,
  lines sorted. Runs are sorted by (number of orders descending, contents text ascending); inside
  a run the orders stay in download page order.
- **Pages**: all pages of each order, copied unchanged.

## Numbering and file names

- **Saved PDFs** are numbered through the day `1`, `2`, `3`, …; a later batch the same day
  continues the numbering from the day's `state.json`.
- **Runs** inside a saved PDF are numbered `01`, `02`, … in printed order. A run is written
  `<pdf>-<run>`, e.g. `3-05` (saved PDF 3, run 05), in the packing list.
- **Saved-PDF file name**: `<n> <code> <name> ×<orders>.pdf`, e.g. `3 Z Lainnya ×393.pdf`.
  Characters not allowed in Windows file names (`\ / : * ? " < > |`) become `-`; the name is cut
  at 100 characters (ending in `…`) to stay clear of the Windows path limit. The number prefix
  makes "sort by name" equal printing order.
