# Open questions: labels

Questions: Q-LABEL-1, Q-LABEL-2, Q-LABEL-3, Q-LABEL-4, Q-LABEL-5, Q-LABEL-7, Q-LABEL-8.

## Q-LABEL-1 — Reading a courier logo the encoded bytes cannot name (J&T Cargo)

**Words.** The owner's rule names the courier from the **logo** (its image hash) and the
**tracking-ID form**, both agreeing, and asks the owner once about a logo never seen before. Four
courier images are plain images whose encoded bytes hash (XXH3-64) to one stable value each across
every sample file (J&T Express 975×377, SiCepat 184×60, iD Express/IDX 1200×859; the fourth,
`TikTok Shop` 326×59, is the instant-service label's own logo and names no courier — Q-LABEL-2).
**J&T Cargo's is not one of them**: it is a *masked* image (`/SMask`) whose base is a flat colour
(331 bytes) and whose shape is the mask, and the platform drew the same wordmark at two scales
(680×156 on two sample labels, 683×157 on the third), so the encoded bytes differ between exports.
Measured on the sample labels: hashing the **appearance** instead — composite the image with its
mask, fit into a 64 × 64 canvas on white, threshold at 200/255, XXH3-64 — gives J&T Cargo one value
for **both** rasters (`753ccbd14749befa`), and one distinct value per other courier and per platform
strip. So the logo clue *can* name J&T Cargo; the only question is how much of it to use.

| Option | What the owner sees |
|---|---|
| **1. The appearance, for an image the byte hashes did not name** (**Default**) | J&T Cargo is named by its logo + its `57` form, like every other courier, and a re-rastered logo of a known courier is never asked about again. Costs the image decode only on the pages whose bytes matched nothing (about 1 page in 600) |
| 2. The appearance for every logo | The same table for every courier and one clue order only; a re-made asset of *any* courier is never asked about again, but every page pays the decode |
| 3. J&T Cargo's logo stays unread; `FastTrack` + the `57` form name it | No image work is needed; the cheapest. A Cargo label whose printed `FastTrack` is missing or unreadable stays unknown, and an unseen *masked* logo of another courier is asked about again on every batch |
| 4. Render the label's header and compare pictures (~17 ms a page) | The same result as 1, for the most code and the highest cost |

Recommended: **1** — measured stable on both rasters, it gives J&T Cargo the same two agreeing
clues as the other couriers (the owner's rule), and the cost is paid only where it is needed. The
printed `FastTrack` and the `TT Order ID` caption stay as the fallback clue
([courier deduction](../labels/courier.md)).

**What the answer changes:** the second clue's form (an appearance hash in `couriers.toml`), or
nothing at all.

## Q-LABEL-2 — An instant-service label names no courier (Gojek)

**Words.** The platform has an *Instant / Same Day* service ([business-process 02 › the
instant-service label](../../business-process/02-data-sources.md); Tokopedia's seller guide): its
label is the `TikTok Shop`-branded page with `Kode Pengambilan` + a `WX` + 4-digit pickup code and
the tag `GKS`. It is the only label with **no courier logo and no tracking ID**, so the two clues
that name every other courier are both missing — and it belongs to one of **Gojek, Grab or Blitz**;
the page never says which. The owner names it **Gojek**. It is not in the sample download (no page
of that style among its 604 pages; Tokopedia's rule: a Same Day label can be printed only after the
delivery is arranged). **Everything known about this label comes from one page** — the tag, the
pickup-code form, the missing logo and tracking ID, the clock-time deadline — so each mark must be
re-checked on the next instant label before it is relied on; the sample download is searched for the
caption `Kode Pengambilan` only, so a differently drawn instant label is missed.

| Option | What the owner sees |
|---|---|
| **1. The name comes from the order export** (**Default**) | An instant order is named from its CSV row's `Shipping Provider Name` (Gojek) and marked *instant*, so its packing-list line reads `Gojek (instant) ×1`. With no CSV row the order shows `Gojek (instant)` from the tag `GKS` and also appears in the unknown-courier warning |
| 2. A tag → courier table in `couriers.toml` | `GKS` = Gojek; a later `GRB`/`BLZ` is a file edit, not a program change; the label alone always names the courier, and an unseen tag is asked about once like an unseen logo |
| 3. Ask the owner once per unseen tag, like the logo prompt | Nothing to maintain; the same question returns whenever the platform changes a code |

Recommended: **1**, with 2's table as the fallback for the export-less case (the tag is the only
label-side key, so it may as well be recorded).

**What the answer changes:** `couriers.toml` (a `tag` table or nothing), the courier part of a
packing-list line, and whether an instant order can be named at all.

## Q-LABEL-3 — Where the pickup code shows during packing

**Words.** On an instant-service label the driver asks for the **pickup code** (`Kode Pengambilan`,
`WX` + 4 digits) when collecting the parcel: the owner confirms it with the courier before handing
the parcel over (Tokopedia's guide). The app reads the label page, so it holds the code; the
question is whether it prints it.

| Option | What the owner sees |
|---|---|
| **1. On the packing list, beside the order** (**Default**) | The instant order's line carries its pickup code, so the code the driver asks for is on the sheet the owner carries; nothing else changes |
| 2. On screen only (the pick and the Day screen) | The packing list stays as it is; the code is read from the screen when the driver arrives |
| 3. Nowhere | The code is read from the label page itself |

**What the answer changes:** one field of the packing-list line and the day's data file.

## Q-LABEL-4 — The Shopee-style label: pack it, or leave it alone

**Words.** A 3-page sample download of the Shopee-style label (one order a page) is a label style the
app cannot pack. It is neither a TikTok nor a Tokopedia page: its header carries the `Shopee`
wordmark, its order number is **not** the 18-digit Order ID but its own `No.Pesanan` (14 characters:
six digits then eight letters and digits), its slip's columns are named in Indonesian (`# Nama
Produk SKU Variasi Qty`, where `SKU` holds the seller SKU and `Variasi` the variation — the two
tokens are swapped against the Tokopedia slip), it prints no `Qty Total:` and no `Customer Message`,
and its deadline is `Batas Kirim: dd-mm-yyyy` (a date, with no time). Its courier is deducible like
any other ([courier deduction](../labels/courier.md), item 9: the `SPX` wordmark as the logo plus an
`SPXID` + 11-digit tracking ID). The batch stops on its first page (*page 1 has no Order ID*),
because the pack flow keys every order by the 18-digit Order ID and reads item lines from the slip's
`Product Name` · `SKU` · `Seller SKU` · `Qty` header.

| Option | What the owner sees |
|---|---|
| **1. Pack it, keyed by its own `No.Pesanan`** (**Default**: the batch stops with *page 1 has no Order ID*) | An SPX batch packs like any other: the label reader accepts the 14-character number as that style's key, the slip reader accepts the Indonesian header and reads `Variasi` as the variation, and everything that shows an Order ID shows the 14-character number instead. Cost: a second key and a second slip header in the readers; a download mixing the styles still packs as one batch |
| 2. Leave the style alone, but say so | The batch still stops, with a message that names the style (*These are a Shopee download; this app packs TikTok and Tokopedia labels*), so a mixed download is understood at once and its pages can be printed from the seller centre. Cost: one message |

Recommended: **1** if these orders are packed here too — the label carries everything a pick and a
packing list need (slip lines, tracking ID, courier), and the only new thing is where the order
number comes from.

**What the answer changes:** the label reader's key rule, the slip reader's header rule and the day's
data file; or one error message.

## Q-LABEL-5 — Which image on a page is the courier's logo

**Words.** The courier is named from the page's **logo image**
([courier deduction](../labels/courier.md), item 1: the image's encoded bytes, else its appearance,
hashed and looked up in `couriers.toml`) — but nothing yet says **which** of a page's images that is,
and pages carry several: the courier's logo, the buying channel's strip (`tokopedia | TikTok` 527×73,
`tokopedia | Shop` 905×118, `TikTok Shop` 326×59, `tokopedia PLUS` 618×78, `Shopee` 800×253), a
page-wide frame (2000×2000 or 1000×1000), the slip's footer strip, and — on the Shopee-style label —
its barcode and QR as images of their own. The phrase "the one small *unmasked* image in the header"
does not hold: J&T Cargo's and SPX's logos are both masked, and on the sample download's pages the
smallest *unmasked* images are a barcode and the QR.

Measured on the sample labels over the six styles (the page is 419 pt tall; y counted from the
bottom): the courier's logo and the channel strip are the only images whose **whole box lies in the
page's top 40 pt** — J&T Express 388–415 · SiCepat 390–414 · IDX 390–417 beside its strip 397–408 ·
J&T Cargo 394–413 beside its strip 397–409 · SPX 394–411 beside the `Shopee` strip 394–412 · the
instant label's strip 394–411 — while the frame (134–418), the slip's footer strip (39–62, or 104–124
on the instant label) and the barcodes and the QR (173–224, 330–373) all fall outside it.

| Option | What the owner sees |
|---|---|
| **1. The images in the page's top 40 pt, with *Not a courier* as an answer** (**Default**) | The logo is read from the header band only; a hash there that the table does not know asks once, and the prompt gains a **Not a courier** answer (a channel strip) which the app remembers beside the logo hashes ([D-LABEL-4](../decisions/label.md)) so it is never asked again. The barcodes and the QR are never asked about — they are outside the band |
| 2. A list of known page images in `couriers.toml` | Every frame size and every channel strip measured above is skipped and everything else asks once. Cost: the list grows with each new channel brand, and on a Shopee-style page the barcode and the QR are not in it — three more questions on every order |
| 3. Ask about every image the table does not know | The least code and the noisiest to use: the sample download's first page asks 4 times (its channel strip, two barcodes, the QR) and asks again on the next page, because those codes differ with every order |
| 4. Keep the "small unmasked image in the header" phrase | Nothing changes, and the Shopee-style pages name the wrong clue: the smallest unmasked image on them is a barcode, while J&T Cargo's and SPX's own logos would be skipped for being masked |

Recommended: **1** — measured on all six styles, it asks about a channel strip once ever rather than
once per batch, never asks about a per-order code, and leaves the owner's two-clue rule as it is.

**What the answer changes:** how the app picks the image to hash, the
[logo prompt](../ui/new-batch.md)'s answers and `couriers.toml`'s fields.

## Q-LABEL-7 — Where the remembered logos are kept

**Words.** The app remembers every logo the owner names in the [logo prompt](../ui/new-batch.md), and
every *Not a courier* answer ([D-LABEL-4](../decisions/label.md)). They must be written somewhere in the
data folder.

**Example.** The owner names an unseen SiCepat logo in batch 3. In batch 4 the same logo is named without a
question. The hash is a line in a file the owner may open.

| Option | What the owner sees |
|---|---|
| **1. In `couriers.toml`** (**Default**) | The hash is appended to the named courier's `logo` list; *Not a courier* hashes go to a `not_courier` list at the top. One courier file, and the owner can read or correct what the app remembered |
| 2. In a file of its own in the data folder | `couriers.toml` stays exactly as the owner wrote it; the app writes a second file with the hashes only |

**What the answer changes:** which file the prompt writes to, and what Settings › Change… and Start check.

## Q-LABEL-8 — A page with no header image and a vendor-marked tracking ID

**Words.** The rule names a courier only when the logo and the tracking-ID form agree
([courier deduction](../labels/courier.md)). A real label always has header-band images, so a page
with **none** exists only in the made-up test labels (`testdata/`), which carry no pictures. On such a
page there is no logo to agree with.

**Example.** A page with no header image and the tracking ID `JY0000001234`.

| Option | What the owner sees |
|---|---|
| **1. The form names the courier when it carries the vendor's own marker** (**Default**) | A lettered form (`JY`, `TKP`, `SPXID`) that fits exactly one courier names it; a bare-number form (`00`, `57`) never does. Real labels are never affected, because they always have a header image; the test labels keep their couriers |
| 2. Strict: no logo and no printed text clue, no name | The test labels need pictures added (`testdata/make_testdata.py`, `expected-labels.json`, the Python reference's test), then every page is named by two clues |

**What the answer changes:** `deduce_courier_in` in `crates/engine/src/label.rs` and the test labels.
