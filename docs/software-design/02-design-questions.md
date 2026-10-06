# 02 — Design Questions

## Decided

| # | Question | Decision |
|---|---|---|
| D1 | Cancellations | Cancelled and picked-up orders simply disappear from the export; neither needs attention. They become `removed` |
| D2 | Orders made ready on earlier days | Imported like any other order: if it is in the export, it must be sent |
| D3 | Where labels are processed | On a PC, by a small Python script. The Android app does no PDF work |
| D4 | Devices | One device, no network. Moving to another device = session export/import |
| D5 | What is checked | Quantity is easy to check; contents often are not. Scan filters reject orders outside the stack being checked (e.g. quantity ≠ 1, other items) |
| D6 | Separating groups | One label PDF per pack group; the printer stops at the end of each; the next group is printed after tearing |
| D7 | Categories | Free rules, e.g. "sepatu"; "not sepatu AND (spion OR knalpot)"; "remaining motorcycle items". Ordered list, first match wins, last one catches the rest |
| D8 | Combining conditions | Free AND / OR (and NOT) |
| D9 | Order of groups | Largest group first |
| D10 | Packing list | A4, one document per batch, categories as headings |
| D11 | Deleted sessions | Nothing kept. Delete needs confirmation; optional "today's session deletable only after 17:00" |
| D12 | Item display | Display name = product name before the first `\|` + variation; Seller SKU optional. Only SKU ID is stable |
| D13 | Mark reasons | Checked · wrong packing · label lost/damaged · pending. One tap each, no details. Wrong ones on their own page. Duplicate scans warn |
| D15 | App language | Indonesian, English, System; English as fallback |

Also decided: only the `Perlu dikirim` / `Menunggu pengambilan` status is handled; label pages
are never modified; camera only (no hardware scanner); session files are `.zip` without PDFs;
timestamps named `*_time`.

## Open

### D14 — App stack
Options in [04-app-architecture.md](04-app-architecture.md#stack-options).

### D16 — Who reads the CSV for the app?
The paper packing list shows batch numbers and group codes; the app should show the same.
- a. Both the script and the app read the raw CSV, each with the same rules. Two
  implementations of the import (Python and the app's language). Codes match only if the same
  CSVs are imported in the same order on both.
- b. Only the script reads the CSV. It writes a small batch file (`batch-N.zip`: stripped
  orders, batch number, category, group code) that the app imports. One implementation; the app
  always matches the paper. The batch file must reach the phone (share, USB, cloud folder).

### D17 — Order ID as the key
The platform can split one order into several packages, each with its own tracking ID and
label. The samples never do (1 package per order). Proposal: key on Order ID; the import warns
when one order has more than one tracking ID. To confirm.

### D18 — Scan modes
- **Inspect**: each scan opens the order card (items, quantities, buyer message) and waits for
  a tap: Checked / Wrong packing / Label / Pending.
- **Fast**: each scan that passes the filter is marked `checked` at once, no tap. For a uniform
  stack (e.g. "sepatu, quantity 1"). Problems are marked afterwards from the recent list or the
  order detail.
- Options: a. both, switchable on the scan screen; b. Fast only; c. Inspect only.

### D19 — The variation `Default`
- a. Hide it (`Sepatu Standar Samping Motor` instead of `… — Default`).
- b. Always show the variation.

### D20 — Reprinting labels
- a. Reprint from the seller centre by Order ID, read from the problems page.
- b. The app exports the problem list; the script makes a reprint PDF from the day's label PDFs.

### D21 — Scanning an order marked wrong packing / label / pending again
After repacking or reprinting, the parcel is scanned again.
- a. In Fast mode it becomes `checked`; the message shows the previous status.
- b. Never automatic; it opens the order card for a tap.

### D22 — Changed order after it was checked
If a newer CSV shows different items, quantities or tracking ID for a checked order:
- a. Back to `unchecked`, with a "changed after check" note (current proposal).
- b. Keep `checked`, show a warning only.
