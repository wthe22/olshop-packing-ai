# 01 — Requirements (PC app)

## The PC app

| Tool | Runs on | Does |
|---|---|---|
| **PC app** | Windows PC, Rust + Tauri window | Label PDFs with packing slip → picks shown and adjusted on screen, one saved label PDF per pick, A4 packing lists; revert/amend of the last batch ([07](07-pc-app.md), [08](08-pc-app-ui.md)) |

Only the PC side does PDF work. The PC app and the Android app share one rules file,
`categories.toml` ([04-rules-file.md](../../shared/04-rules-file.md)); how the PC and the phone
relate is in [`shared/README.md`](../../shared/README.md). The Python programs in
[`tools/analysis/`](../../../tools/analysis/README.md) are an analysis aid for agents, not a tool
of the shop.

## Concepts

| Term | Meaning |
|---|---|
| **Pick** | One entry of `categories.toml`. Its condition selects orders from what is left; the match is written as one **saved PDF** |
| **Saved PDF** | The output of one pick: that pick's label pages, re-ordered into **runs** and numbered through the day `1`, `2`, `3`, … ([03-labels-and-picks.md](03-labels-and-picks.md)) |
| **Run** | Inside a saved PDF, consecutive orders with identical **contents**; numbered `01`, `02`, … within the saved PDF and written `3-05` (saved PDF 3, run 05) |
| **Contents** | The order written as text: per line `product name` + `variation` + `quantity`, lines sorted. Two orders with the same contents are packed the same way |
| **Category** | One entry of `categories.toml`: the word the screens use for a pick |
| **Condition** | A rule in the condition language (free AND / OR / NOT), e.g. `name contains "sepatu" and total_quantity = 1`. Used by picks and by the app's scan filters |
| **Order** | Key: `Order ID`. Has one tracking ID and one or more item lines |
| **Item line** | Product, variation and quantity, from the label's packing slip. (The Android app reads the orders CSV instead, which also has the SKU ID) |
| **Display name** | Product name up to the first `\|`, trimmed; then ` — ` and the variation. A variation of `Default` or empty is left out. Examples: `Spion Beat — Standard, honda`; `Sepatu Standar Samping Motor` |

## PC app

**Pick flow** — [03-labels-and-picks.md](03-labels-and-picks.md); the app-specific decisions are in
[07-pc-app.md](07-pc-app.md#what-the-app-decides):

- **P1** Read the label PDFs (*Shipping label + Packing slip*); per order take each value from the
  label and its slip.
- **P2** Find each order's pages and its item lines; an order can span several pages. A page's
  Order ID comes from the label text (two-step search); a page without one continues the previous
  order's pages.
- **P3** Build the picks from `categories.toml` (each entry = one pick): each entry in file order
  takes the matching orders from what is left; the last entry may have no condition and takes the
  rest; the leftovers are the final "rest" pick.
- **P4** Write one saved label PDF per pick (pages unchanged). Inside it the orders are re-ordered
  into **runs** (identical contents next to each other), runs sorted by (number of orders
  descending, contents text ascending), inside a run the download page order. Saved PDFs are
  numbered through the day `1`, `2`, …, runs inside `01`, `02`, …, written `3-05`. An order already
  saved earlier the same day is left out and warned.
- **P5** Write the A4 packing list when asked (layouts **full**, **summary**, **pick**; scope per
  saved PDF, whole batch, or none).
- **P6** Write `summary.txt`; stop with a clear message on any problem.

In addition:

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
