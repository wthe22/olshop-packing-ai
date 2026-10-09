# 01 — Requirements

## Two tools

| Tool | Runs on | Does |
|---|---|---|
| **PC app** | Windows PC, Rust + Tauri window | Label PDFs with packing slip → picks shown and adjusted on screen, one saved label PDF per pick, A4 packing lists; revert/amend of the last batch ([07](07-pc-app.md), [08](08-pc-app-ui.md)) |
| *PC script* | Windows PC, Python | *Reference implementation of phase 1, replaced by the PC app:* label PDFs and an optional orders CSV → the same picks from the command line ([03](03-pc-script.md)) |
| **Android app** | Android phone or tablet, portrait and landscape | Sessions and batches, order list, camera scanning to check parcels, marks with undo, session export/import |

The app reads the orders CSV; the PC app does not (all its data comes from the label and its
packing slip). Only the PC side does PDF work. They share one rules file, `categories.toml` ([04-rules-file.md](04-rules-file.md)). There
is no network connection between them; files are moved by hand (USB, cloud folder, chat to
self).

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
| **Session** | One working day of checking in the app. Holds batches, orders and marks. Created, renamed, deleted, exported to a `.zip`, imported again |
| **Batch** | *App, to be revised.* One orders-CSV import, numbered 1, 2, … within the day |
| **Pack group** | *App, to be revised.* All orders of one batch with the same signature |
| **Signature** | *App, to be revised.* The packing list written as text: the order's lines as `SKU ID×quantity`, sorted by SKU ID as text, joined with `+`. Example: `1730000000000000001×1+1730000000000000002×2` |
| **Scan filter** | A condition active on the app's scan screen. A scanned order that does not match is rejected and not marked |
| **Mark** | A check result, recorded as an event. Every mark can be undone |

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

## Order status (app)

| Status | Meaning |
|---|---|
| `unchecked` | Never marked |
| `checked` | Packed correctly |
| `wrong packing` | Wrong or missing item, wrong quantity, extra item. Repacked, then scanned again |
| `label problem` | Label lost or damaged. Reprinted by hand from the seller centre, then scanned again |
| `pending` | Set aside on purpose, e.g. no stock |
| `removed` | Not in the newest orders CSV any more (cancelled or picked up). Needs no attention; hidden by default; not counted in progress |

The shown status is the latest mark that is not undone; `removed` overrides it.

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

## Android app

Not yet revised for the pick flow of the PC script; revised before phase 3.

### A1 Import a batch
- Read the orders CSV with the shared rules.
- Summary before saving: new orders, groups, removed orders, ignored rows, warnings.
- An order whose SKU IDs, quantities or tracking ID differ from the earlier import is listed as
  a warning (not expected to happen). Its status is not changed; the newest values are stored.
- Buttons **Apply import** / **Cancel**. Nothing is saved before Apply.

### A2 Scan to check
- Phone camera only (no handheld scanner). Button to switch camera (back/front, or between
  back lenses).
- Reads the label QR code or 1D barcode (both hold the tracking ID). Order ID is accepted too.
- Scan filter with free AND/OR. Typical use: `display_name contains "sepatu" and
  total_quantity = 1`. Several saved filters; one active filter at a time, editable on the spot.
- **A scan that passes the filter marks the order `checked`.** Below the result are buttons to
  change that mark: **Wrong packing** · **Label lost/damaged** · **Pending**. No press = stays
  checked.
- A code stays ignored while it keeps being seen; it counts as a new scan only after it has
  been out of view for 2 s. A label held in front of the camera for 10 s gives one scan.
- Scanning an order that is already `checked`: duplicate warning, nothing changes.
- Scanning an order marked wrong packing, label problem or pending: a warning shows its mark and
  asks **Process** / **Cancel**. Process = the normal scan (marked checked, change buttons
  below); Cancel = scan ignored.
- Every result is explained in words: what happened, why, and what to do
  ([06-app-ui.md](06-app-ui.md#scan-messages)).
- Manual entry: type the last digits of the tracking ID when a code cannot be read.

### A3 Marks and undo
- One tap per mark, no typing: Checked · Wrong packing · Label lost/damaged · Pending.
- The last 5 marks stay on the scan screen with Undo. Older ones: history screen and the
  order's detail, each undoable. A mark made by an accidental scan is undone the same way.
- Undo is itself an event; nothing is erased.

### A4 Order list
- Filter by status, category, batch, pack group; search by tracking ID (also by its last
  digits), Order ID, product text.
- Detail: items with display names (Seller SKU shown small when present), buyer message,
  courier, batch, pack group, mark history, mark buttons.

### A5 Problems page
- Every order marked `wrong packing`, `label problem` or `pending`, grouped by status, with
  full tracking ID and Order ID in large text, for finding the order in the seller centre.

### A6 Sessions
- Create (default name = date), rename, export (`.zip`, no PDFs), import, delete.
- Delete always asks for confirmation. Setting: a session created today can be deleted only
  after a set time (default 17:00, can be turned off).
- Nothing is kept of a deleted session.

### A7 Settings
- Categories: edit in the app, or import/export `categories.toml` to edit on the PC. The same
  file moves both ways.
- Saved scan filters, language, delete-protection time, sounds and vibration.
- Settings export/import as one `.zip` (categories + filters + options).
- Language: Indonesian, English, or System (follow the device; English when the device
  language is neither).
