# Decisions: labels

Where a record names a former question (`Q-…`), the owner accepted its default.

## D-LABEL-2 — A courier is named only when two clues agree

**Decision.** An order's courier is named from the **logo** (its image hash) and the **tracking-ID form**, and
only when they agree. Otherwise it stays empty and the batch warns.

**Why.** Some couriers print their name only as a logo image, and two couriers share a tracking-ID length;
neither clue alone is safe.

**Applies to.** [courier deduction](../labels/courier.md).

## D-LABEL-4 — The app remembers the images the owner names

**Decision.** A logo named in the logo prompt, and an image answered *Not a courier*, is remembered; the app
never asks about the same image again. Where: D-LABEL-10.

**Why.** Asking again for an image already answered is work the owner should do once.

**Applies to.** [courier deduction](../labels/courier.md#the-logo-prompt-and-what-is-remembered), [New batch](../ui/new-batch.md#the-logo-prompt).

## D-LABEL-5 — The Shopee-style label is packed, keyed by its own order number

**Decision.** A Shopee-style label packs like any other. Its order is keyed by the 14-character number of its
`No.Pesanan` caption; its slip's Indonesian header names the same cells in another order (`SKU` holds the
Seller SKU, `Variasi` the variation); its `Batas Kirim` is its ship-by date.

**Why.** The label carries everything a group and a summary need; only the order's number comes from elsewhere.

**Applies to.** [reading](../labels/reading.md), [courier deduction](../labels/courier.md).

## D-LABEL-6 — A masked logo is named by its appearance

**Decision.** A header image whose encoded bytes are in no list is hashed by its 64 × 64 appearance and looked up
in `shapes` (Q-LABEL-1).

**Why.** J&T Cargo's masked wordmark is re-rastered between exports; its appearance gave one stable value for
both rasters, and the decode is paid only where the bytes named nothing.

**Applies to.** [courier deduction](../labels/courier.md), item 2.

## D-LABEL-9 — The logo candidates are the images in the page's top 40 pt

**Decision.** Only images whose box lies in the page's top 40 pt can be a logo, and the logo prompt has a **Not
a courier** answer (Q-LABEL-5).

**Why.** Measured on all six styles: that band holds exactly the courier's logo and the channel strip; the
frame, barcodes and QR fall outside, so a per-order code is never asked about.

**Applies to.** [courier deduction](../labels/courier.md), item 1.

## D-LABEL-10 — The remembered images live in parser.toml

**Decision.** A named logo is appended to its courier's `logos`; a *Not a courier* hash to `not_courier.logos`
(Q-LABEL-7).

**Why.** One rules file, and the owner can read or correct what the app remembered.

