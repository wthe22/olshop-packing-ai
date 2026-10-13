# 01 — Requirements (Android app)

The Android app checks every packed parcel by scanning its label. It does no PDF work: reading
the labels, the picks and the packing lists are on the PC ([desktop requirements](../../desktop/design/01-requirements.md)).
How the two tools relate (no network, a shared rules file, files moved by hand) is in
[`shared/README.md`](../../shared/README.md).

## Concepts

Order, item line, display name, contents, condition and category are as in
[the desktop requirements](../../desktop/design/01-requirements.md#concepts). Terms of the app:

| Term | Meaning |
|---|---|
| **Session** | One working day of checking in the app. Holds batches, orders and marks. Created, renamed, deleted, exported to a `.zip`, imported again |
| **Batch** | *App, to be revised.* One orders-CSV import, numbered 1, 2, … within the day |
| **Pack group** | *App, to be revised.* All orders of one batch with the same signature |
| **Signature** | *App, to be revised.* The packing list written as text: the order's lines as `SKU ID×quantity`, sorted by SKU ID as text, joined with `+`. Example: `1730000000000000001×1+1730000000000000002×2` |
| **Scan filter** | A condition active on the app's scan screen. A scanned order that does not match is rejected and not marked |
| **Mark** | A check result, recorded as an event. Every mark can be undone |

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
