# 01 — Requirements (PC app and PC script)

## The PC tools

| Tool | Runs on | Does |
|---|---|---|
| **PC app** | Windows PC, Rust + Tauri window | Label PDFs with packing slip → picks shown and adjusted on screen, one saved label PDF per pick, A4 packing lists; revert/amend of the last batch ([07](07-pc-app.md), [08](08-pc-app-ui.md)) |
| *PC script* | Windows PC, Python | *Reference implementation of phase 1, replaced by the PC app:* label PDFs and an optional orders CSV → the same picks from the command line ([03](03-pc-script.md)) |

Only the PC side does PDF work. Both tools share one rules file, `categories.toml`
([04-rules-file.md](../../shared/04-rules-file.md)); how the PC and the phone relate is in
[`shared/README.md`](../../shared/README.md).

## Concepts

| Term | Meaning |
|---|---|
| **Pick** | One entry of `categories.toml` used by the PC script. Its condition selects orders from what is left; the match is written as one **saved PDF** |
| **Saved PDF** | The output of one pick: that pick's label pages, re-ordered into **runs** and numbered through the day `1`, `2`, `3`, … ([03-pc-script.md](03-pc-script.md)) |
| **Run** | Inside a saved PDF, consecutive orders with identical **contents**; numbered `01`, `02`, … within the saved PDF and written `3-05` (saved PDF 3, run 05) |
| **Contents** | The order written as text: per line `product name` + `variation` + `quantity`, lines sorted. The same with or without a CSV. Two orders with the same contents are packed the same way |
| **Category** | One entry of `categories.toml`: the app's name for what the script calls a pick |
| **Condition** | A rule in the condition language (free AND / OR / NOT), e.g. `name contains "sepatu" and total_quantity = 1`. Used by picks and by the app's scan filters |
| **Order** | Key: `Order ID`. Has one tracking ID and one or more item lines |
| **Item line** | Product, variation, quantity and (from the CSV) SKU ID. Comes from the CSV, else the label's packing slip |
| **Display name** | Product name up to the first `\|`, trimmed; then ` — ` and the variation. A variation of `Default` or empty is left out. Examples: `Spion Beat — Standard, honda`; `Sepatu Standar Samping Motor` |

## PC script rules (pick flow)

1. A page's Order ID comes from the label text (two-step search, [03-pc-script.md](03-pc-script.md));
   a page without one continues the previous order's pages.
2. Item lines come from the CSV when given, else the label's packing slip; the label gives the
   label-level fields (tracking ID, courier, `ship_by`). **The CSV wins** over the slip, and
   each difference is warned.
3. An order already saved earlier the same day is left out and warned
   ([03-pc-script.md](03-pc-script.md)).
4. **Contents** = per line product name + variation + quantity, lines sorted; the same with or
   without a CSV.
5. **Picks**: each `categories.toml` entry in file order takes the matching orders from what is
   left; the last entry may have no condition and takes the rest; the leftovers are the final
   "rest" pick.
6. Inside a saved PDF the orders are re-ordered into **runs** (identical contents next to each
   other), runs sorted by (number of orders descending, contents text ascending), inside a run
   the download page order.
7. Saved PDFs are numbered through the day `1`, `2`, …; runs inside `01`, `02`, …; written
   `3-05`.

Details: [03-pc-script.md](03-pc-script.md).

## PC script

- **P1** Read the label PDFs (required) and the optional orders CSV; per order fill each value
  from the CSV, else the slip/label ([03-pc-script.md](03-pc-script.md)).
- **P2** Find each order's pages and its item lines; an order can span several pages.
- **P3** Build the picks from `categories.toml` (each entry = one pick), taken from what is
  left; the leftovers are the final "rest" pick. Interactive on request.
- **P4** Write one saved label PDF per pick (pages unchanged), re-ordered into runs and numbered
  through the day; leave out orders already saved today (warning).
- **P5** Write the A4 packing list when asked and item data is present (layouts **full**,
  **summary**, **pick**; scope per saved PDF, whole invocation, or none).
- **P6** Print a screen summary and `summary.txt`; warn when there is no item data; stop with a
  clear message on any problem.
- Details: [03-pc-script.md](03-pc-script.md).

## PC app

P2–P6 apply as in the script, with the changes listed in
[07-pc-app.md](07-pc-app.md#what-carries-over-from-the-script-what-changes). In addition:

- **U1** A Windows program; daily use needs no terminal ([08-pc-app-ui.md](08-pc-app-ui.md)).
- **U2** Input: one or more *Shipping label + Packing slip* PDFs per batch; plain labels stop
  the batch with instructions. No orders CSV.
- **U3** A batch is shown as a plan (the rules in order, with counts and runs) that can be
  changed for this batch only — skip, move, add a one-off condition, rename a file — before
  anything is written.
- **U4** One **Save** writes the batch completely or not at all, into
  `labels/<day>/batch <b>/`; numbering continues through the day.
- **U5** The last batch can be reverted (undone) or amended (redone in place).
- **U6** The day screen lists every batch with its saved PDFs and packing lists, each with an
  Open button for printing.
- **U7** `categories.toml` is edited in the app with live checks and counts.
- **U8** A label file already used today is recognised before reading.
