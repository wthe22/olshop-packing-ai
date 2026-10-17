# Requirements

## The PC app

| Tool | Runs on | Does |
|---|---|---|
| **PC app** | Windows PC, Rust + Tauri window | Label PDFs with packing slip → picks shown and adjusted on screen, one saved label PDF per pick, A4 packing lists; revert/amend of the last batch ([PC app](app/README.md), [Screens](ui/README.md)) |

The app packs each batch with the grouping of picks the owner chooses; the groupings live in the data folder as
pick files ([rules file](rules-file/README.md)). The analysis helpers in
`tools/analysis/` are an aid for agents, not a tool of the shop, and share no state with the app.

## Concepts

| Term | Meaning |
|---|---|
| **Pick** | One entry of a grouping. Its condition selects orders from what is left; the match is written as one **saved PDF** |
| **Saved PDF** | The output of one pick: that pick's label pages, re-ordered into **runs** and numbered through the day `1`, `2`, `3`, … ([picks](labels/picks.md)) |
| **Run** | Inside a saved PDF, consecutive orders with identical **contents**; numbered `01`, `02`, … within the saved PDF and written `3-05` (saved PDF 3, run 05) |
| **Contents** | The order written as text: per line `product name` + `variation` + `quantity`, lines sorted. Two orders with the same contents are packed the same way |
| **Category** | One entry of a grouping: the word the screens use for a pick |
| **Grouping** | A named collection of categories, one file each in the data folder; a batch is packed with one grouping ([rules file](rules-file/README.md)) |
| **Condition** | A rule in the condition language (free AND / OR / NOT), e.g. `name contains "sepatu" and total_quantity = 1`. Used by picks |
| **Order** | Key: `Order ID`. Has one tracking ID and one or more item lines |
| **Item line** | Product, variation and quantity, from the label's packing slip. |
| **Display name** | Product name up to the first `\|`, trimmed; then ` — ` and the variation. A variation of `Default` or empty is left out. Examples: `Spion Beat — Standard, honda`; `Sepatu Standar Samping Motor` |

## Features

Each requirement has a scoped ID `F-<SCOPE>-n` ([scopes](../README.md#scopes)).

### Labels and picks (LABEL, RULES)

The reading and sorting rules are in [labels/README.md](labels/README.md); the app-specific decisions are
in [PC app](app/README.md).

- **F-LABEL-1** Read the label PDFs (*Shipping label + Packing slip*); per order take each value from the
  label and its slip.
- **F-LABEL-2** Find each order's pages and its item lines; an order can span several pages. A page's
  Order ID comes from the label text (two-step search); a page without one continues the previous
  order's pages.
- **F-LABEL-3** Build the picks from the categories of the grouping the batch uses (each entry = one pick):
  each entry in file order
  takes the matching orders from what is left; the last entry may have no condition and takes the
  rest; the leftovers are the final "rest" pick.
- **F-LABEL-4** Write one saved label PDF per pick (pages unchanged). Inside it the orders are re-ordered
  into **runs** (identical contents next to each other), runs sorted by (number of orders
  descending, contents text ascending), inside a run the download page order. Saved PDFs are
  numbered through the day `1`, `2`, …, runs inside `01`, `02`, …, written `3-05`. An order already
  saved earlier the same day is left out and warned.
- **F-LABEL-5** Write the A4 packing list when asked (layouts **full**, **summary**, **pick**; scope per
  saved PDF, whole batch, or none).

### Batch and data (APP)

- **F-APP-1** Write `summary.txt`; stop with a clear message on any problem.
- **F-APP-2** Input: one or more *Shipping label + Packing slip* PDFs per batch; plain labels stop
  the batch with instructions. No orders CSV.
- **F-APP-3** One **Save** writes the batch completely or not at all, into
  `labels/<day>/batch <b>/`; numbering continues through the day.
- **F-APP-4** The last batch can be reverted (undone) or amended (redone in place).
- **F-APP-5** A label file already used earlier in the day is recognised before reading.

### Screens (UI)

- **F-UI-1** A Windows program; daily use needs no terminal ([Screens](ui/README.md)).
- **F-UI-2** A batch is shown as a plan (the categories of the batch's grouping in order, with counts and
  runs) that can be
  changed for this batch only — skip, move, add a one-off condition, rename a file — before
  anything is written.
- **F-UI-3** The day screen lists every batch with its saved PDFs and packing lists, each with an
  Open button for printing.
- **F-UI-4** The groupings of categories are edited in the app with live checks and counts.
