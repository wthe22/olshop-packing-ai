# 03 — PC Script

A Python command-line script run on the Windows PC after each label download. It implements
P1–P6 of [01-requirements.md](01-requirements.md). A **pick** selects orders from what is left;
each pick's orders are written as one **saved PDF** of label pages (see *Concepts* in
[01-requirements.md](01-requirements.md#concepts)). The orders CSV is optional: the label and
its packing slip carry enough for the pick flow.

## Stack

| Need | Library | Note |
|---|---|---|
| Python | 3.12 or newer | `tomllib` is built in from 3.11 |
| CSV | standard library `csv` | |
| Rules file | standard library `tomllib` | Format in [04-rules-file.md](04-rules-file.md) |
| Read page text with position, copy pages unchanged | `pypdf` (BSD-3) | `page.extract_text(visitor_text=…)` reports each text run with its x and y; tested on the sample label and slip downloads (an Order ID on every page) |
| Write the A4 packing list | `fpdf2` (LGPL-3.0) | Layout prototype `script/prototype/packing_list_preview.py` |
| Fonts | Arial (`arial.ttf`, `arialbd.ttf`) and Consolas (`consola.ttf`) from `C:\Windows\Fonts` | Consolas = fixed width, so tracking-ID columns line up. Missing font → clear error |
| Tests | `pytest` | |

## Inputs

| Input | Required | Content |
|---|---|---|
| Shipping-label PDFs | yes | One or more files of this download (a download holds at most 200 pages). Either plain **Shipping label** or **Shipping label + Packing slip** ([business-process 02](../business-process/02-data-sources.md#label--packing-slip-export-option)); pages are read as they are, so one download may hold both kinds |
| Orders CSV | no | The seller-centre export ([business-process 02](../business-process/02-data-sources.md)). Fills values and is checked against the labels; the orders themselves come from the labels |
| `categories.toml` | yes | Each entry is one pick; also the menu of `--interactive` |

Without a slip and without a CSV there is **no item data**: the script warns, only the
label-level fields work (Order ID, tracking ID, courier, `ship_by`), a pick that uses an item
field stops the run, and no packing list is written.

## Command

Run from the repository root with the project virtual environment:

```
python -m packing prepare --labels <label.pdf> [<label.pdf> …] [--csv <orders.csv>]
                          [--rules categories.toml] [--interactive]
                          [--layout full,summary,pick] [--packing-list per-pdf|whole|none]
                          [--day 2026-10-07] [--work work] [--redo]
```

| Option | Default | Meaning |
|---|---|---|
| `--labels` | required | One or more label PDFs of this download (at most 200 pages each). Wildcards allowed |
| `--csv` | none | The orders CSV of the same day. Optional |
| `--rules` | `categories.toml` in the repository root | The picks, in order |
| `--interactive` | off | The owner chooses the picks from a menu (below) instead of running the rules file straight through |
| `--layout` | `full` | What a packing-list sheet shows, comma-separated for several: `full`, `summary`, `pick` |
| `--packing-list` | `per-pdf` | Packing-list scope: `per-pdf` (one sheet per saved PDF), `whole` (one sheet with a section per saved PDF), `none` |
| `--day` | today | The day folder |
| `--work` | `work` | Folder that holds the day folders. Git-ignored: it contains real order data |
| `--redo` | off | Forget the day's last invocation (its entries in `state.json`, its saved PDFs and packing lists), then run as normal ([D6](02-design-questions.md#d6--redo-after-fixing-categoriestoml)) |

## Interactive mode

With `--interactive` the owner picks from a numbered menu instead of running the rules file
straight through:

1. The script prints what is left (orders, label pages) and the `categories.toml` entries that
   still match at least one remaining order, numbered, each with its order count:
   ```
   600 orders left
     1  A  Sepatu            172
     2  B  Spion & Knalpot    35
   1-2 = save that pick · c = type a condition · r = save the rest
   ```
2. A number saves that entry's matching orders as one PDF. `c` asks for a condition
   ([04-rules-file.md](04-rules-file.md)) and saves its matching orders. The menu is shown again
   with the new counts.
3. `r` saves every remaining order as one PDF and ends the mode.

## Per-order data

| Value | Comes from |
|---|---|
| `Order ID` | The label text (two-step search below). The CSV `Order ID` is matched against it |
| `tracking_id` | The CSV `Tracking ID`; else the label text; a difference is warned |
| `courier` | The CSV `Shipping Provider Name`; else deduced from the label (below) |
| `ship_by` | The label `In transit by: dd/mm/yyyy hh:mm` (no CSV column) |
| `paid_time`, `rts_time`, `created_time` | The CSV (`Paid Time`, `RTS Time`, `Created Time`). CSV-only |
| `channel` | The CSV `Purchase Channel`. CSV-only |
| item lines | The CSV rows (product name, variation, `SKU ID`, Seller SKU, quantity, product category); else the slip table |
| buyer message | The CSV `Buyer Message`; else the slip's `Customer Message` (continuation page) |

**The CSV wins** over the slip and the label, and every difference is warned (Order ID, field,
the two values). An order in the CSV with no label page is a warning (a download may be
missing).

### Order ID from the label

`pypdf` `extract_text`; first search `(?<!\d)(5\d{17})(?!\d)`; if nothing is found, drop all
whitespace and search `OrderI[dD][:：](\d{18})` (the second colon is fullwidth U+FF1A). The raw
search must come first: on J&T labels the Order ID sits on its own line under the tracking ID,
and dropping whitespace first would glue the two into one number.

### Multi-page orders

A page with no Order ID belongs to the previous page's order; a first page with no Order ID is
an error (file and page). Consecutive pages with the same Order ID are one order (a slip's
continuation page repeats the Order ID, the table header and `Customer Message`). Label pages
are copied unchanged, so an order's pages stay together and in their original order.

## Reading the packing slip

The slip is read by text position, not by reading order: `pypdf`
`page.extract_text(visitor_text=…)` reports each text run with its x and y.

- **Columns** from the header words `Product Name`, `SKU`, `Seller SKU`, `Qty`: each word's x is
  a column's left edge; a column ends at the next header word's x (the last at the page's right
  edge). The `SKU` column holds the **variation**.
- **Rows**: sorted by y, a row starts at each `Qty` value. The runs inside a column's x-range at
  or after the row's y (and before the next row) are that cell. A wrapped cell joins its lines
  with a space, except a wrap directly after `-`, which joins without a space
  (`FI 2012-` + `2015` → `FI 2012-2015`).
- **Cross-check**: the slip prints `Qty Total:`; if it differs from the sum of the order's `Qty`,
  a warning is shown (Order ID, printed total, sum).

## Courier deduction (without a CSV)

1. The text on the label: the courier name, a courier web address, or the label's sort-code
   style. (In the sample only J&T labels print the courier name as text: 517 of 601 pages.)
2. Else the tracking-ID pattern ([business-process 02](../business-process/02-data-sources.md#columns-packing-needs)):
   `JY` + 10 digits → J&T Express; `TKP` + 10 digits → IDX. 12 digits alone is ambiguous (SiCepat
   starts `00`, J&T Cargo is also 12 digits), so it is used only together with step 1's clues.
3. Unknown → `courier` empty, and a warning lists those orders.

## Picks

- **Default**: the `categories.toml` entries in file order, each one pick. The last entry may
  have no `when` and takes the rest. A pick takes the matching orders **from what is left**
  after the earlier picks.
- **Leftovers**: orders matching no pick stay in "the rest" and are saved as a final PDF.
- **Duplicate guard**: an order already saved earlier the same day (`state.json`) is left out of
  the saved PDFs and listed in a warning with its Order ID, tracking ID and the time it was
  first saved.
- **No CSV / CSV of another day**: if the CSV holds none of the label orders, the script warns
  with the count and uses the slip and label values as if no CSV were given (every order is "not
  in the CSV").

## A saved PDF

- **Re-ordering**: inside a saved PDF the orders are grouped into **runs** of identical contents
  and printed run by run. The contents key is per line `product name` + `variation` + `quantity`,
  lines sorted; it is the same with or without a CSV. Runs are sorted by (number of orders
  descending, contents text ascending); inside a run the orders stay in download page order.
  Without item data there is nothing to group by: the saved PDF keeps the download page order
  and holds one run.
- **Pages**: all pages of each order, copied unchanged.

## Numbering and file names

- **Saved PDFs** are numbered through the day `1`, `2`, `3`, …; a later run of the script the
  same day continues the numbering from `state.json`.
- **Runs** inside a saved PDF are numbered `01`, `02`, … in printed order. A run is written
  `<pdf>-<run>`, e.g. `3-05` (saved PDF 3, run 05), in the summary and the packing list.
- **Saved-PDF file name**: `<n> <code> <name> ×<orders>.pdf`, e.g. `3 Z Lainnya ×393.pdf`.
  Characters not allowed in Windows file names (`\ / : * ? " < > |`) become `-`; the name is cut
  at 100 characters (ending in `…`) to stay clear of the Windows path limit. The number prefix
  makes "sort by name" equal printing order.

## Day folder and day state

```
work/2026-10-07/
  state.json
  summary.txt
  1 A Sepatu ×172.pdf
  2 B Spion & Knalpot ×35.pdf
  3 Z Lainnya ×393.pdf
  packing-list.pdf                 (scope per-pdf: packing-list-1.pdf, packing-list-2.pdf, …)
```

`state.json` remembers every order saved that day:

```json
{
  "day": "2026-10-07",
  "saved": [
    {"order_id": "580000000000000001", "tracking_id": "JY0000001234",
     "invocation": 1, "pdf": 1, "run": 5, "save_time": "2026-10-07T07:40:12+07:00"}
  ]
}
```

The next saved-PDF number and the duplicate guard both come from this file; nothing else is
needed to continue a day.

## Processing steps of `prepare`

1. **Read the rules file**. Any error stops the run with the line and
   column of `categories.toml` ([04-rules-file.md](04-rules-file.md)).
2. **Read the label PDFs** in the order given. Per page: the Order ID, the label-level fields
   (`ship_by`, courier text) and — when the page carries a slip — the item lines by text
   position. Group the pages into orders.
3. **Read the CSV** when given (UTF-8 with BOM, strip the trailing tab; keep the packing-status
   rows, count the others; group rows by Order ID).
4. **Merge**: fill each value from the CSV, else the slip/label, and warn per difference. Warn
   about CSV orders with no label page.
5. **Duplicate guard**: drop the orders already in `state.json` from what is left; warn.
6. **Picks**: `--interactive` shows the menu; else the rules file, entry by entry. Each pick
   takes its orders from what is left; the leftovers become the final pick (the rest).
7. **Runs**: re-order each saved PDF's orders into runs.
8. **Write** the saved PDFs, the packing list (when asked and item data is present), `state.json`
   and `summary.txt`, then print the summary.

### Screen summary

Example (figures illustrative):

```
Day 2026-10-07 · 2 label files · orders-07.csv
  Pages: 601 read · 600 orders · 0 not in the CSV
  Item data: slips (601 pages), CSV
  Pick A  Sepatu: 172 orders → 1  A Sepatu ×172.pdf
  Pick B  Spion & Knalpot: 35 orders → 2  B Spion & Knalpot ×35.pdf
  Rest Z  Lainnya: 393 orders → 3  Z Lainnya ×393.pdf
  Duplicates: 0
  Warnings: none
Written: work/2026-10-07/  (3 saved PDFs, packing-list.pdf)
```

## Errors and warnings

Errors stop the run before anything is written; warnings are listed and the run continues.

| Kind | Message |
|---|---|
| Error | Rules file: line and column of `categories.toml` |
| Error | A pick uses a field with no source: a CSV-only field without a CSV, or an item field with neither slip nor CSV — names the pick and the field |
| Error | A label page with no Order ID that is not a continuation (file and page) |
| Error | CSV rows with an empty Tracking ID or SKU ID (the export was made before shipment was arranged) — lists the Order IDs |
| Warning | The CSV and the slip differ (Order ID, field, both values) |
| Warning | No item data (no slip, no CSV): only label-level fields work; an item pick then stops the run and no packing list is written |
| Warning | The CSV holds none of the label orders (the slip and label values are used) |
| Warning | Couriers that could not be deduced (Order IDs / tracking IDs) |
| Warning | Orders already saved today (Order ID, tracking ID, first save time) |
| Warning | `Qty Total` differs from the sum of `Qty` (Order ID, printed total, sum) |
| Warning | Orders in the CSV with no label page |

## Packing list (A4)

- The layouts `full`, `summary`, `pick` keep their look. A heading is the saved PDF number and
  the pick: `1  A  Sepatu`. The groups under a heading are the runs, numbered `3-05`; in `full`
  each run's tracking IDs are printed as before.
- Scope, chosen per invocation (`--packing-list`): `per-pdf` (one sheet per saved PDF), `whole`
  (one sheet with a section per saved PDF), `none`.
- A packing list needs item data (CSV or slips). With none, it is skipped with a message (an
  item-level pick stops the run first).
- Header `Packing list · <day> · <n>  <code>  <name>` (scope `per-pdf`) or
  `Packing list · <day> · saved PDFs <first>-<last>` (scope `whole`), then the counts
  (`<orders> orders · <units> units · <runs> runs · printed hh:mm`) and `page x/y`.
- The pick summary counts units per item = (product name, variation), as slip lines have no
  SKU ID.

```
Packing list · 2026-10-07 · 2  B Spion & Knalpot                               page 1/1
35 orders · 35 units · 4 runs · printed 08:05
▓ 2  B  Spion & Knalpot ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓ 35 orders
2-01  Spion Beat  ×1                                                                6
    JY0000001234  JY0000001235  JY0000001236  JY0000001237  JY0000001238  JY0000001239
──────────────────────────────────────────────────────────────────────────────────────
2-02  Spion Beat  ×1 + Knalpot Beat  ×1                                              1
    JY0000001240
```

## Code layout

```
script/
  packing/
    __main__.py      command line (`prepare`), interactive loop, summary, errors and warnings
    orders.py        orders-CSV reading, display names, signatures
    rules.py         `categories.toml` + condition language (parser, evaluator)
    labels.py        page → Order ID + label-level fields, page copying
    slip.py          packing-slip table read by text position
    picks.py         picks, what is left, day state, numbering, duplicate guard, runs
    packing_list.py  the three layouts
  tests/             pytest; uses testdata/ (committed, made-up) and samples/ (local only,
                     tests skipped when absent)
  prototype/
    packing_list_preview.py   throwaway layout prototype; reference for packing_list.py
```
