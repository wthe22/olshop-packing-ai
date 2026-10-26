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

## D-LABEL-7 — An instant-service label is named by its service tag

**Decision.** A courier's `instant_tags` name the courier of an instant-service label, which has no logo and no
tracking ID: `GKS` is Gojek. A tag in no list is asked about once in the logo prompt, like an unseen logo
(Q-LABEL-2).

**Why.** Nothing else on the page names the courier, and a new tag is a file edit, not a program change.

**Applies to.** [courier deduction](../labels/courier.md), item 7; [rules file](../rules-file/README.md#couriers).

## D-LABEL-8 — The pickup code is printed in the summary PDF

**Decision.** An instant order's pickup code is read and kept with the order, and the summary PDF's facts page
lists it beside the order (Q-LABEL-3).

**Why.** The driver asks for the code; it belongs on the paper the owner carries.

**Applies to.** [summary PDF](../labels/summary-pdf.md#the-facts-pages), [records](../data/records.md).

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
order; the summary PDF numbers them ([D-LABEL-18](#d-label-18--the-summary-pdf-numbers-order-contents-and-counts-couriers)).

**Why.** It is the owner's own word and the data already names it (`order_contents`, `order-contents.toml`).

**Applies to.** [groups](../labels/groups.md#a-group-pdf), [03 Plan](../workflow/03-plan.md#10-order-contents-and-file-names), [records](../data/records.md), [views](../data/views.md), [Plan](../ui/plan.md).

## D-LABEL-16 — The summary PDF reads facts, groups, picks

**Decision.** The summary PDF prints the facts pages first, then each group with its order contents (as the v0.1
packing list did), then the pick pages, each part from a new page (R-LABEL-2).

**Why.** The owner found the v0.1 order better: what the batch is, then what goes into each group PDF, then what
to take from stock.

**Applies to.** [summary PDF](../labels/summary-pdf.md).

## D-LABEL-17 — The day summary: batches, couriers, picks

**Decision.** The day summary holds each live batch with its group PDFs and their Printed ticks, the orders per
courier, and from a new page the day's pick pages. It does not list the reprints (Q-LABEL-12, option 1
without the reprints).

**Why.** The owner's answer: the reprints are on the Day screen's *Reprints* card; the paper is for the end of
the day — what is not printed yet, what each courier takes, what was taken from stock.

**Applies to.** [day summary](../labels/day-summary.md).

## D-LABEL-18 — The summary PDF numbers order contents and counts couriers

**Decision.** On the summary PDF's group pages each order content carries a number in its group, `A-1`, `A-2`,
… (the group's code and the place). The facts pages list the batch's orders per courier after the groups. The
summary PDF and the day summary share one layout: a bar per section, rows whose text starts in line with the
bar's heading, the counts in three columns, the same gaps everywhere. It replaces the rule of D-LABEL-15 that
an order content has no number.

**Why.** The owner asked for numbers to refer to an order content easily, for the couriers' counts on the
batch's paper, and for a first page that reads cleanly: the old facts page had text out of line with the
bars, uneven gaps and the counts inside the text; the day summary's columns read better.

**Applies to.** [summary PDF](../labels/summary-pdf.md), [day summary](../labels/day-summary.md),
[groups](../labels/groups.md#a-group-pdf).
