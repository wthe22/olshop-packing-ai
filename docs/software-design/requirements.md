# Requirements

## The PC app

| Tool | Runs on | Does |
|---|---|---|
| **PC app** | Windows PC, Rust + Tauri window | A shop owner's label PDFs with packing slip → label groups shown and adjusted on screen, one PDF per group and a summary PDF per batch, each verified before it is saved; revert and amend of the newest batch; search and reprint ([PC app](app/README.md), [Screens](ui/README.md)) |

The owner of the app prepares the labels of one or more shop owners; each is a **namespace**
([namespaces](app/namespaces.md)).

## Concepts

| Term | Meaning |
|---|---|
| **Namespace** | One shop owner: a folder of the data folder with one folder per date. On screen an **owner** |
| **Journal** | The one file of a namespace and date that holds every saved fact ([journal](data/journal.md)) |
| **Input PDF** | A label download, stored once by its content: `input/{sha256}.pdf` |
| **Sort rule** | A named list of label group rules in `packing-workflows.toml`; a batch is sorted by one ([rules file](rules-file/README.md)) |
| **Label group** | One entry of a sort rule: code, name and **filter**. Its filter takes orders from what is left; its orders are written as one **group PDF** |
| **Group PDF** | The output of one label group: `{batch}-{code} {name} x{count}.pdf`, its orders' label pages re-ordered by **order content** ([groups](labels/groups.md)) |
| **Summary PDF** | One per batch: the units to take from stock and a page of facts ([summary PDF](labels/summary-pdf.md)) |
| **Order content** | An order's lines as (product name, variation, quantity), sorted. Orders with the same order content are packed the same way and are consecutive in their group PDF |
| **Copy** | One appearance of an order's label in the inputs: its label page and the continuation pages right after it. An order with two or more copies is a **repeat** ([repeated orders](app/duplicates.md)) |
| **Filter** | A condition in the filter language (free AND / OR / NOT), e.g. `name contains "sepatu" and total_quantity = 1` |
| **Order** | Key: the number its label prints (`Order ID`) — 18 digits on the TikTok and Tokopedia labels, 14 characters on the Shopee-style one. One tracking ID and one or more item lines |
| **Item line** | Product, variation, Seller SKU and quantity, from the label's packing slip. An item is its name and variation |
| **Display name** | Product name up to the first `\|`, trimmed; then ` — ` and the variation, left out when empty. Examples: `Spion Beat — Standard, honda`; `Sepatu Standar Samping Motor` |
| **Reprint** | One PDF of chosen saved orders' label pages, printed again ([reprint](app/reprint.md)) |

## Features

Each requirement has a scoped ID `F-<SCOPE>-n` ([scopes](../README.md#scopes)).

### Labels and groups (LABEL, RULES)

- **F-LABEL-1** Read the label PDFs (*Shipping label + Packing slip*); per order take each value from the label
  and its slip ([reading](labels/reading.md)).
- **F-LABEL-2** Find each order's pages and its item lines; an order can span several pages. A page's Order ID
  comes from the label text (three-step search); a page without a label part continues the copy right before it.
- **F-LABEL-3** Build the label groups from the batch's sort rule: each group, in order, takes the matching
  orders from what is left; the last may have no filter and takes the rest; leftovers are a final group.
- **F-LABEL-4** Write one PDF per label group (pages unchanged), its orders re-ordered by order content, sorted
  by (orders descending, order content ascending), inside one order content in reading order. An order already
  saved on the same date of the same owner is left out and warned.
- **F-LABEL-5** Write one A4 summary PDF per batch: the units to take from stock and the facts of the batch.
- **F-LABEL-6** Find the orders that appear more than once in a batch, count them, and let the owner choose
  which copy stays; every order is written once.

### Batch and data (APP)

- **F-APP-1** Stop with a clear message on any problem; nothing is left half-written.
- **F-APP-2** Input: one or more *Shipping label + Packing slip* PDFs per batch; plain labels stop the batch
  with instructions.
- **F-APP-3** One **Save** saves the batch completely or not at all; batch numbers run through a namespace and
  date.
- **F-APP-4** The newest batch of a date can be reverted (undone) or amended (redone in place).
- **F-APP-5** A label file already used on the date is recognised before reading, and one file chosen twice is
  read once.
- **F-APP-7** Every output is checked before it is saved by code independent of the code that wrote it: every
  input page appears exactly once — written or left out on purpose — and every output page is its source page
  unchanged ([verification](app/verification.md)).
- **F-APP-8** One namespace per shop owner, each with its own dates, batches, numbers and guard
  ([namespaces](app/namespaces.md)).
- **F-APP-9** Every saved fact is kept once, in the journal, and every order is traceable to its input, pages
  and courier clue; the files a person reads are written from it ([data](data/README.md)).
- **F-APP-10** Saved orders can be found and their labels printed again in one verified PDF; a lost output is
  written again from the stored inputs ([reprint](app/reprint.md)).

### Screens (UI)

- **F-UI-1** A Windows program; daily use needs no terminal ([Screens](ui/README.md)).
- **F-UI-2** A batch is shown as a plan (the label groups of its sort rule, with counts and order contents) that
  can be changed for this batch only — skip, move, add a one-off group, rename a file — before anything is
  written.
- **F-UI-3** The Day screen lists every batch of the open date with its group PDFs and summary PDF, each with
  an **Open** button for printing and a printed tick.
- **F-UI-4** The sort rules are edited in the app with live checks and counts.
- **F-UI-6** The files that hold repeated orders are marked; a screen shows how many repeats there are, groups
  the files that repeat each other and lets the owner choose which copy stays ([Repeated orders](ui/duplicates.md)).
- **F-UI-7** A side panel names the open owner, lists its dates, and opens another date or owner
  ([Panel](ui/panel.md)).
- **F-UI-8** Search finds saved orders by any filter field, shows each one's trace, and reprints the ticked
  ones ([Search](ui/search.md)).
