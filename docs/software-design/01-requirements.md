# 01 — Requirements

## Two tools

| Tool | Runs on | Does |
|---|---|---|
| **PC script** | Windows PC, Python | Orders CSV + label PDFs → new orders of the batch, categories, pack groups, one label PDF per pack group, A4 packing list |
| **Android app** | Android phone or tablet, portrait and landscape | Sessions and batches, order list, camera scanning to check parcels, marks with undo, session export/import |

Both read the orders CSV. Only the PC script does PDF work. They share one rules file,
`categories.toml` ([04-rules-file.md](04-rules-file.md)). There is no network connection
between them; files are moved by hand (USB, cloud folder, chat to self).

## Concepts

| Term | Meaning |
|---|---|
| **Session** | One working day of checking in the app. Holds batches, orders and marks. Created, renamed, deleted, exported to a `.zip`, imported again. The script's equivalent is the day folder |
| **Batch** | One orders-CSV import, numbered 1, 2, … within the day. Holds the orders that were new in that CSV |
| **Order** | Key: `Order ID`. Has one tracking ID and one or more item lines |
| **Item line** | SKU ID + quantity. Name, variation, Seller SKU and product category are looked up by SKU ID from the newest import |
| **Display name** | Product name up to the first `\|`, trimmed; then ` — ` and the variation. A variation of `Default` or empty is left out. Examples: `Spion Beat — Standard, honda`; `Sepatu Standar Samping Motor` |
| **Signature** | The packing list written as text: the order's lines as `SKU ID×quantity`, sorted by SKU ID as text, joined with `+`. Example: `1730000000000000001×1+1730000000000000002×2` |
| **Pack group** | All orders of one batch with the same signature. Numbered 01, 02, … in print order within the batch. Written `2-05` (batch 2, group 05) where the batch is not obvious |
| **Category** | One entry of `categories.toml`. Every order falls into exactly one: the first category whose condition matches |
| **Condition** | A rule in the condition language (free AND / OR / NOT), e.g. `name contains "sepatu" and total_quantity = 1`. Used by categories and by the app's scan filters |
| **Scan filter** | A condition active on the app's scan screen. A scanned order that does not match is rejected and not marked |
| **Mark** | A check result, recorded as an event. Every mark can be undone |

## Shared batch and group rules (script and app give the same numbers)

1. Keep only CSV rows with `Order Status = Perlu dikirim` and
   `Order Substatus = Menunggu pengambilan`. Ignore every other row (count them).
2. Group the kept rows by `Order ID`. All rows of one order share the tracking ID.
3. **Batch N** = orders of this CSV that were not in batches 1 … N−1 of the same day.
4. **Category** of an order = first category in `categories.toml` whose condition matches.
5. **Pack groups** = orders of batch N with the same signature.
6. **Group numbers**: sort the groups by (category position in the file, number of orders
   descending, signature text ascending) and number them 01, 02, … in that order. Numbering
   runs through all categories (category A may have 01–04, category B 05–14).
7. **Orders inside a group**: by `RTS Time` ascending, then `Order ID` ascending. This is the
   order of the label pages and of the tracking IDs on the packing list.

So every orders CSV of the day is imported into both, in the same order. The packing list
header shows the batch's order count and group count; the app's import summary shows the same
two numbers so a mismatch is seen at once.

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

- **P1** Read one orders CSV (rules 1–2). Keep only the packing columns
  ([business-process 02](../business-process/02-data-sources.md)).
- **P2** Find batch N by comparing with the day's earlier batches (rule 3).
- **P3** Categories and pack groups from `categories.toml` (rules 4–7).
- **P4** Read one or more label PDFs. Write one PDF per pack group, in print order, containing
  that group's label pages unchanged. An order's pages stay together and in their original
  order (an order can span several pages).
- **P5** Write the packing list in any of three A4 layouts: **full** (groups with their
  tracking IDs, then pick summary), **summary** (groups without tracking IDs, then pick
  summary), **pick** (pick summary only). Chosen per run; several at once allowed.
- **P6** Print a summary on screen and stop with a clear message on any problem.
- Details: [03-pc-script.md](03-pc-script.md).

## Android app

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
