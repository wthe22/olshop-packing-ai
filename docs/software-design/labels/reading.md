# Reading the label and the packing slip

## Per-order data

| Value | Comes from |
|---|---|
| `order_id` | The label text (three-step search below) |
| `tracking_id` | The label text, in one of the tracking-ID forms of `packing-workflows.toml` ([courier](courier.md)) |
| `courier`, `courier_clue` | Deduced from the label: the courier and the clue that named it ([courier](courier.md)) |
| `ship_by_date` | The date of the label's `In transit by: dd/mm/yyyy hh:mm`, or of the Shopee-style label's `Batas Kirim: dd-mm-yyyy` ([D-APP-21](../decisions/app-features.md)) |
| `pickup_code` | The instant-service label's code: the run of two capital letters and 4 digits printed in large type beside its `Kode Pengambilan` caption, within 30 pt of its line (the page text does not keep the two together) (kept in the record; its label prints it) |
| item lines | The slip table: product name, variation, Seller SKU, quantity |
| `total_quantity` | The slip's `Qty Total`; none on the Shopee-style slip |

An item is identified by its product name and variation ([D-APP-20](../decisions/app-features.md)); the platform's
`SKU ID` is not on the label PDFs.

## Every page must carry a packing slip

The slip is the only source of item data. A page that starts a copy and has no slip table stops the batch,
and nothing is written:

> **These labels have no packing slip.** 200 of 200 pages are plain shipping labels. In the seller centre,
> download the labels again with **Shipping label + Packing slip**.

If only some pages lack the slip, the message names the input and the pages.

## Order ID from the label

On the page text, first search `(?<!\d)(5\d{17})(?!\d)`; if nothing is found, drop all whitespace and
search `OrderI[dD][:：](\d{18})` (the second colon is fullwidth U+FF1A); if nothing is found again, search
`No\.Pesanan:\s*(\d{6}[0-9A-Z]{8})` on the raw text, with a guard against a longer token. The raw search
must come first: on J&T labels the Order ID sits on its own line under the tracking ID, and dropping
whitespace first would glue the two into one number. The third step also runs on the raw text: dropping the
whitespace glues the number to the word printed below its caption.

The **Shopee-style label** ([business-process 02](../../business-process/02-data-sources.md)) prints no
18-digit Order ID: it keys the order by its own `No.Pesanan` number, 14 characters
([D-LABEL-5](../decisions/label.md)).

## Copies and multi-page orders

A page that has an Order ID **and** a label part — a tracking ID, or an image in the header band — starts a
**copy** of that order. A page without a label part continues the copy of the page before it, in the same
input; a slip's continuation page repeats the Order ID, the table header and `Customer Message`, and
carries no tracking ID and no courier logo. A copy's pages are together and are copied unchanged, in their
order. Two copies of one Order ID are a **repeat**, never one longer order
([duplicates](../app/duplicates.md), [D-LABEL-12](../decisions/label.md)).

## Page fingerprint

The **page fingerprint** names a page by what it prints: XXH3-64 of the page's text runs in page order —
each run's text with its x and y rounded to 0.1 pt — followed by the byte hashes of its header-band images.
The reader makes it while reading, at no extra cost, and it compares the copies of a repeated order
(*same* or *differs*). Whether an output page *is* its source page is the verifier's **page digest**, a
different function in a different crate ([verification](../app/verification.md#the-page-digest),
[D-LABEL-13](../decisions/label.md)).

## Reading the packing slip

The slip is read by text position, not by reading order: each text run of the page comes with its x and y
([text runs from PDFium](../app/architecture.md#text-runs-from-pdfium)).

- **Headers.** A slip prints one of two headers, each naming the **role** of its four columns left to
  right: `Product Name` · `SKU` · `Seller SKU` · `Qty` on the TikTok and Tokopedia slips, whose `SKU`
  column holds the **variation**; `Nama Produk` · `SKU` · `Variasi` · `Qty` on the Shopee-style slip, whose
  `SKU` column holds the **Seller SKU** and `Variasi` the variation. A header word is looked for in the
  line's joined text (its runs joined with no separator), because a font change inside a word can end a run.
- **Columns**: each header word's first character gives its column's left edge (less a tolerance); a
  column ends at the next header word's x (the last at the page's right edge).
- **Rows**: sorted by y, a row starts at each `Qty` value. The runs inside a column's x-range at or after
  the row's y (and before the next row) are that cell. A wrapped cell joins its lines with a space, except
  a wrap directly after `-`, which joins without one (`FI 2012-` + `2015` → `FI 2012-2015`).
- **The table's end** is the caption under it — `Qty Total:` on the TikTok and Tokopedia slips, `Pesan:`
  on the Shopee-style slip — so the caption's value and the footer's tracking IDs are not read into the
  last row.
- **Variation** `Default` reads as empty.
- **Cross-check**: when the slip prints `Qty Total:` and it differs from the sum of the copy's `Qty`, a
  warning names the Order ID, the printed total and the sum. The Shopee-style slip prints no total and
  raises none.
