# Decisions: the summary PDF and the day summary

The LABEL scope's records about the printed summaries; the other LABEL records are in [label.md](label.md).
Where a record names a former question (`Q-…`), the owner accepted its default.

## D-LABEL-16 — The summary PDF reads facts, groups, picks

**Decision.** The summary PDF prints the facts pages first, then each group with its order contents (as the v0.1
packing list did), then the pick pages, each part from a new page (R-LABEL-2).

**Why.** The owner found the v0.1 order better: what the batch is, then what goes into each group PDF, then what
to take from stock.

**Applies to.** [summary PDF](../labels/summary-pdf.md).

## D-LABEL-19 — The summary PDF: one row everywhere, couriers by name

**Decision.** Every part of the summary PDF draws the same row: a bold key at the left, the text, the counts at
the right with their numbers under each other, a thin rule under it. The group pages number each order
content in its group, `A-1`, `A-2`, … (the group's code and the place). The facts pages list the orders per
courier by the courier's **name**, then the warnings, the inputs with the ledger, the sort rule and the
versions — not the group PDFs, which the group pages show, and not the instant orders' pickup codes, which
their labels carry. It replaces D-LABEL-18 (one layout in three count columns) and D-LABEL-8 (the pickup codes
on the facts page), and the rule of D-LABEL-15 that an order content has no number.

**Why.** The owner's words (2026-10-27): the group pages and the pick pages read far better than the first
page; the first page should take their spacing, name the couriers, and drop the groups and the instant
orders. The verifier reads each group's counts from its bar instead.

**Applies to.** [summary PDF](../labels/summary-pdf.md), [groups](../labels/groups.md#a-group-pdf),
[verification](../app/verification.md).

## D-LABEL-20 — The day summary: the batches as their Day cards

**Decision.** The day summary draws each live batch as its Day card shows it — a bar with the batch, its save
time and files, its orders and units; a row per group PDF with its orders and order contents (*3 contents*) —
then the orders per courier by name, and from a new page the day's pick pages. No Printed ticks, no reprints.
It replaces D-LABEL-17.

**Why.** The owner's words (2026-10-27): the Day screen already reads well, the PDF of the same facts read
worse; whether a group PDF is printed does not matter on this paper; *contents*, not *order contents*. The
reprints stay on the Day screen's *Reprints* card (Q-LABEL-12).

**Applies to.** [day summary](../labels/day-summary.md).

## D-LABEL-23 — The day summary: its inputs after the couriers, its order contents by most orders

**Decision.** After the couriers, the day summary lists each live batch's inputs and ledger as the summary
PDF's facts pages do, keyed by the batch's number (R-LABEL-11). After the pick pages it lists each order
content of the whole day once, over every batch and group, with its orders, most orders first; contents with
as many orders keep the one-line ones before the mixed ones (R-LABEL-10). The pick pages keep their order, by
display name. It replaces D-LABEL-22, under which the one-line contents came first, by item and quantity.

**Why.** The owner's words (2026-10-29): to know how many orders are one item, two items or mixed items, the
contents the most orders share read first, as in a group PDF; the day's paper should name its input PDFs as a
batch's summary does. The order contents list is long, so it is the last part.

**Applies to.** [day summary](../labels/day-summary.md).
