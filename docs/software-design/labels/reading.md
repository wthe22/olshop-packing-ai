# Reading the label and the packing slip

## Per-order data

| Value | Comes from |
|---|---|
| `order_id` | The label text (three-step search below) |
| `tracking_id` | The label text, in one of the tracking-ID forms of `parser.toml` ([courier](courier.md)); on an instant-service label, which prints none, its `pickup_code` ([D-LABEL-24](../decisions/label.md)) |
| `courier`, `courier_clue` | Deduced from the label: the courier and the clue that named it ([courier](courier.md)) |
| `ship_by_date` | The date of the label's `In transit by: dd/mm/yyyy hh:mm`, or of the Shopee-style label's `Batas Kirim: dd-mm-yyyy` ([D-APP-21](../decisions/app-features.md)) |
| `pickup_code` | The instant-service label's code: two capital letters and 4 digits printed in large type beside its `Kode Pengambilan` caption, within 30 pt of its line (the page text does not keep the two together); on the Shopee-style instant label two digits and two capital letters after the caption in its own text run (its label prints it; it is stored as the tracking ID) |
| `account`, `shop` | The listed shop name found in the sender, and the account that lists it ([accounts](../app/accounts.md#the-account-of-an-order)) |
| `area` | The receiver's province, city and district as printed ([below](#the-receivers-area)) |
| `weight` | The parcel's weight as printed, in grams ([below](#the-weight)) |
| item lines | The slip table: product name, variation, Seller SKU, quantity |
| `total_quantity` | The slip's printed `Qty Total`; none on the Shopee-style slip (a filter counts the lines instead: [conditions](../rules-file/conditions.md)) |

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
carries no tracking ID and no courier logo. A **split label** prints its slip on the page after the label: a
label page with no slip, then a page with the same Order ID, no header image, the slip, the `In transit by`
deadline and the label's own tracking ID. That page continues the copy too, and the copy takes its slip and,
when the label page printed none, its deadline ([D-LABEL-12](../decisions/label.md)). A copy's pages are together and are copied unchanged, in their
order. Two copies of one Order ID are a **repeat**, never one longer order
([duplicates](../app/duplicates.md), [D-LABEL-12](../decisions/label.md)).

A page headed `Packing List` is a packing list the shop prints, not a shipping label. The reader stops the batch on it, whatever order it names ([D-LABEL-28](../decisions/label.md)).

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
  a wrap directly after `-`, which joins without one (`FI 2012-` + `2015` → `FI 2012-2015`). The
  Shopee-style instant label (a Shopee slip on a page with `Kode Pengambilan`) breaks a cell at any
  character, so its lines join with nothing between (`Sa` + `mping` → `Samping`); a break that fell on a
  space loses it, and a filter compares these fields without spaces ([D-LABEL-30](../decisions/label.md)).
- **The table's end** is the caption under it — `Qty Total:` on the TikTok and Tokopedia slips, `Pesan:`
  on the Shopee-style slip — so the caption's value and the footer's tracking IDs are not read into the
  last row.
- **Variation** `Default` reads as empty.
- **Cross-check**: when the slip prints `Qty Total:` and it differs from the sum of the copy's `Qty`, a
  warning names the Order ID, the printed total and the sum. The Shopee-style slip prints no total and
  raises none.

## The sender

Read for an order's account ([accounts](../app/accounts.md#the-account-of-an-order)); `orders.csv` keeps the
shop name found, not the text.
Every style prints it with the caption `Pengirim` (`Pengirim :` or `Pengirim:`), in one of four places:

| Style | Where the name is |
|---|---|
| J&T Express, SiCepat, IDX, J&T Cargo | Right of the caption on its baseline, or in the caption's own run |
| Anteraja | Under the caption, about 11 pt lower |
| Instant service | After the caption in its line of the page text (its run comes before the caption's) |
| Shopee-style | Right of the caption on its baseline, a run of its own |

- **The text looked in**: the caption's line of the page text, and the runs right of the caption on its
  baseline (±3 pt) or under it (to 12 pt lower, about its x).
- **A match**: a listed name, spaces and upper and lower case taken out of both, is found in that text with
  its spaces taken out. The page text's spaces are not reliable (`Tok o Cont oh`), and a digit of the
  vertical tracking ID can stick to the name's run (`TokoContoh7`), so a name is found inside the text, not
  compared with it.
- **The name read**, shown by the shop prompt for a sender no account lists: the runs right of the caption on
  its baseline up to a run of one word with six digits or more (a phone number, or the tracking number the
  Shopee-style label prints there), else those under it, else the caption's line of the page text after the
  caption; taken up to a phone number (`(+62)`, `+62`, digits with `*`) or the next caption (a word and
  `:`), a digit stuck to its end taken off, trimmed. Its spaces may be wrong (`Tok o Cont oh`); a match takes spaces out, so it is
  written as read and still finds those labels, and the Accounts screen can put it right.
- The Shopee-style label can print a long name cut short (`Toko Conto`): it is asked about once, and from
  then on the longest listed name found wins.
- **No caption** on the copy's first page: the copy has **no sender**.

## The receiver's area

The province, city and district of the receiver, read from the copy's first page **as printed**: no
spelling put right, no `Kab.` or `Kota` taken off. A part the layout does not print is none, never guessed.
Each layout prints them in its own place, found by its captions:

| Layout | Where | Read as |
|---|---|---|
| TikTok standard (J&T Express, SiCepat) and boxed (IDX, J&T Cargo, Anteraja) | The first line under the `Penerima` caption, in its column (within 2 pt of its x), up to 25 pt lower, that holds two commas | `province,city,district`; the boxed layout prints the name first and goes on with the street |
| Instant (`Alamat :` caption: Gojek, Grab) | The address under `Alamat :`, its lines 11.5 pt apart | The last three parts: `…, district, city, province` |
| Shopee-style (a `Berat:` caption: SPX, Grab Express) | The province ends the address under `Penerima:`, left of the `Pengirim:` column, its lines 7.5 to 8.2 pt apart; the city and the district are the first two texts, left to right, of the line right above `Berat:` (or above its `CASHLESS` note) | Province: the address's last part; city, district: as the line prints them |

- **A wrapped address** breaks a line inside a word too, and the break drops the space between words. A
  line that starts with a capital letter starts a new word; any other is joined without a space
  (`Kalide` / `res` is `Kalideres`).
- **A province** takes the spaces of its name in the list of provinces, its letters as printed, so a break
  inside or between its words reads right (`JA` / `WA TENGAH`, `DKI` / `JAKARTA`). A name not in the list
  stays as read.
- Shopee-style: a word-by-word line (justified) is one line; a tag (`HOME`, `OFFICE`) right of the caption's
  column and a street's hint above the city line are not read; the tracking ID printed upright at the page's
  edge is left of the city line's start. A cell too long for its column wraps onto two lines centred on the
  row (`KOTA JAKARTA` / `SELATAN`); a cell is the texts starting within 40 pt of each other, top first.
- **J&T with an `Alamat :` caption** too: its line under `Penerima` wins; the instant reading is used only
  when that line is missing, so a long address is never read as the area.
- **A long name printed over the area line** (SiCepat): text that jumps back left on one line is read as
  another text, so the area line still starts in the caption's column.
- On the owner's samples (744 orders of the four layouts) every order reads a province, a city and a
  district.

## The weight

The parcel's weight, read from the copy's first page as printed and kept in grams
([D-LABEL-27](../decisions/label.md)). The value is the number on its caption's line (within 2 pt of its
baseline); a caption's word elsewhere, as in a product name, is passed over.

| Layout | Caption | Value |
|---|---|---|
| TikTok standard (J&T Express, SiCepat, IDX, Anteraja) | `Weight :` (its run may start with the date printed left of it) | Kilograms with three decimals: `0.490 KG` is 490 g |
| TikTok boxed (J&T Cargo) | `Weight` | Whole kilograms: `5 KG` is 5000 g |
| Shopee-style (SPX, Grab Express) | `Berat:` | Grams: `450 gr` |

The instant labels (Gojek, Grab) print none, and so do some boxed IDX and Anteraja labels: the weight is
*not printed*. On the owner's samples every other order reads one.
