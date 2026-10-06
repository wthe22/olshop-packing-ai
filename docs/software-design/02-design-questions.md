# 02 — Design Questions

Open questions only. Each one is undecided until the owner chooses.

### D1 — App stack
Options in [04-app-architecture.md](04-app-architecture.md#stack-options).

### D2 — Who reads the CSV for the app?
The paper packing list shows batch numbers and group codes; the app should show the same.
- a. Both the script and the app read the raw CSV, each with the same rules. Two
  implementations of the import (Python and the app's language). Codes match only if the same
  CSVs are imported in the same order on both.
- b. Only the script reads the CSV. It writes a small batch file (`batch-N.zip`: stripped
  orders, batch number, category, group code) that the app imports. One implementation; the app
  always matches the paper. The batch file must reach the phone (share, USB, cloud folder).

### D3 — Order ID as the key
The platform can split one order into several packages, each with its own tracking ID and
label. The samples never do (1 package per order). Proposal: key on Order ID; the import warns
when one order has more than one tracking ID.

### D4 — Scan modes
- **Inspect**: each scan opens the order card (items, quantities, buyer message) and waits for
  a tap: Checked / Wrong packing / Label / Pending.
- **Fast**: each scan that passes the filter is marked `checked` at once, no tap. For a uniform
  stack (e.g. "sepatu, quantity 1"). Problems are marked afterwards from the recent list or the
  order detail.
- Options: a. both, switchable on the scan screen; b. Fast only; c. Inspect only.

### D5 — The variation `Default`
- a. Hide it (`Sepatu Standar Samping Motor` instead of `… — Default`).
- b. Always show the variation.

### D6 — Reprinting labels
- a. Reprint from the seller centre by Order ID, read from the problems page.
- b. The app exports the problem list; the script makes a reprint PDF from the day's label PDFs.

### D7 — Scanning an order marked wrong packing / label / pending again
After repacking or reprinting, the parcel is scanned again.
- a. In Fast mode it becomes `checked`; the message shows the previous status.
- b. Never automatic; it opens the order card for a tap.

### D8 — Changed order after it was checked
If a newer CSV shows different items, quantities or tracking ID for a checked order:
- a. Back to `unchecked`, with a "changed after check" note.
- b. Keep `checked`, show a warning only.