**Applies to.** [rules file](../rules-file/README.md#couriers).

## D-LABEL-11 — A page with no header image is named by a lettered form

**Decision.** On a page with no header image, a lettered tracking-ID form that fits one courier (`JY`, `TKP`,
`SPXID`) names it; a bare-number form (`00`, `57`) never does (Q-LABEL-8).

**Why.** Real labels always have a header image; the test labels keep their couriers.

**Applies to.** [courier deduction](../labels/courier.md), item 4.

## D-LABEL-12 — A copy starts at a page with a label part, and its pages are together

**Decision.** A page with an Order ID and a tracking ID or header image starts a copy; a page without continues
the copy right before it in the same input (Q-LABEL-9). A continuation page anywhere else stops the batch. One
exception, the split label: a page right after a one-page copy that had no slip, with its Order ID, no header
image, a slip and no tracking ID but the copy's own, continues that copy.

**Why.** A second copy right after the first is then still a repeat. Measured: every multi-page order of the
samples has its second page right after its first, and page ranges make every order traceable. The split J&T
label (`2026-11-25 split.pdf`, the owner's rare case) prints the tracking ID beside its slip; a repeat's
second copy never follows a copy without a slip, as a download either has slips on every label or on none.

**Applies to.** [duplicates](../app/duplicates.md#1-copies), [reading](../labels/reading.md#copies-and-multi-page-orders).

## D-LABEL-13 — Two page identities, one for each job

**Decision.** The reader's **page fingerprint** (text, positions, header-image hashes) compares the copies of a
repeat (Q-LABEL-10). Whether an output page is its source page is the verifier's **page digest** (D-APP-12).

**Why.** The fingerprint is free while reading; the proof of an output must not come from the code that made it.

**Applies to.** [reading](../labels/reading.md#page-fingerprint), [verification](../app/verification.md#the-page-digest).

## D-LABEL-29 — The clue that named an order's courier is not stored

**Decision.** A saved order keeps its source page, not the clue that named its courier; to trace a courier
named wrong, the page is read again. It replaces D-LABEL-14.

**Why.** The owner's answer to Q-APP-26 (2026-11-27): A. The source page is the trace; the clue is read
from it again.

**Applies to.** [courier deduction](../labels/courier.md#the-clue-that-named-it).

## D-LABEL-15 — Orders with the same items are one order content

**Decision.** Inside a group PDF, the orders with the same **order content** — their item lines as (product
name, variation, quantity, Seller SKU) — are printed next to each other, and the screens call such a block an *order
content* (Q-LABEL-11). The Seller SKU is part of each item, so orders whose items differ only in Seller SKU are two contents. The printed `Qty Total` is stored with the order, not with its
content, so one order content is exactly one block. The screens list a group's order contents in printing
order; the summary PDF numbers them ([D-LABEL-19](summaries.md#d-label-19--the-summary-pdf-one-row-everywhere-couriers-by-name)).

**Why.** It is the owner's own word and the data already names it (`order_contents`, `order-contents.toml`).

**Applies to.** [groups](../labels/groups.md#a-group-pdf), [03 Plan](../workflow/03-plan.md#10-order-contents-and-file-names), [files](../data/files.md), [Plan](../ui/plan.md).

## D-LABEL-21 — An instant label is named by its tag; its header images are never asked about

**Decision.** A courier's `instant_tags` name the courier of an instant-service label, which has no logo and no
tracking ID: `GKS` is Gojek, `Sameday` and `Same-day` are Grab. Its header images are the buying channel's strips: none of them
is asked about in the logo prompt. A tag in no list is asked about once in the logo prompt. It replaces
D-LABEL-7.

**Why.** Nothing else on the page names the courier, and a new tag is a file edit, not a program change. The
prompt once showed an instant label's `TikTok Shop` strip, the owner answered *Grab*, and from then on no
TikTok instant label of Gojek or Grab could be named (a courier logo on an instant label names nothing).

**Applies to.** [courier deduction](../labels/courier.md), item 7; [rules file](../rules-file/README.md#couriers).

## D-LABEL-24 — An instant label's pickup code is its tracking ID

**Decision.** An order whose label prints no tracking ID and a pickup code under `Kode Pengambilan` takes the
pickup code as its tracking ID (R-LABEL-16): `BC0000` on the TikTok Shop instant label, `12AB` on the
Shopee-style one, as the owner chose (Q-LABEL-15). The pickup code has no field of its own: the tracking ID is the only one. The courier is named before,
from the label as printed (D-LABEL-21), so an instant label is still known by its missing tracking ID.

**Why.** The owner's request: the courier's rider asks for that code, so Search and a `tracking_id` filter
must find it, and an instant order no longer shows *no tracking ID*. (R-LABEL-18).

**Applies to.** [reading](../labels/reading.md#per-order-data), [courier](../labels/courier.md), [files](../data/files.md).

## D-LABEL-25 — The regular SPX layout's badge is an SPX logo

**Decision.** The `REGULAR SERVICE` badge image of the regular SPX layout (bytes `c622a9c866f1abf4`) is
in the bundled `spx` logos. With the `CM` + 11-digit form it names SPX by the two-clue rule (D-LABEL-2), as
the wordmark does on the other SPX layouts (Q-LABEL-14).

**Why.** The layout prints no SPX wordmark; the badge is the one header image of the page that is not a
buying channel's strip. The owner chose it for now; a second layout carrying the same badge for another
courier would show as a logo and form that disagree.

**Applies to.** [courier](../labels/courier.md), the bundled `parser.toml`.

## D-LABEL-26 — The receiver's area is read as printed

**Decision.** Each order records the receiver's province, city and district as its label prints them
(R-LABEL-17): as three columns of `orders.csv`. No spelling is put
right and no `Kab.` or `Kota` is taken off; a part the layout does not print is none. Only the spaces of a
province a line break cut are taken from the list of provinces. A batch saved before gets its area from its
stored inputs by the migration the owner starts (D-APP-33), straight into `orders.csv`.

**Why.** The owner's request: the area is on every label, and as printed it is the label's own truth;
normalising would be a guess the owner did not ask for.

**Applies to.** [reading](../labels/reading.md#the-receivers-area), [files](../data/files.md#orderscsv).

## D-LABEL-27 — The weight is read as printed, in grams

**Decision.** Each order records the parcel's weight its label prints (R-LABEL-19): as the `weight_grams` column of `orders.csv` (empty: not printed). The TikTok
labels print kilograms (`0.490 KG`), the Shopee-style one grams (`450 gr`); both are kept as whole grams. A
batch saved before gets its weight from its stored inputs by the migration the owner starts (D-APP-33), straight into `orders.csv`. No other field of the label is added: the
rest is unreliable or of no use to the owner.

**Why.** The owner's request: the weight is worth keeping; it is on every label but the instant ones.

**Applies to.** [reading](../labels/reading.md#the-weight), [files](../data/files.md#orderscsv), [Search](../ui/search.md).

## D-LABEL-28 — A packing list stops the batch

**Decision.** A page headed `Packing List` is a packing list the shop prints, not a shipping label. The reader stops the batch on it, wherever it falls and whatever order it names. The stop names the input and the page; the owner removes that page from the input and reads again. A packing list is never read as a slip: it prints no tracking ID and no courier, and after a label of its own order it would add its lines to that copy a second time.

**Why.** The owner does not feed a packing list into the program: it is not a shipping label, and read as a continuation it would add lines to a label's copy.

**Applies to.** [reading](../labels/reading.md#copies-and-multi-page-orders), [copies](../../crates/engine/src/copies/mod.rs).

## D-LABEL-30 — A name the instant label breaks is joined with nothing; filters ignore its spaces

**Decision.** The Shopee-style instant label breaks a product name or Seller SKU at any character, and a
break on a space prints the space on neither line. Its lines are joined with nothing between (`Sa` + `mping`
→ `Samping`), and the name is saved as read. Every filter (Search and the sort rules) compares the product
name and the Seller SKU with every space taken out of both sides, so `Standar |Penyangga` and
`Standar | Penyangga` match the same filter.

**Why.** The owner's answer to Q-LABEL-13: a word the line broke reads right, and a filter still finds a
name whose space the break dropped.

**Applies to.** [reading](../labels/reading.md), [conditions](../rules-file/conditions.md).
