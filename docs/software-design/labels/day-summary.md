# Day summary (A4)

One file for the whole open date of a workspace, made on request: **Day summary** on the
[Day screen](../ui/day.md) writes `{workspace}/{date}/day summary.pdf` from the date's journal and opens
it in the PC's default PDF viewer. It is a view of the journal ([views](../data/views.md)): written again
on every request, never read for a decision, not recorded and not checked by the verifier. It holds the
batches as their Day cards show them, the couriers, the inputs, the day's picks and the day's order
contents, not the reprints, and is drawn in the [summary PDF](summary-pdf.md#the-layout)'s layout
([D-LABEL-20](../decisions/summaries.md), [D-LABEL-23](../decisions/summaries.md)).

```
Day summary · Toko Contoh · 2026-10-07                                                 page 1/2
729 orders · 801 units · 2 batches · 5 groups · printed 17:05
▓ Batch 2 · saved 09:12 · 2 files                                       129 orders · 141 units ▓
  2-A     Sepatu                                               88 orders     3 contents
  ─────────────────────────────────────────────────────────────────────────────────────────────
  2-Z     Lainnya                                              41 orders    12 contents
  ─────────────────────────────────────────────────────────────────────────────────────────────
▓ Batch 1 · saved 07:40 · 3 files · amended                             600 orders · 660 units ▓
  …
▓ Couriers                                                                                    ▓
  J&T Express                                                 412 orders
  ─────────────────────────────────────────────────────────────────────────────────────────────
  Unknown                                                       3 orders
  ─────────────────────────────────────────────────────────────────────────────────────────────
▓ Inputs                                                                                      ▓
  2       labels_3.pdf                                       130 pages            3f1c0a9be2d4
  ─────────────────────────────────────────────────────────────────────────────────────────────
  2       130 pages read = 129 written + 1 left out (1 repeats, 0 saved earlier)
  ─────────────────────────────────────────────────────────────────────────────────────────────
  1       …

  (from a new page: the pick pages; then from another new page:)
▓ Order contents (orders per content)                               212 contents · 729 orders ▓
  1       Sepatu Standar Samping Motor — honda ×1                                        301
  ─────────────────────────────────────────────────────────────────────────────────────────────
  2       Sepatu Standar Samping Motor — honda ×2                                         42
  ─────────────────────────────────────────────────────────────────────────────────────────────
  …
  212     Kaca Spion ×1                                                                    1
          Spion Beat ×1
```

- Header `Day summary · <workspace> · <date>`, then `<orders> orders · <units> units · <batches> batches ·
  <groups> groups · printed hh:mm`, and `page x/y`. Only live batches count. A count of one reads singular
  (*1 order*): nothing reads this file back.
- **Each batch** is a bar, newest first, as on the Day screen: its number, the time of its newest save,
  its files, *amended* when it was; at the bar's right its orders and units. Under it each group PDF as a
  row: `<batch>-<code>` as the key, the group's name, its orders and its order contents (*3 contents*).
- **Couriers**: the day's orders per courier, by name, most orders first; the orders no clue named are
  *Unknown*, last. The count the owner hands over to each courier's pickup.
- **Inputs**: each live batch's inputs, newest batch first, the batch's number as the key: the name the
  file had, its pages and the first 12 characters of its SHA-256, then the batch's ledger in bold, as the
  summary PDF's facts pages show them. The paper alone names the day's files.
- Then, from a new page, the **pick pages** of the whole day: the units per item over every live batch, as
  the summary PDF's pick pages, under the bar *Pick summary (units to take from stock)*.
- Last, from a new page, the day's **order contents** under the bar *Order contents (orders per content)*,
  with `<n> contents · <orders> orders` at its right: each order content of every live batch and group
  once, numbered from 1 as its key, its items as `display name ×quantity`, one per line, and its orders at
  the right in bold, as the summary PDF's group pages draw a content. Most orders first, as a group PDF
  orders its contents; among contents with as many orders, the ones of one line come first, by display
  name and then quantity, then the mixed ones (two lines or more), by their lines. It shows how many
  orders are one of an item, two of it, or a mix; it is the longest part, so it comes last.
- A date with no live batch has no day summary: the button is off.
- The file is written to `day summary.pdf.partial` and renamed over the old one. A day summary open in a
  PDF viewer that locks it stops the write with the file named; the old file stays.
