# 03 — Reading the Labels and Making the Picks

The rules by which the PC app ([07-pc-app.md](07-pc-app.md)) reads a label download and turns it
into picks, saved PDFs and packing lists: the Order ID of a page, the packing slip, the courier,
the picks, the runs and the numbering, and the look of the packing list. They were first built
and tested in the Python tools (`tools/analysis/`, frozen — an analysis aid, not part of the app).
Where this file and 07 differ, 07 is the design. Requirements: P1–P6 in
[01-requirements.md](01-requirements.md); a **pick** selects orders from what is left and each
pick's orders are written as one **saved PDF** of label pages (see *Concepts* in
[01-requirements.md](01-requirements.md#concepts)). All order data comes from the label and its
packing slip.

## Per-order data

| Value | Comes from |
|---|---|
| `Order ID` | The label text (two-step search below) |
| `tracking_id` | The label text |
| `courier` | Deduced from the label (below) |
| `ship_by` | The label `In transit by: dd/mm/yyyy hh:mm` |
| item lines | The slip table (product name, variation, quantity) |
| buyer message | The slip's `Customer Message` (continuation page) |

### Order ID from the label

On the page text, first search `(?<!\d)(5\d{17})(?!\d)`; if nothing is found, drop all
whitespace and search `OrderI[dD][:：](\d{18})` (the second colon is fullwidth U+FF1A). The raw
search must come first: on J&T labels the Order ID sits on its own line under the tracking ID,
and dropping whitespace first would glue the two into one number.

### Multi-page orders

A page with no Order ID belongs to the previous page's order; a first page with no Order ID is
an error (file and page). Consecutive pages with the same Order ID are one order (a slip's
continuation page repeats the Order ID, the table header and `Customer Message`). Label pages
are copied unchanged, so an order's pages stay together and in their original order.

## Reading the packing slip

The slip is read by text position, not by reading order: each text run of the page comes with
its x and y ([07 › Text runs from PDFium](07-pc-app.md#text-runs-from-pdfium)).

- **Columns** from the header words `Product Name`, `SKU`, `Seller SKU`, `Qty`: each word's x is
  a column's left edge; a column ends at the next header word's x (the last at the page's right
  edge). The `SKU` column holds the **variation**.
- **Rows**: sorted by y, a row starts at each `Qty` value. The runs inside a column's x-range at
  or after the row's y (and before the next row) are that cell. A wrapped cell joins its lines
  with a space, except a wrap directly after `-`, which joins without a space
  (`FI 2012-` + `2015` → `FI 2012-2015`).
- **Cross-check**: the slip prints `Qty Total:`; if it differs from the sum of the order's `Qty`,
  a warning is shown (Order ID, printed total, sum).

## Courier deduction

The label is read for two clues and the courier is named **only when the two agree** (the owner's
rule, 2026-10-11): one clue alone can be wrong, so a wrong name would need two labels to be wrong
in the same way.

1. **The logo** — the main clue. Every label prints the courier's logo as an embedded image; the
   image's **encoded bytes** are hashed (pdfium's `FPDFImageObj_GetImageDataRaw`, *not* the decoded
   pixels) with **XXH3-64** and looked up in the courier data's logo table. One hash per courier,
   measured stable across every sample export. The cost is about 0.5 µs a page (the bytes are
   1.5–15 KB; measured 0.44 µs for a 14 KB logo). Python's `xxhash` (in `tools/analysis`) and Rust's
   `xxhash-rust` compute the same value, so a hash taken there can be checked against the app's. A hash
   not in the table is **asked of the owner once** ("which courier is this logo?") and the answer
   is written into the data file, so later labels are named without asking.
2. **The tracking-ID form** ([business-process 02](../../business-process/02-data-sources.md#columns-packing-needs)):
   `JY` + 10 digits → J&T Express; `00` + 10 digits → SiCepat; `57` + 10 digits → J&T Cargo;
   `TKP` + 10 digits → IDX.
3. **Named only when the two agree**: the tracking form gives the *candidate* couriers, and the
   courier is named when the logo's courier is among them. A logo whose courier and form disagree,
   a logo with no form, a form with no logo, or neither → no name. Unknown → `courier` empty, and
   a warning lists those orders.
4. **A printed text clue** (`www.jet.co.id`, `FastTrack`) stands in for the logo only where the
   logo cannot be read — J&T Cargo's logo is a masked image with no stable hash (D18). SiCepat and
   IDX print no courier name at all.
5. **The service tag is not used yet** (the owner, 2026-10-11: it is weak for some couriers). The
   measured tag lists stay in [business-process 02](../../business-process/02-data-sources.md) for
   when a case needs them.

The courier list is `couriers.toml` (in the data folder in the app), and it also holds the logos
the owner names in the prompt; its fields and an example are in
[07 › `couriers.toml`](07-pc-app.md#courierstoml).

## Picks

- **Default**: the `categories.toml` entries in file order, each one pick. The last entry may
  have no `when` and takes the rest. A pick takes the matching orders **from what is left**
  after the earlier picks.
- **Leftovers**: orders matching no pick stay in "the rest" and are saved as a final PDF.
- **Duplicate guard**: an order already saved earlier the same day (the day's `state.json`) is left out of
  the saved PDFs and listed in a warning with its Order ID, tracking ID and the time it was
  first saved.

## A saved PDF

- **Re-ordering**: inside a saved PDF the orders are grouped into **runs** of identical contents
  and printed run by run. The contents key is per line `product name` + `variation` + `quantity`,
  lines sorted. Runs are sorted by (number of orders
  descending, contents text ascending); inside a run the orders stay in download page order.
- **Pages**: all pages of each order, copied unchanged.

## Numbering and file names

- **Saved PDFs** are numbered through the day `1`, `2`, `3`, …; a later batch the
  same day continues the numbering from the day's `state.json`.
- **Runs** inside a saved PDF are numbered `01`, `02`, … in printed order. A run is written
  `<pdf>-<run>`, e.g. `3-05` (saved PDF 3, run 05), in the packing list.
- **Saved-PDF file name**: `<n> <code> <name> ×<orders>.pdf`, e.g. `3 Z Lainnya ×393.pdf`.
  Characters not allowed in Windows file names (`\ / : * ? " < > |`) become `-`; the name is cut
  at 100 characters (ending in `…`) to stay clear of the Windows path limit. The number prefix
  makes "sort by name" equal printing order.

## Packing list (A4)

- The layouts `full`, `summary`, `pick` keep their look. A heading is the saved PDF number and
  the pick: `1  A  Sepatu`. The groups under a heading are the runs, numbered `3-05`; in `full`
  each run's tracking IDs are printed as before.
- Scope, chosen per batch: `per-pdf` (one sheet per saved PDF), `whole` (one sheet with a
  section per saved PDF), `none`.
- A packing list needs the slips' item lines; every page of a batch carries a slip
  ([07 › Every page must carry a packing slip](07-pc-app.md#every-page-must-carry-a-packing-slip)).
- Header `Packing list · <day> · <n>  <code>  <name>` (scope `per-pdf`) or
  `Packing list · <day> · saved PDFs <first>-<last>` (scope `whole`), then the counts
  (`<orders> orders · <units> units · <runs> runs · printed hh:mm`) and `page x/y`.
- The header takes the top 23 mm: the title (13 pt bold) and `page x/y` in the first 7 mm, the
  counts line (9.5 pt) in the next 5 mm, then a 1 mm gap. **The body starts at 23 mm on every
  page** — the first page and after every page break, the `(continued)` line included. A body row
  at the top margin is printed over the title and the counts (review round 1).
- Fonts: Arial 9.5–13 and Consola 10 from `C:\Windows\Fonts`, embedded whole. PDFium's *non-CID*
  font path writes `× ÷ é ® ™ ° ± ñ Ä ½ €` as `ÿ`, so those characters are drawn with a **CID copy
  of the same font**; the CID font's own text layer turns a plain space into U+00A0 and `-` into
  U+00AD, so it carries those characters only and the rest of the text keeps a clean text layer.
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
