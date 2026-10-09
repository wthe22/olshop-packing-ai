# Packing Checker — Documentation

Two tools for the daily packing work of the shop (TikTok Shop + Tokopedia orders, shipped by
J&T, SiCepat, IDX):

- a **PC app** (Windows, Rust) that turns the shipping-label PDFs with packing slip into
  picks — one saved label PDF per pick — and A4 packing lists (a Python **PC script** with the
  same rules is the reference implementation);
- an **Android app** (Kotlin) that checks every packed parcel by scanning its label with the
  phone camera.

| Folder | Contains | Read when |
|---|---|---|
| [`business-process/`](business-process/) | How the packing work is done, and the files it uses. True even without the tools | You need to know *why* or what the real data looks like |
| [`software-design/`](software-design/) | What the tools do and how they are built | You build or change a tool |
| [`development/`](development/) | Setup, repository layout, build plan with done-criteria, progress | You start or continue development |

Rule: a statement that would still be true if the tools were never built belongs in
`business-process/`. A screen, field, rule or library belongs in `software-design/`. How to set
up, build and test belongs in `development/`.

## Words used everywhere

| Word | Meaning |
|---|---|
| **Order** | One platform order, identified by its 18-digit `Order ID`. Has one tracking ID and one or more item lines |
| **Tracking ID** | The courier's parcel number, e.g. `JY0000001234`. Printed on the label as text, 1D barcode and QR code |
| **Item line** | One `SKU ID` with a quantity inside an order |
| **Packing list** | The set of (SKU ID, quantity) of one order. Two orders with the same set are packed the same way |
| **Pick** | One rule in `categories.toml`; the PC app (and script) writes the orders it matches as one saved label PDF |
| **Saved PDF** | The label PDF of one pick, numbered through the day (1, 2, 3, …) |
| **Run** | Inside a saved PDF, consecutive orders with identical contents; numbered 01, 02, … and written `3-05` |
| **Pack group** | *App.* All orders of one batch with the same packing list; revised with the app |
| **Category** | An owner-defined rule ("name contains sepatu"): a pick in the PC script, a section heading in the app |
| **Batch** | One round of arrange shipment → download → prepare, numbered 1, 2, 3, … within the day. *PC app:* one label download prepared and saved together (`labels/<day>/batch 2/`). *Android app (to be revised):* the new orders of one orders-CSV export |
| **Session** | One day of checking in the app: its batches, orders and marks |
| **Mark** | A check result on an order in the app: checked, wrong packing, label lost/damaged, pending |

`samples/` holds real platform files. It is **confidential and never committed**. Docs and
committed test data use made-up IDs only (`JY0000001234`, `580000000000000001`).
