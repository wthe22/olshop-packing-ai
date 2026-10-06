# 03 — PC Script

A Python command-line script run on the Windows PC after every orders export. It implements
P1–P6 of [01-requirements.md](01-requirements.md).

## Stack

| Need | Library | Note |
|---|---|---|
| Python | 3.12 or newer | `tomllib` is built in from 3.11 |
| CSV | standard library `csv` | |
| Rules file | standard library `tomllib` | Format in [04-rules-file.md](04-rules-file.md) |
| Read page text, copy pages unchanged | `pypdf` (BSD-3) | Tested on the samples: finds the Order ID on all 562 pages |
| Write the A4 packing list | `fpdf2` (LGPL-3.0) | Tested with the layout prototype `script/prototype/packing_list_preview.py` |
| Fonts | Arial (`arial.ttf`, `arialbd.ttf`) and Consolas (`consola.ttf`) from `C:\Windows\Fonts` | Consolas = fixed width, so tracking-ID columns line up. Missing font → clear error |
| Tests | `pytest` | |

## Command

Run from the repository root with the project virtual environment:

```
python -m packing prepare --csv <orders.csv> --labels <label.pdf> [<label.pdf> …]
                          [--layout full,summary,pick] [--rules categories.toml]
                          [--day 2026-10-06] [--work work] [--redo]
```

| Option | Default | Meaning |
|---|---|---|
| `--csv` | required | The orders CSV exported from the seller centre |
| `--labels` | required | One or more label PDFs of this batch (downloads hold at most 200 pages each). Wildcards allowed |
| `--layout` | `full` | Packing-list layouts to write, comma-separated: `full`, `summary`, `pick` |
| `--rules` | `categories.toml` in the repository root | The categories file |
| `--day` | today | The day folder to use |
| `--work` | `work` | Folder that holds the day folders. Git-ignored: it contains real order data |
| `--redo` | off | Rebuild the newest batch instead of adding a new one (e.g. after fixing the rules file or adding a forgotten label PDF). Uses the same CSV and labels given on the command line |

Second command, for reprinting a packing list in another layout without touching the state:

```
python -m packing relist [--batch N] [--layout summary] [--day …] [--work …]
```

## Day folder

```
work/2026-10-06/
  state.json
  batch-01/
    packing-list-full.pdf
    packing-list-pick.pdf                         (only the layouts asked for)
    labels/
      01 ×609 Sepatu Standar Samping Motor x1.pdf
      02 ×172 Sepatu Standar Samping Motor x2.pdf
      …
      30 ×1 Cover Knalpot Beat — FI 2012-2015 x1 + Spion Beat — Chrome Standard, honda x1.pdf
      …
    not-in-this-batch.pdf                         (only if any)
    summary.txt                                   (the screen summary, kept)
  batch-02/
    …
```

### Label file names

`<group number> ×<orders> <items>.pdf`

- `<items>` = the group's items as `display name x<quantity>`, sorted by display name, joined
  with ` + `.
- Characters not allowed in Windows file names (`\ / : * ? " < > |`) become `-`.
- `<items>` is cut at 100 characters (ending in `…`) to stay clear of the Windows path limit.
- The number prefix makes "sort by name" equal print order.

### `state.json`

```json
{
  "day": "2026-10-06",
  "batches": [
    {
      "number": 1,
      "import_time": "2026-10-06T14:30:12+07:00",
      "csv_file": "orders-06-1.csv",
      "label_files": ["Shipping label_1.pdf", "Shipping label_2.pdf"],
      "orders": ["580000000000000001", "580000000000000002"]
    }
  ]
}
```

`orders` = the Order IDs that were **new** in that batch. This is all the script needs to find
the next batch. `--redo` removes the last entry, then runs as normal.

## Processing steps of `prepare`

1. **Read the rules file**. Any error (syntax, unknown field, wrong operator) stops the run
   with file, line and column.
2. **Read the CSV** (UTF-8 with BOM). Strip the trailing tab and surrounding spaces of every
   value. Keep rows with the packing status; count the others. Group rows by Order ID.
   A row with an empty Tracking ID or SKU ID → error listing the Order IDs (the export was made
   before shipment was arranged).
3. **New orders**: orders not listed in any batch of `state.json` form batch N.
   Orders of earlier batches missing from this CSV are counted as *removed* (shown, no action).
   Zero new orders → message "no new orders since batch N−1", nothing written.
4. **Category** per order (first match). An order matching no category goes under an automatic
   last heading `?  Uncategorised` and is listed in the summary as a warning.
