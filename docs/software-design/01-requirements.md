# 01 — Requirements

## Two tools

| Tool | Runs on | Does |
|---|---|---|
| **PC script** | Windows PC, Python | Orders CSV + label PDFs → new orders of the batch, pack groups, categories, one label PDF per pack group, A4 packing list |
| **Android app** | Android phone or tablet, portrait and landscape | Sessions and batches, order list, camera scanning to check parcels, marks with undo, session export/import |

The Android app does no PDF work.

## Concepts

| Term | Meaning |
|---|---|
| **Session** | One working day of checking. Holds batches, orders and marks. Created, renamed, deleted, exported to a `.zip`, imported again |
| **Batch** | One orders-CSV import, numbered 1, 2, … within the session. Holds the orders that were new in that CSV |
| **Order** | Key: `Order ID`. Has one tracking ID and one or more item lines |
| **Item line** | SKU ID + quantity. Name, variation, Seller SKU and product category are looked up by SKU ID from the newest import |
| **Display name** | Product name up to the first `\|`, trimmed, then the variation: `Spion Beat — Standard, honda` |
| **Packing list** | The set of (SKU ID, quantity) of an order, ignoring line order |
| **Pack group** | All orders of the session with the same packing list. Code such as `A01`, kept in every batch of the session |
| **Category** | A named rule from an ordered list, reused every day. Every order falls into exactly one; the first matching category wins; the last one ("Lainnya") catches the rest |
| **Condition** | A rule tree with free AND / OR / NOT over order fields. Used by categories and scan filters |
| **Scan filter** | A condition active on the scan screen. A scanned order that does not match is rejected and not marked |
| **Mark** | A check result recorded as an event; every mark can be undone |

## Order status

| Status | Meaning |
|---|---|
| `unchecked` | Never marked |
| `checked` | Packed correctly |
| `wrong packing` | Must be repacked, then checked again |
| `label problem` | Label lost or damaged; reprint at the PC, then check again |
| `pending` | Set aside on purpose, e.g. no stock |
| `removed` | Not in the newest orders CSV any more (cancelled or picked up). Needs no attention; hidden by default; not counted in progress |

The shown status is the latest mark not undone; `removed` overrides it.

## PC script

- **P1** Read the orders CSV. Keep only `Perlu dikirim` / `Menunggu pengambilan` rows; ignore all
  others. Keep only the packing columns (business-process 02).
- **P2** Compare with the earlier batches of the same day: new orders form batch N.
- **P3** Pack groups and categories (rules file shared with the app if D16-a).
- **P4** Read one or more label PDFs. Put each order's pages, unchanged and in their original
  order, into one PDF per pack group. Groups are printed largest first (D9).
- **P5** One A4 packing list for the batch (D10-a): categories as headings, one row per pack
  group, plus a pick summary (units per SKU).
- **P6** Details in [03-pc-script.md](03-pc-script.md).

## Android app

### A1 Import a batch
- Source per D16. Show the result before saving: new orders, removed orders, changed orders
  (items, quantities or tracking ID), rows ignored because of status.
- A changed order that was already checked goes back to `unchecked`, with the reason shown.

### A2 Scan to check
- Phone camera only. Button to switch camera (back/front, or between back lenses).
- Reads the label QR code or 1D barcode (both hold the tracking ID). Order ID is accepted too.
- Scan filter with free AND/OR (D8). Typical use: "name contains sepatu AND total quantity = 1".
- A code stays ignored while it keeps being seen; it counts as a new scan only after it has been
  out of view for 2 s. A label held in front of the camera for 10 s gives one scan.
- Scanning an order that is already checked gives a duplicate warning and changes nothing.
- Every result is explained in words: what happened, why, and what to do (05-app-ui).
- Scan modes, see D18.

### A3 Marks and undo
- Quick buttons: **Checked** · **Wrong packing** · **Label lost/damaged** · **Pending**. No typing.
- The last 5 marks stay on screen with Undo. Older ones: history screen and the order's
  detail, each undoable. A mark created by an accidental scan is undone the same way.
- Undo is itself an event; nothing is erased.

### A4 Order list
- Filter by status, category, pack group, batch; search by tracking ID (also by its last
  digits), Order ID, product text.
- Detail: items with display names (Seller SKU shown small when present), buyer message,
  courier, batch, pack group, mark history, mark buttons.

### A5 Problems page
- Own page listing every order marked `wrong packing`, `label problem` or `pending`, grouped
  by status, with full tracking ID and Order ID in large text, so the orders are easy to find
  at the PC.

### A6 Sessions
- Create (default name = date), rename, export (`.zip`, no PDFs), import, delete.
- Delete always asks for confirmation. Setting: a session created today can be deleted only
  after a set time (default 17:00, can be turned off).
- Nothing is kept of a deleted session.

### A7 Settings
- Saved scan filters (global), language, delete-protection time, sounds/vibration.
- Settings export/import as a separate `.zip`.
- Language: Indonesian, English, or System (follow the device; English when the device
  language is neither).
