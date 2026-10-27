# Steps 3–6: from chosen files to orders

## 3. Choose the files

- **Add files…** or a drop gives paths. Each file is read quickly: name, size, times, **page count** and
  **SHA-256** of its bytes.
- One file chosen twice — the same path, or the same bytes under another name — is listed once, with a
  note naming the other ([D-APP-10](../decisions/app.md)).
- A file whose SHA-256 is an input of a live batch of this date shows *used in batch n*: it can be read; its
  orders are then found as already saved (step 7). On an amend's selection page the batch being redone is
  never that batch: its own files, chosen again from anywhere, are the same files.
- The list is **sorted by file name**: the download's name starts with its time and part number, so name
  order is download order. This is the **reading order** of the batch.
- The **sort rule** is chosen here: the rule of the workspace's newest batch is preselected
  ([D-RULES-1](../decisions/rules.md)).

**Stops:** a file that cannot be opened as a PDF.

**Check:** is name order always the order the owner wants? (A file renamed by hand moves its orders inside
their order contents.)

## 4. Read the pages

The worker thread opens the files in reading order and reads every page, with a progress bar and
**Cancel**; nothing is written. A page gives ([reading](../labels/reading.md)):

| Value | How |
|---|---|
| **Order ID** | Three searches in order: an 18-digit number starting with 5; `OrderId:` + 18 digits with whitespace removed; `No.Pesanan:` + 14 characters |
| **Label part** | A tracking ID, or an image in the header band (top 40 pt) |
| **Tracking ID** | A token matching a form of `packing-workflows.toml`, fully |
| **Courier and clue** | Logo bytes, else appearance, against the tracking form; text where no image can be read; the tag on an instant label ([courier](../labels/courier.md)) |
| **Ship by** | The date of `In transit by: dd/mm/yyyy hh:mm` or `Batas Kirim: dd-mm-yyyy` |
| **Pickup code** | `Kode Pengambilan` on an instant label |
| **Slip** | The table by text position: name, variation, Seller SKU, quantity per row; `Qty Total` |
| **Fingerprint** | Text runs with positions and header-image hashes ([reading](../labels/reading.md#page-fingerprint)) |

**Check:** the three searches find the Order ID of every style the shop receives (TikTok, Tokopedia,
Shopee-style); a new style with another caption is not found and stops the batch (step 5).

## 5. Copies and orders

Pages are walked once, in reading order, page order ([duplicates](../app/duplicates.md#1-copies)):

1. A page with an Order ID and a label part **starts a copy** of that order.
2. A page with no label part **continues** the copy of the page before it in the same input; an Order ID
   on it must be that copy's.
3. A copy's values: the tracking ID, courier and ship-by of its first page; the item lines of every slip
   table of its pages, in page order; the `Qty Total` of its last slip that prints one.
4. Copies are grouped into **orders** by Order ID. One copy is normal; two or more is a **repeat** (step 8).

**Stops:** the first page of an input that would continue a copy; a continuation page with another order's
Order ID; a page that starts a copy and has no slip table (plain labels). Each names the input and page.

**Keeps:** I1 — every page is in exactly one copy, every copy in one order, a copy's pages consecutive.

**Check:** a copy is told from a continuation by its content ([D-LABEL-12](../decisions/label.md)).

## 6. Checks and the logo prompt

- **`Qty Total`** against the lines of each copy: a difference is a warning with the Order ID, the printed
  total and the sum. The Shopee-style slip prints no total and is never checked.
- **Unknown courier** and **no tracking ID** are warnings: the order is sorted normally; only a filter on
  `courier` or `tracking_id` cannot see it.
- **An unseen header image** — a candidate whose hash is in no list — opens *Which courier is this logo?*
  before the plan. A courier or *Not a courier* is written into `packing-workflows.toml` at once, so it is
  asked once ever; *I don't know* writes nothing ([New batch](../ui/new-batch.md#the-logo-prompt)).

**Check:** these are warnings, not stops; the owner may disagree for *no tracking ID*.
