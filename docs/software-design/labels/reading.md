# Reading the label and the packing slip

## Per-order data

| Value | Comes from |
|---|---|
| `Order ID` | The label text (two-step search below) |
| `tracking_id` | The label text |
| `courier` | Deduced from the label ([courier.md](courier.md)). An instant-service label names none — its courier comes from the order export |
| `ship_by` | The label `In transit by: dd/mm/yyyy hh:mm` |
| item lines | The slip table (product name, variation, quantity) |
| buyer message | The slip's `Customer Message` (continuation page) |

## Order ID from the label

On the page text, first search `(?<!\d)(5\d{17})(?!\d)`; if nothing is found, drop all
whitespace and search `OrderI[dD][:：](\d{18})` (the second colon is fullwidth U+FF1A). The raw
search must come first: on J&T labels the Order ID sits on its own line under the tracking ID,
and dropping whitespace first would glue the two into one number.

The **Shopee-style label** ([business-process 02](../../business-process/02-data-sources.md))
prints no 18-digit Order ID at all — its order number is 14 characters (`No.Pesanan`) — so this
search finds nothing on its pages and the batch would stop at the first one. How the app handles
that style is [Q-LABEL-4](../questions/label.md).

## Multi-page orders

A page with no Order ID belongs to the previous page's order; a first page with no Order ID is
an error (file and page). Consecutive pages with the same Order ID are one order (a slip's
continuation page repeats the Order ID, the table header and `Customer Message`). Label pages
are copied unchanged, so an order's pages stay together and in their original order.

## Reading the packing slip

The slip is read by text position, not by reading order: each text run of the page comes with
its x and y ([text runs from PDFium](../app/architecture.md)).

- **Columns** from the header words `Product Name`, `SKU`, `Seller SKU`, `Qty`: each word's x is
  a column's left edge; a column ends at the next header word's x (the last at the page's right
  edge). The `SKU` column holds the **variation**.
- **Rows**: sorted by y, a row starts at each `Qty` value. The runs inside a column's x-range at
  or after the row's y (and before the next row) are that cell. A wrapped cell joins its lines
  with a space, except a wrap directly after `-`, which joins without a space
  (`FI 2012-` + `2015` → `FI 2012-2015`).
- **Cross-check**: the slip prints `Qty Total:`; if it differs from the sum of the order's `Qty`,
  a warning is shown (Order ID, printed total, sum).
