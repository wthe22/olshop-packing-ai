# Steps 1–5: from a session to orders

## 1. Open a session

- The app finds the data folder ([Start](../ui/start.md)), then opens **today's Main** session
  ([sessions](../app/sessions.md)); the [Sessions panel](../ui/sessions.md) opens any other.
- It loads the session's `state.json` ([data-folder](../app/data-folder.md)); no file means an empty
  session. Everything below reads and writes this one session only.

**Keeps:** I6 (numbers come from this session's `state.json`).

## 2. Choose the files

- **Add files…** or a drop gives paths. Each path is read quickly: name, size, modified time, **page
  count** and a **fingerprint** (XXH3-64 of the file's bytes).
- A path already listed is listed once. The list is **sorted by file name** — the download's own name
  starts with its time and part number, so name order is download order.
- A file whose fingerprint is in a batch of this session shows *used in batch n*: it can be read; its
  orders are then found as already saved (step 6). A fingerprint equal to another listed file shows
  *same file as n* ([New batch](../ui/new-batch.md)).
- Two chosen files with the same name are refused: the batch's `download/` folder keeps files by
  name, so a second one would overwrite the first.

**Stops and warnings:** a file that cannot be opened stops the batch ([Messages](../ui/messages.md)).

**Check:** is name order always the order the owner wants? (A file renamed by hand changes the
download position of its orders, which sets the order inside a run.)

## 3. Read the pages

A worker thread opens the files in list order and reads every page with a progress bar and **Cancel**;
nothing is written. A page gives ([reading](../labels/reading.md)):

| Value | How |
|---|---|
| **Order ID** | Three searches in order on the page text: an 18-digit number starting with 5; `OrderId:` + 18 digits with whitespace removed; `No.Pesanan:` + 14 characters |
| **Tracking ID** | The parcel number the label prints, in a form of the data folder's `couriers.toml` |
| **Courier** | Named only when the logo hash and the tracking-ID form agree ([courier](../labels/courier.md)) |
| **Ship by** | `In transit by: dd/mm/yyyy hh:mm`, or `Batas Kirim: dd-mm-yyyy` |
| **Slip lines** | The table read by text position: product name, variation, seller SKU, quantity per row, and `Qty Total` |
| **Header images** | The images drawn in the top 40 pt, hashed for the logo and for the page fingerprint |
| **Fingerprint** | The page fingerprint ([reading](../labels/reading.md#page-identity)) |

**Stops:** a page with an Order ID but **no slip table** — the batch stops and names the files and
pages (plain labels, [inputs](../app/inputs.md)); the first page of a file with no Order ID.

**Check:** the three searches find the Order ID of every style the shop receives (TikTok,
Tokopedia, Shopee-style); a new style with another caption is not found and stops the batch.

## 4. Copies and orders

Pages are walked once, in list order, page order ([duplicates](../app/duplicates.md#1-copies)):

1. A page with an Order ID and a label part (tracking ID or header image) **starts a copy**.
2. A page with the Order ID and no label part, or with no Order ID at all, **continues** a copy.
3. A copy's values: the tracking ID of its first page and the first ship-by found; the item lines of
   every slip table of its pages, in page order; the `Qty Total` and the customer message of its
   last slip that has one. Orders are numbered 1, 2, 3, … by their first appearance in the files:
   the **download position**, which keeps the order inside a run.
4. Copies are grouped into **orders** by Order ID. An order with one copy is normal; with two or more
   it is a **repeat** (step 7).

**Keeps:** I1. A page that fits no copy — the first page of a file with no Order ID — stops the
batch with its file and page.

**Check:** a copy is told from a continuation by its content ([Q-LABEL-9](../questions/duplicates.md)).

## 5. Checks and the logo prompt

- **`Qty Total` against the lines** of each copy: a difference is a warning with the Order ID, the
  printed total and the sum. The Shopee-style slip prints no total and is never checked.
- **Unknown courier** and **no tracking ID** are warnings: the order is sorted normally and only a
  condition on `courier` or `tracking_id` cannot see it.
- **A logo not in `couriers.toml`** opens *Which courier is this logo?* before the plan; the answer is
  written to the file, so it is asked once ([New batch](../ui/new-batch.md)).

**Check:** these three are warnings, not stops; the owner may disagree for *no tracking ID*.
