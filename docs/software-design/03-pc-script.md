# 03 — PC Script

A small Python command-line script. Proposal; not implemented.

## Stack

| Need | Library | Note |
|---|---|---|
| CSV | standard library `csv` | |
| Read page text, copy pages unchanged | `pypdf` (BSD-3) | Checked on the samples: text extraction finds the Order ID on all 562 pages |
| Write the A4 packing list | `fpdf2` (LGPL-3.0) or `reportlab` (BSD) | Embedded TTF font (e.g. Noto Sans) for any character in product names |
| Rules file | standard library `json` | |

Python from the project `.venv`.

## Usage

```
python packing.py prepare --csv orders.csv --labels "Shipping label_*.pdf" [--day 2026-10-06]
```

- `--day` defaults to today. Batch number is found automatically from the day's state.
- `--redo` rebuilds the newest batch (e.g. after fixing the rules file) without creating a new one.

## Day folder

```
work/2026-10-06/
  state.json                      orders seen today → batch number; pack-group codes
  batch-01/
    packing-list.pdf              A4, the whole batch
    labels/
      01 A01 ×957 Sepatu Standar Samping Motor x1.pdf
      02 A02 ×247 Sepatu Standar Samping Motor x2.pdf
      …
      27 B01 ×27 Spion Beat — Standard, honda x1.pdf
      …
    not-in-this-batch.pdf         label pages of orders that are not new in this batch (only if any)
    batch-01.zip                  for the app (only if D2-b)
  batch-02/
    …
```

- File names start with the print sequence, so a folder sorted by name is in print order.
- `work/` holds real order data; it is git-ignored.

## Steps of `prepare`

1. **Read CSV**: strip trailing tabs; keep rows with `Perlu dikirim` / `Menunggu pengambilan`;
   ignore the rest (count shown). Keep the packing columns. Group rows by Order ID.
2. **New orders** = orders not in `state.json`. They form batch N. Orders in `state.json` but not
   in the CSV are listed as removed (count only).
3. **Pack groups**: signature = sorted `SKU ID×quantity`. A signature seen earlier today keeps its
   code; a new one gets the next number of its category (`A01`, `A02`, `B01`, …).
4. **Category**: first matching category of the rules file.
5. **Label pages**: for each label PDF, page by page, find the Order ID (the 18-digit number on
   the page). IDX/Tokopedia labels extract as single characters; whitespace is removed before
   searching when the first search finds nothing. A page without an Order ID continues the
   order of the previous page, which keeps multi-page labels together. Pages are copied
   unchanged.
6. **Output order**: categories in rules order → pack groups by number of orders in this batch,
   largest first → orders by RTS time, then Order ID.
7. **Write** the label PDFs, the packing list, and `state.json`. Print a short summary:
   new / removed / ignored rows, groups, label pages used, pages not in this batch, new orders
   without a label page.

## Packing list (A4)

- A4 portrait, 10 mm margins, body 10.5 pt, group codes 13 pt bold.
- Header line: day · batch N · print time · page x/y.
- Per category: heading with its totals (orders, units); one row per pack group in print order:
  sequence, code, items (one line per item: display name × quantity), orders.
- Last section: pick summary — one row per SKU: display name, units in this batch.
- ~40 rows per page: the 49 groups of 2026-10-06 fit on 2 pages; batch 2 (20 groups) on 1.

```
2026-10-06 · Batch 2 · printed 16:40                                       page 1/1
A  Sepatu ─────────────────────────────────────────────────── 420 orders · 512 units
 01  A01  Sepatu Standar Samping Motor ×1                                348 orders
 02  A02  Sepatu Standar Samping Motor ×2                                 61 orders
 …
B  Spion & Knalpot ────────────────────────────────────────────  52 orders ·  53 units
 05  B01  Spion Beat — Standard, honda ×1                                  9 orders
 11  B07  Cover Knalpot Beat — FI 2012-2015 ×1                             1 order
          Spion Beat — Chrome Standard, honda ×1
Pick summary ──────────────────────────────────────────────────────────────────────
 Sepatu Standar Samping Motor                                          512 units
 …
```
(Figures illustrative.)

## Rules file (`categories.json`)

Ordered list. Condition tree: `all` (AND), `any` (OR), `not`, and leaves
`{field, op, value}`. Text compare is case-insensitive.

```json
[
  {"code": "A", "name": "Sepatu",
   "when": {"field": "name", "op": "contains", "value": "sepatu"}},
  {"code": "B", "name": "Spion & Knalpot",
   "when": {"all": [
     {"not": {"field": "name", "op": "contains", "value": "sepatu"}},
     {"any": [{"field": "name", "op": "contains", "value": "spion"},
              {"field": "name", "op": "contains", "value": "knalpot"}]}]}},
  {"code": "C", "name": "Motor lainnya",
   "when": {"field": "product_category", "op": "contains", "value": "sepeda motor"}},
  {"code": "Z", "name": "Lainnya"}
]
```

| Field | Level | Meaning |
|---|---|---|
| `name` | item | Full product name |
| `display_name` | item | Name before the first `\|` |
| `variation`, `sku_id`, `seller_sku`, `product_category` | item | |
| `line_quantity` | item | Quantity of one line |
| `total_quantity` | order | Sum of quantities |
| `distinct_items` | order | Number of lines |
| `courier`, `channel` | order | |
| `has_buyer_message` | order | true / false |

Item-level leaf = true when **any** line of the order matches. So
`not (name contains sepatu)` = no line contains "sepatu".
Operators: text `contains`, `equals`, `starts_with`; number `=`, `≠`, `<`, `≤`, `>`, `≥`.
The same format is used by the app's scan filters (with extra fields `category`, `group`,
`batch`).
