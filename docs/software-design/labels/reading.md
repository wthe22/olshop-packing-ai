# Reading the label and the packing slip

## Per-order data

| Value | Comes from |
|---|---|
| `Order ID` | The label text (three-step search below) |
| `tracking_id` | The label text, in one of the forms of the data folder's `couriers.toml` ([courier.md](courier.md)) |
| `courier` | Deduced from the label ([courier.md](courier.md)). An instant-service label names none — its courier comes from the order export |
| `ship_by` | The label `In transit by: dd/mm/yyyy hh:mm`, or the Shopee-style label's `Batas Kirim: dd-mm-yyyy` (that day at 00:00) |
| item lines | The slip table (product name, variation, quantity) |
| buyer message | The slip's `Customer Message` (continuation page) |

## Order ID from the label

On the page text, first search `(?<!\d)(5\d{17})(?!\d)`; if nothing is found, drop all
whitespace and search `OrderI[dD][:：](\d{18})` (the second colon is fullwidth U+FF1A); if
nothing is found again, search `No\.Pesanan:\s*(\d{6}[0-9A-Z]{8})` (the third step, with a guard
against a longer token). The raw search must come first: on J&T labels the Order ID sits on its
own line under the tracking ID, and dropping whitespace first would glue the two into one
number.

The **Shopee-style label** ([business-process 02](../../business-process/02-data-sources.md))
prints no 18-digit Order ID at all: it keys the order by its own `No.Pesanan` number, 14
characters. That caption is the search's third step, so its pages are read like any other
style's. The step runs on the raw text before the collapsed one: dropping the whitespace glues
the number to the word printed below the caption, and the guard that rejects a longer token
would then reject every page.

## Multi-page orders and copies

A page with no Order ID belongs to the previous page's copy; a first page with no Order ID is an
error (file and page). A page with an Order ID **and** a label part — a tracking ID, or an image in
the header band — starts a **copy** of that order; a page with the Order ID and no label part
continues the order's copy (a slip's continuation page repeats the Order ID, the table header and
`Customer Message`, and carries no tracking ID and no courier logo). Two copies of one Order ID are a
**repeat**, never one longer order ([duplicates](../app/duplicates.md), [Q-LABEL-9](../questions/duplicates.md)).
Label pages are copied unchanged, so a copy's pages stay together and in their original order.

## Page identity

The **page fingerprint** names one page by what it prints: XXH3-64 of the page's text runs in page
order — each run's text with its x and y rounded to 0.1 pt — followed by the byte hashes of the
page's header-band images ([courier.md](courier.md)). A page copied into a saved PDF keeps its
fingerprint, so the same function recognises a page in the downloads and in the saved PDFs. It is
used to compare the copies of a repeated order ([duplicates](../app/duplicates.md)) and to match
saved pages to input pages ([saved-pdf-check.md](../app/saved-pdf-check.md)). The way a page is
fingerprinted is [Q-LABEL-10](../questions/duplicates.md).

## Reading the packing slip

The slip is read by text position, not by reading order: each text run of the page comes with
its x and y ([text runs from PDFium](../app/architecture.md)).

- **Headers.** A slip prints one of two headers, each naming the **role** of its four columns
  left to right: `Product Name` · `SKU` · `Seller SKU` · `Qty` on the TikTok and Tokopedia
  slips, whose `SKU` column holds the **variation**; `Nama Produk` · `SKU` · `Variasi` · `Qty` on
  the Shopee-style slip, whose `SKU` column holds the **seller SKU** and `Variasi` the variation.
  A cell fills the field its column's role names, so the two middle columns swap between the two
  slips. A header word is looked for in the **line's joined text** — the line's runs joined with
  no separator — because a font change inside a word ends the run (`Variasi` comes as `V` +
  `ariasi`), so no single run holds the whole word.
- **Columns**: each header word's first character gives its column's left edge (less a
  tolerance); a column ends at the next header word's x (the last at the page's right edge).
- **Rows**: sorted by y, a row starts at each `Qty` value. The runs inside a column's x-range at
  or after the row's y (and before the next row) are that cell. A wrapped cell joins its lines
  with a space, except a wrap directly after `-`, which joins without a space
  (`FI 2012-` + `2015` → `FI 2012-2015`).
- **The table's end** is the caption under it — `Qty Total:` on the TikTok and Tokopedia slips,
  `Pesan:` on the Shopee-style slip, which prints no total — so the caption's value and the
  footer's tracking IDs are not read into the last row.
- **Cross-check**: the slip prints `Qty Total:`; if it differs from the sum of the order's `Qty`,
  a warning is shown (Order ID, printed total, sum). The Shopee-style slip prints no total, so
  nothing is cross-checked on it and no warning is raised.
