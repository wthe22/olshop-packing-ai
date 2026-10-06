# Packing Checker — Documentation

Two tools for the daily packing work of the shop (TikTok Shop + Tokopedia orders, shipped by
J&T, SiCepat, IDX):

- a **PC script** (Python) that turns the orders CSV and the shipping-label PDFs into one label
  PDF per pack group and an A4 packing list;
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
| **Pack group** | All orders of one batch with the same packing list |
| **Category** | An owner-defined rule ("name contains sepatu") that splits pack groups into sections, e.g. Sepatu / Spion & Knalpot / Lainnya |
| **Batch** | The new orders of one orders-CSV export. The first export of the day is batch 1; each later export adds batch 2, 3, … |
| **Session** | One day of checking in the app: its batches, orders and marks |
| **Mark** | A check result on an order in the app: checked, wrong packing, label lost/damaged, pending |

`samples/` holds real platform files. It is **confidential and never committed**. Docs and
committed test data use made-up IDs only (`JY0000001234`, `580000000000000001`).
