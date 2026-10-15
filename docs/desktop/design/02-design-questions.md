# 02 — Design Questions (PC app)

Questions that need the owner's choice. Each explains the problem, gives options with an
example of what you would see, and marks one **Built for now**: what the code does until the
owner decides (so development is not blocked). Add new ones as the next free `D` number; remove
one once its answer is written into the other documents.

## D13 — Unticking the rest row

**Words.** The *rest row* is the last row of the Plan screen ([08 › 3. Plan](08-pc-app-ui.md#3-plan)):
the last `categories.toml` entry when it has no condition (e.g. *Z Lainnya*), else
`? Uncategorised`. Unticking **Use** on a row skips that pick for this batch: its orders fall to
the rows below. The rest row has no row below it.

**Example.** Batch 3 has 256 orders: 130 Sepatu, 20 Spion & Knalpot, 106 for *Z Lainnya*. You
untick *Z Lainnya*.

| Option | What you see |
|---|---|
| **1. The 106 orders go to `? Uncategorised`** (**Built for now**) | A new last row `? Uncategorised` with 106 orders; the batch still saves all 256 orders |
| 2. The rest row has no tick | *Z Lainnya* is always used; to leave orders out, change the conditions |
| 3. The 106 orders are left out of this batch | They are not saved; they count as "not saved today" and come back in the next batch with the same labels |

**What the answer changes:** the plan count in `plan.rs` and the Plan screen (task 2.10).

## D14 — A log file

**Words.** [08 › 6. Settings](08-pc-app-ui.md#6-settings) has an *About* row with
**Open log file**. A log file would be a text file in the data folder where the app writes
what it did and any unexpected error (e.g. `log.txt`: "07:40:12 batch 1 saved, 3 files, 600
orders"). Nothing else in the design writes one; every batch already has its `summary.txt`.

| Option | What you see |
|---|---|
| **1. No log file** (**Built for now**) | *About* shows the version and the PDFium version only; errors are shown on screen |
| 2. `log.txt` in the data folder | One line per batch saved, reverted or amended, plus unexpected errors; **Open log file** opens it |

**What the answer changes:** the *About* row of Settings and one small writer in the app.

## D18 — Reading a courier logo the encoded bytes cannot name (J&T Cargo)

**Words.** The owner's rule (2026-10-11) names the courier from the **logo** (its image hash) and
the **tracking-ID form**, both agreeing, and asks the owner once about a logo never seen before.
Four courier images are plain images whose encoded bytes hash (XXH3-64) to one stable value each
across every sample file (J&T Express 975×377, SiCepat 184×60, iD Express/IDX 1200×859; the fourth,
`TikTok Shop` 326×59, is the instant-service label's own logo and names no courier — D21). **J&T Cargo's is not one of them**: it is a *masked* image (`/SMask`) whose base is a
flat colour (331 bytes) and whose shape is the mask, and the platform drew the same wordmark at two
scales (680×156 on two sample labels, 683×157 on the third), so the encoded bytes differ between
exports. Measured 2026-10-13: hashing the **appearance** instead — composite the image with its
mask, fit into a 64 × 64 canvas on white, threshold at 200/255, XXH3-64 — gives J&T Cargo one value
for **both** rasters (`2a36acec58f5…`), and one distinct value per other courier and per platform
strip. So the logo clue *can* name J&T Cargo; the only question is how much of it to use.

| Option | What you see |
|---|---|
| **1. The appearance, for an image the byte hashes did not name** (**Built for now**) | J&T Cargo is named by its logo + its `57` form, like every other courier, and a re-rastered logo of a known courier is never asked about again. Costs the image decode only on the pages whose bytes matched nothing (about 1 page in 600 today) |
| 2. The appearance for every logo | The same table for every courier and one clue order only; a re-made asset of *any* courier is never asked about again, but every page pays the decode |
| 3. J&T Cargo's logo stays unread; `FastTrack` + the `57` form name it | Nothing to build for the image; the cheapest. A Cargo label whose printed `FastTrack` is missing or unreadable stays unknown, and an unseen *masked* logo of another courier is asked about again on every batch |
| 4. Render the label's header and compare pictures (~17 ms a page) | The same result as 1, for the most code and the highest cost |

Recommended: **1** — measured stable on both rasters, it gives J&T Cargo the same two agreeing
clues as the other couriers (the owner's rule), and the cost is paid only where it is needed. The
printed `FastTrack` and the `TT Order ID` caption stay as the fallback clue (03 › *Courier
deduction*).

**What the answer changes:** the second clue's form (an appearance hash in `couriers.toml`), or
nothing at all.

## D20 — How the app writes a taught logo into `couriers.toml`

**Words.** When the owner names an unseen logo in the prompt ([08 › the logo prompt](08-pc-app-ui.md#the-logo-prompt)),
the answer is written into `couriers.toml` — a file the owner also edits by hand (D19, answered
2026-10-11: one file, no second one). The only question left is *how* the app writes it, so that a
hand edit is never lost.

| Option | What you see |
|---|---|
| **1. A `[logo]` table appended at the end of the file** (**Built for now**) | The app adds one line (`"hash" = "SiCepat"`) at the end and never touches the entries above it, so your own comments and order survive. A logo it could only read by **appearance** (D18) goes into a `[shape]` table the same way. The same courier's logos then live in two places — its `logo` list and this table |
| 2. The hash is added to that courier's own `logo` list | One place per courier, but the app rewrites the entry you wrote — `toml_edit` keeps your comments and order, so only that line changes |
| 3. Nothing is written | Simplest: the answer names that batch's orders only, and the same logo is asked about again next batch |

Recommended: **1** — the app only ever appends, so nothing you typed can be lost.

**What the answer changes:** what the app does after the owner names a logo (append a line, edit a
list, or nothing).

## D17 — The check of the saved PDFs: every batch or only a large one

**Words.** Task 2.18 makes the app read its own output back before a batch is finished: every order
exactly once, in one saved PDF, page for page, plus the packing list's counts. Errors (a page
missing, a page twice, an order in two PDFs or in none, a page holding another order's label) and
warnings (the pages are all there, only out of order) are listed separately, *Save anyway* is
allowed, and what was found stays on the day's batch card. The check costs about 1–2 s on the
owner's biggest day (600 pages, 5 saved PDFs).

**Example.** A normal morning batch is 2 files, about 100 pages, 100 orders — well under a second.

| Option | What you see |
|---|---|
| **1. Every batch** (**Built for now**) | Every *Save batch* runs the check; on the 600-page day the save takes 1–2 s longer, on a normal batch nothing noticeable |
| 2. Only a batch above a size | Batches under, say, 200 pages save without the check and the day says *not checked*: small batches stay quick, but a bug that only hits small batches is never seen |

**What the answer changes:** one condition around the check in `pdf/src/batch.rs` and one line on
the Day card. Both options keep the whole check for the big batch.

## D21 — An instant-service label names no courier (Gojek)

**Words.** The platform has an *Instant / Same Day* service ([business-process 02 › the
instant-service label](../../business-process/02-data-sources.md); Tokopedia's seller guide,
2026-08-03): its label is the `TikTok Shop`-branded page with `Kode Pengambilan` + a `WX` + 4-digit
pickup code and the tag `GKS`. It is the only label with **no courier logo and no tracking ID**, so
the two clues that name every other courier are both missing — and it belongs to one of **Gojek,
Grab or Blitz**; the page never says which. The owner reported it as **Gojek** (2026-10-13). It is
not in the 2026-10-13 download (no page of that style among its 604; Tokopedia's rule: a Same Day
label can be printed only after the delivery is arranged). **Everything known about this label comes
from that one page** — the tag, the pickup-code form, the missing logo and tracking ID, the
clock-time deadline — so each mark must be re-checked on the next instant label before it is relied
on; the download was swept for the caption `Kode Pengambilan` only, so a differently drawn instant
label would have been missed.

| Option | What you see |
|---|---|
| **1. The name comes from the order export** (**Built for now**) | An instant order is named from its CSV row's `Shipping Provider Name` (Gojek) and marked *instant*, so its packing-list line reads `Gojek (instant) ×1`. With no CSV row the order shows `Gojek (instant)` from the tag `GKS` and also appears in the unknown-courier warning |
| 2. A tag → courier table in `couriers.toml` | `GKS` = Gojek; a later `GRB`/`BLZ` is a file edit, not a program change; the label alone always names the courier, and an unseen tag is asked about once like an unseen logo |
| 3. Ask the owner once per unseen tag, like the logo prompt | Nothing to maintain; the same question returns whenever the platform changes a code |

Recommended: **1**, with 2's table as the fallback for the export-less case (the tag is the only
label-side key, so it may as well be recorded).

**What the answer changes:** `couriers.toml` (a `tag` table or nothing), the courier part of a
packing-list line, and whether an instant order can be named at all.

## D22 — Where the pickup code shows during packing

**Words.** On an instant-service label the driver asks for the **pickup code** (`Kode Pengambilan`,
`WX` + 4 digits) when collecting the parcel: the owner confirms it with the courier before handing
the parcel over (Tokopedia's guide). The app reads the label page, so it holds the code; the
question is whether it prints it.

| Option | What you see |
|---|---|
| **1. On the packing list, beside the order** (**Built for now**) | The instant order's line carries its pickup code, so the code the driver asks for is on the sheet the owner carries; nothing else changes |
| 2. On screen only (the pick and the Day screen) | The packing list stays as it is; the code is read from the screen when the driver arrives |
| 3. Nowhere | The code is read from the label page itself, as today |

**What the answer changes:** one field of the packing-list line and the day's data file.

## D23 — The Shopee-style label: pack it, or leave it alone

**Words.** `samples/2026-10-15 spx.pdf` (3 pages, one order a page, downloaded 2026-10-15) is a label
style the app cannot pack. It is neither a TikTok nor a Tokopedia page: its header carries the
`Shopee` wordmark, its order number is **not** the 18-digit Order ID but its own `No.Pesanan` (14
characters: six digits then eight letters and digits), its slip's columns are named in Indonesian
(`# Nama Produk SKU Variasi Qty`, where `SKU` holds the seller SKU and `Variasi` the variation — the
two tokens are swapped against the Tokopedia slip), it prints no `Qty Total:` and no
`Customer Message`, and its deadline is `Batas Kirim: dd-mm-yyyy` (a date, with no time). Its courier
is deducible like any other ([03 › *Courier deduction*](03-labels-and-picks.md#courier-deduction),
item 9: the `SPX` wordmark as the logo plus an `SPXID` + 11-digit tracking ID). Today the batch stops
on its first page (*page 1 has no Order ID*), because the pack flow keys every order by the 18-digit
Order ID and reads item lines from the slip's `Product Name` · `SKU` · `Seller SKU` · `Qty` header.

| Option | What you see |
|---|---|
| **1. Pack it, keyed by its own `No.Pesanan`** (**Built for now**: the batch stops with *page 1 has no Order ID*) | An SPX batch packs like any other: the label reader accepts the 14-character number as that style's key, the slip reader accepts the Indonesian header and reads `Variasi` as the variation, and everything that shows an Order ID shows the 14-character number instead. Cost: a second key and a second slip header in the readers; a download mixing the styles still packs as one batch |
| 2. Leave the style alone, but say so | The batch still stops, with a message that names the style (*These are a Shopee download; this app packs TikTok and Tokopedia labels*), so a mixed download is understood at once and its pages can be printed from the seller centre as today. Cost: one message |

Recommended: **1** if these orders are packed here too — the label carries everything a pick and a
packing list need (slip lines, tracking ID, courier), and the only new thing is where the order
number comes from.

**What the answer changes:** the label reader's key rule, the slip reader's header rule and the day's
data file; or one error message.

## D24 — Which image on a page is the courier's logo

**Words.** The courier is named from the page's **logo image**
([03 › *Courier deduction*](03-labels-and-picks.md#courier-deduction), item 1: the image's encoded
bytes, else its appearance, hashed and looked up in `couriers.toml`) — but nothing yet says **which**
of a page's images that is, and pages carry several: the courier's logo, the buying channel's strip
(`tokopedia | TikTok` 527×73, `tokopedia | Shop` 905×118, `TikTok Shop` 326×59, `tokopedia PLUS`
618×78, `Shopee` 800×253), a page-wide frame (2000×2000 or 1000×1000), the slip's footer strip, and —
on the Shopee-style label — its barcode and QR as images of their own. The phrases used so far ("the
one small *unmasked* image in the header") do not hold: J&T Cargo's and SPX's logos are both masked,
and on the `2026-10-15 spx.pdf` pages the smallest *unmasked* images are a barcode and the QR.

Measured 2026-10-15 over the six styles (the page is 419 pt tall; y counted from the bottom): the
courier's logo and the channel strip are the only images whose **whole box lies in the page's top
40 pt** — J&T Express 388–415 · SiCepat 390–414 · IDX 390–417 beside its strip 397–408 · J&T Cargo
394–413 beside its strip 397–409 · SPX 394–411 beside the `Shopee` strip 394–412 · the instant
label's strip 394–411 — while the frame (134–418), the slip's footer strip (39–62, or 104–124 on the
instant label) and the barcodes and the QR (173–224, 330–373) all fall outside it.

| Option | What you see |
|---|---|
| **1. The images in the page's top 40 pt, with *Not a courier* as an answer** (**Built for now**) | The logo is read from the header band only; a hash there that the table does not know asks once, and the prompt gains a **Not a courier** answer (a channel strip) which is written beside the logo table so it is never asked again. The barcodes and the QR are never asked about — they are outside the band |
| 2. A list of known page images in `couriers.toml` | Every frame size and every channel strip measured above is skipped and everything else asks once. Cost: the list grows with each new channel brand, and on a Shopee-style page the barcode and the QR are not in it — three more questions on every order |
| 3. Ask about every image the table does not know | The least code and the noisiest to use: the first `2026-10-15 spx.pdf` page asks 4 times (its channel strip, two barcodes, the QR) and asks again on the next page, because those codes differ with every order |
| 4. Keep the "small unmasked image in the header" phrase | Nothing to build, and the Shopee-style pages name the wrong clue: the smallest unmasked image on them is a barcode, while J&T Cargo's and SPX's own logos would be skipped for being masked |

Recommended: **1** — measured on all six styles, it asks about a channel strip once ever rather than
once per batch, never asks about a per-order code, and leaves the owner's two-clue rule as it is.

**What the answer changes:** how the app picks the image to hash, the logo prompt's answers
([08 › the logo prompt](08-pc-app-ui.md#the-logo-prompt)) and `couriers.toml`'s fields.
