# 01 — Requirements

## Two tools

| Tool | Runs on | Does |
|---|---|---|
| **PC script** | Windows PC, Python | Orders CSV + label PDFs → new orders of the batch, pack groups, categories, one label PDF per pack group, A4 packing list |
| **Android app** | Android phone or tablet, portrait and landscape | Sessions and batches, order list, camera scanning to check parcels, marks with undo, session export/import |

Both read the orders CSV. Only the PC script does PDF work.

## Concepts

| Term | Meaning |
|---|---|
| **Session** | One working day of checking. Holds batches, orders and marks. Created, renamed, deleted, exported to a `.zip`, imported again |
| **Batch** | One orders-CSV import, numbered 1, 2, … within the session. Holds the orders that were new in that CSV |
| **Order** | Key: `Order ID`. Has one tracking ID and one or more item lines |
| **Item line** | SKU ID + quantity. Name, variation, Seller SKU and product category are looked up by SKU ID from the newest import |
| **Display name** | Product name up to the first `\|`, trimmed, then ` — ` and the variation. Variation `Default` is shown as blank: `Spion Beat — Standard, honda`, `Sepatu Standar Samping Motor` |
| **Packing list** | The set of (SKU ID, quantity) of an order, ignoring line order |
| **Pack group** | All orders of one batch with the same packing list. Numbered 01, 02, … in print order within the batch; written `2-05` (batch 2, group 05) where the batch is not obvious |
| **Category** | A named rule from an ordered list, reused every day. Every order falls into exactly one; the first matching category wins; the last one ("Lainnya") catches the rest |
| **Condition** | A rule tree with free AND / OR / NOT over order fields. Used by categories and scan filters |
| **Scan filter** | A condition active on the scan screen. A scanned order that does not match is rejected and not marked |
| **Mark** | A check result recorded as an event; every mark can be undone |

## Same numbers on PC and phone

The script and the app compute batches and pack groups the same way from the same CSVs:

- batch N = orders not seen in batches 1 … N−1 of the day;
- print order = categories in list order → groups by number of orders, largest first → ties by
  signature (sorted `SKU ID×quantity` text).

So every orders CSV is imported into both, in the same order. The packing list header shows the
batch's order and group counts; the app's import summary shows the same two numbers to compare.

## Order status

| Status | Meaning |
|---|---|
| `unchecked` | Never marked |
| `checked` | Packed correctly |
| `wrong packing` | Must be repacked, then checked again |
| `label problem` | Label lost or damaged; reprinted from the seller centre, then checked again |
| `pending` | Set aside on purpose, e.g. no stock |
| `removed` | Not in the newest orders CSV any more (cancelled or picked up). Needs no attention; hidden by default; not counted in progress |

The shown status is the latest mark not undone; `removed` overrides it.

## PC script

- **P1** Read the orders CSV. Keep only `Perlu dikirim` / `Menunggu pengambilan` rows; ignore all
  others. Keep only the packing columns (business-process 02).
- **P2** Compare with the earlier batches of the same day: new orders form batch N.
- **P3** Pack groups and categories, from the shared `categories.json`.
- **P4** Read one or more label PDFs. Put each order's pages, unchanged and in their original
  order, into one PDF per pack group, in print order.
- **P5** One A4 packing list for the batch: categories as headings, one row per pack group,
  then a pick summary (units per SKU).
- **P6** Details in [03-pc-script.md](03-pc-script.md).

## Android app

### A1 Import a batch
- Read the orders CSV (same rules as P1–P3).
- Summary before saving: new orders, groups, removed orders, ignored rows.
- Orders whose SKU IDs, quantities or tracking ID differ from the earlier import: shown as a
  warning (not expected to happen). Their status is not changed.
- Buttons: **Apply import** / **Cancel**. Nothing is saved before Apply.

### A2 Scan to check
- Phone camera only. Button to switch camera (back/front, or between back lenses).
- Reads the label QR code or 1D barcode (both hold the tracking ID). Order ID is accepted too.
- Scan filter with free AND/OR. Typical use: "name contains sepatu AND total quantity = 1".
- **A scan that passes the filter marks the order `checked`.** Below the result are buttons to
  change that mark: **Wrong packing** · **Label lost/damaged** · **Pending**. No press = checked.
- A code stays ignored while it keeps being seen; it counts as a new scan only after it has been
  out of view for 2 s. A label held in front of the camera for 10 s gives one scan.
- Scanning an order that is already checked: duplicate warning, nothing changes.
- Scanning an order marked wrong packing, label problem or pending: a warning shows its mark and
  asks **Process** / **Cancel**. Process = the normal scan (marked checked, buttons below);
  Cancel = scan ignored.
- Every result is explained in words: what happened, why, and what to do (05-app-ui).

### A3 Marks and undo
- One tap per mark, no typing: Checked · Wrong packing · Label lost/damaged · Pending.
- The last 5 marks stay on screen with Undo. Older ones: history screen and the order's
  detail, each undoable. A mark made by an accidental scan is undone the same way.
- Undo is itself an event; nothing is erased.

### A4 Order list
- Filter by status, category, pack group, batch; search by tracking ID (also by its last
  digits), Order ID, product text.
- Detail: items with display names (Seller SKU shown small when present), buyer message,
  courier, batch, pack group, mark history, mark buttons.

### A5 Problems page
- Own page listing every order marked `wrong packing`, `label problem` or `pending`, grouped
  by status, with full tracking ID and Order ID in large text, for finding and reprinting the
  label in the seller centre.

### A6 Sessions
- Create (default name = date), rename, export (`.zip`, no PDFs), import, delete.
- Delete always asks for confirmation. Setting: a session created today can be deleted only
  after a set time (default 17:00, can be turned off).
- Nothing is kept of a deleted session.

### A7 Settings
- Categories (D1), saved scan filters, language, delete-protection time, sounds/vibration.
- Settings export/import as a separate `.zip`.
- Language: Indonesian, English, or System (follow the device; English when the device
  language is neither).
