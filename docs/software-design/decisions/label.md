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

## D-LABEL-10 — The remembered images live in packing-workflows.toml

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
the copy right before it in the same input (Q-LABEL-9). A continuation page anywhere else stops the batch.

**Why.** A second copy right after the first is then still a repeat. Measured: every multi-page order of the
samples has its second page right after its first, and page ranges make every order traceable.

**Applies to.** [duplicates](../app/duplicates.md#1-copies), [reading](../labels/reading.md#copies-and-multi-page-orders).

## D-LABEL-13 — Two page identities, one for each job

**Decision.** The reader's **page fingerprint** (text, positions, header-image hashes) compares the copies of a
repeat (Q-LABEL-10). Whether an output page is its source page is the verifier's **page digest** (D-APP-12).

**Why.** The fingerprint is free while reading; the proof of an output must not come from the code that made it.

**Applies to.** [reading](../labels/reading.md#page-fingerprint), [verification](../app/verification.md#the-page-digest).

## D-LABEL-14 — Each order records the clue that named its courier

**Decision.** A saved order keeps `courier_clue`: the kind, the hash, text or tag, and the tracking-ID form.

**Why.** A wrong courier can then be traced to the clue that named it.

**Applies to.** [courier deduction](../labels/courier.md#the-clue-that-named-it), [records](../data/records.md).

## D-LABEL-15 — Orders with the same items are one order content

**Decision.** Inside a group PDF, the orders with the same **order content** — their item lines as (product
name, variation, quantity) — are printed next to each other, and the screens call such a block an *order
content* (Q-LABEL-11). The Seller SKU and the printed `Qty Total` are stored with the order, not with its
content, so one order content is exactly one block. The screens list a group's order contents in printing
order; the summary PDF numbers them ([D-LABEL-19](summaries.md#d-label-19--the-summary-pdf-one-row-everywhere-couriers-by-name)).

**Why.** It is the owner's own word and the data already names it (`order_contents`, `order-contents.toml`).

**Applies to.** [groups](../labels/groups.md#a-group-pdf), [03 Plan](../workflow/03-plan.md#10-order-contents-and-file-names), [records](../data/records.md), [views](../data/views.md), [Plan](../ui/plan.md).

## D-LABEL-21 — An instant label is named by its tag; its header images are never asked about

**Decision.** A courier's `instant_tags` name the courier of an instant-service label, which has no logo and no
tracking ID: `GKS` is Gojek, `Sameday` is Grab. Its header images are the buying channel's strips: none of them
is asked about in the logo prompt. A tag in no list is asked about once in the logo prompt. It replaces
D-LABEL-7.

**Why.** Nothing else on the page names the courier, and a new tag is a file edit, not a program change. The
prompt once showed an instant label's `TikTok Shop` strip, the owner answered *Grab*, and from then on no
TikTok instant label of Gojek or Grab could be named (a courier logo on an instant label names nothing).

**Applies to.** [courier deduction](../labels/courier.md), item 7; [rules file](../rules-file/README.md#couriers).
