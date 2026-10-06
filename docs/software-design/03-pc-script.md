# 03 — PC Script

A small Python command-line script. Not implemented.

## Stack

| Need | Library | Note |
|---|---|---|
| CSV | standard library `csv` | |
| Read page text, copy pages unchanged | `pypdf` (BSD-3) | Checked on the samples: finds the Order ID on all 562 pages |
| Write the A4 packing list | `fpdf2` (LGPL-3.0) | Checked with a layout prototype on the samples; embedded TTF font |
| Rules file | standard library `json` | |

Python from the project `.venv`.

## Usage

```
python packing.py prepare --csv orders.csv --labels "Shipping label_*.pdf" [--day 2026-10-06]
```

- `--day` defaults to today. The batch number is found from the day's state.
- `--redo` rebuilds the newest batch (e.g. after fixing the rules file) without creating a new one.

## Day folder

```
work/2026-10-06/
  state.json                      orders seen today → batch number
  batch-01/
    packing-list.pdf              A4, the whole batch
    labels/
      01 ×609 Sepatu Standar Samping Motor x1.pdf
      02 ×172 Sepatu Standar Samping Motor x2.pdf
      …
      06 ×19 Spion Beat — Standard, honda x1.pdf
      …
    not-in-this-batch.pdf         label pages of orders that are not new in this batch (only if any)
  batch-02/
    …
```

- File names start with the group number, so a folder sorted by name is in print order.
- `work/` holds real order data; it is git-ignored.

## Steps of `prepare`

1. **Read CSV**: strip trailing tabs; keep rows with `Perlu dikirim` / `Menunggu pengambilan`;
   ignore the rest (count shown). Keep the packing columns. Group rows by Order ID.
2. **New orders** = orders not in `state.json`. They form batch N. Orders in `state.json` but not
   in the CSV are counted as removed.
3. **Category**: first matching category of `categories.json`.
4. **Pack groups**: signature = sorted `SKU ID×quantity`. Numbered in print order:
   categories in list order → number of orders, largest first → signature text.
5. **Label pages**: for each label PDF, page by page, find the Order ID (the 18-digit number on
   the page). IDX/Tokopedia labels extract as single characters; whitespace is removed before
   searching when the first search finds nothing. A page without an Order ID continues the
   order of the previous page, which keeps multi-page labels together. Pages are copied
   unchanged.
6. **Orders inside a group**: by RTS time, then Order ID.
7. **Write** the label PDFs, the packing list, and `state.json`. Print a short summary:
   new / removed / ignored rows, groups, label pages used, pages not in this batch, new orders
   without a label page.

## Packing list (A4)

- A4 portrait, 10 mm margins, Arial. Items 10.5 pt; group numbers and order counts 11.5 pt bold.
- Header on every page: `Packing list · <day> · Batch N`, `page x/y`; second line: orders ·
  units · groups · print time.
- Per category: grey heading bar with the category's orders and units; one row per pack group:
  group number · items (one line per item: display name ×quantity) · number of orders.
  Thin line between rows.
- Pick summary: two columns, units first, then the display name (wraps, never cut).
- On the samples: batch 1 (956 orders, 46 groups) = 2 pages; batch 2 (481 orders, 20 groups)
  = 1 page.

```
Packing list · 2026-10-06 · Batch 2                                         page 1/1
481 orders · 607 units · 20 groups · printed 16:40
▓ A  Sepatu ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓ 446 orders · 572 units
01  Sepatu Standar Samping Motor  ×1                                            348
02  Sepatu Standar Samping Motor  ×2                                             75
…
▓ B  Spion & Knalpot ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓ 35 orders · 35 units
05  Spion Beat — Standard, honda  ×1                                              9
…
▓ Pick summary (units to take from stock) ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓
 572  Sepatu Standar Samping Motor            1  Spion Scoopy New Gagang Hitam — Dove,
   9  Spion Beat — Standard, honda               honda, Datar
 …
```

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
| `display_name` | item | Display name (see 01) |
| `variation`, `sku_id`, `seller_sku`, `product_category` | item | |
| `line_quantity` | item | Quantity of one line |
| `total_quantity` | order | Sum of quantities |
| `distinct_items` | order | Number of lines |
| `courier`, `channel` | order | |
| `has_buyer_message` | order | true / false |

Item-level leaf = true when **any** line of the order matches. So
`not (name contains sepatu)` = no line contains "sepatu".
Operators: text `contains`, `equals`, `starts_with`; number `=`, `≠`, `<`, `≤`, `>`, `≥`.
The app's scan filters use the same format, with extra fields `category`, `group`, `batch`.