5. **Pack groups and numbers**: shared rules 5–7 in 01-requirements.
6. **Label pages**: open each label PDF in the order given. For each page:
   - extract the text with pypdf and search it for `(?<!\d)(5\d{17})(?!\d)` (Order IDs are
     18 digits starting with 5; on J&T labels it sits on its own line under the tracking ID);
   - if nothing is found: IDX / Tokopedia labels extract as single characters with spaces
     between them, so remove all whitespace and search for `OrderI[dD][:：](\d{18})`.
     (Removing whitespace first would glue the Order ID to the tracking ID on J&T labels, so
     the order of the two searches matters.)
   - checked on the samples: 560 pages found by the first search, 2 by the second, 0 missed,
     never two different Order IDs on one page; about 25 s for 562 pages;
   - a page with no Order ID belongs to the order of the previous page (second page of a long
     label); a first page with no Order ID → error with file and page number. Consecutive
     pages with the same Order ID are also one order. (The samples hold no multi-page label,
     so which of the two a long label does is not known yet; both are handled.)
   - a page whose Order ID is not in batch N (earlier batch, or not in the CSV at all) goes to
     `not-in-this-batch.pdf` and is counted.
7. **Missing labels**: a new order with no label page → warning listing tracking ID and Order
   ID; its group's PDF is written without it, the packing list marks it `(no label)`.
8. **Write** the label PDFs (pages copied unchanged with pypdf), the packing-list PDF(s),
   `state.json` and `summary.txt`.

### Screen summary

Example (figures illustrative):

```
Batch 2 · 2026-10-06 · orders-06-2.csv
  CSV rows: 1,436 kept, 0 ignored (other status)
  Orders: 1,435 in CSV · 481 new · 2 removed since batch 1
  Groups: 20 (A Sepatu 4 · B Spion & Knalpot 10 · Z Lainnya 6)
  Label pages: 562 read · 481 used · 81 not in this batch
  Warnings: none
Written: work/2026-10-06/batch-02/  (20 label files, packing-list-full.pdf)
```

## Packing list (A4)

All layouts: A4 portrait, 10 mm margins. Header on every page:
`Packing list · <day> · Batch N` with `page x/y` on the right; second line
`<orders> orders · <units> units · <groups> groups · printed <hh:mm>`.

### Layout `full` (default)

- Per category: grey heading bar `A  Sepatu` with the category's orders and units on the right.
- Per pack group: group number (11.5 pt bold) · items, one line per item,
  `display name  ×quantity` (10.5 pt) · number of orders on the right (11.5 pt bold).
- Under the items: the group's tracking IDs in 6 columns (Consolas 10 pt), left → right then
  down, in label order (rule 7). A missing label shows `(no label)` after the ID.
- A group that continues on the next page repeats a grey line `05 (continued)  <items>`.
- Thin grey line between groups.
- Then the pick summary (below).
- Size on the samples: batch 1 (956 orders, 46 groups) 6 pages; batch 2 (481 orders) 3 pages.

```
Packing list · 2026-10-06 · Batch 2                                            page 2/3
481 orders · 607 units · 20 groups · printed 16:40
▓ B  Spion & Knalpot ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓ 35 orders · 35 units
05  Spion Beat — Standard, honda  ×1                                                 9
    JY0000000001  JY0000000002  JY0000000003  JY0000000004  JY0000000005  JY0000000006
    JY0000000007  000000000008  JY0000000009
──────────────────────────────────────────────────────────────────────────────────────
06  Cover Knalpot Beat — FI 2012-2015  ×1                                            1
    Spion Beat — Chrome Standard, honda  ×1
    JY0000000010
──────────────────────────────────────────────────────────────────────────────────────
```

### Layout `summary`

As `full` without the tracking-ID lines. Batch 1: 2 pages; batch 2: 1 page.

### Layout `pick`

Header, then only the pick summary. Normally one page.

### Pick summary (end of `full` and `summary`, whole of `pick`)

- Grey bar `Pick summary (units to take from stock)`.
- One entry per SKU of the batch: units (bold, right-aligned) then display name; sorted by
  display name.
- Two columns, left column filled first. Long names wrap onto a second line; never cut.

```
▓ Pick summary (units to take from stock) ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓
 572  Sepatu Standar Samping Motor               1  Spion Scoopy New Gagang Hitam — Dove,
   9  Spion Beat — Standard, honda                  honda, Datar
```

## Code layout

```
script/
  packing/
    __main__.py      command line (prepare, relist)
    orders.py        CSV reading, row filter, Order grouping, display names, signatures
    rules.py         categories.toml + condition language (parser, evaluator)
    batches.py       state.json, new/removed orders, group numbering
    labels.py        page → Order ID, page copying
    packing_list.py  the three layouts
  tests/             pytest; uses testdata/ (committed, made-up) and samples/ (local only,
                     tests skipped when absent)
  prototype/
    packing_list_preview.py   throwaway layout prototype; reference for packing_list.py
```
