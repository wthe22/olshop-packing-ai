# Day summary (A4)

One file for the whole open date of a workspace, made on request: **Day summary** on the
[Day screen](../ui/day.md) writes `{workspace}/{date}/day summary.pdf` from the date's journal and opens
it in the PC's default PDF viewer. It is a view of the journal ([views](../data/views.md)): written again
on every request, never read for a decision, not recorded and not checked by the verifier. It holds the
batches as their Day cards show them, the couriers and the day's picks, not the reprints, and is drawn in
the [summary PDF](summary-pdf.md#the-layout)'s layout ([D-LABEL-20](../decisions/label.md)).

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
```

- Header `Day summary · <workspace> · <date>`, then `<orders> orders · <units> units · <batches> batches ·
  <groups> groups · printed hh:mm`, and `page x/y`. Only live batches count. A count of one reads singular
  (*1 order*): nothing reads this file back.
- **Each batch** is a bar, newest first, as on the Day screen: its number, the time of its newest save,
  its files, *amended* when it was; at the bar's right its orders and units. Under it each group PDF as a
  row: `<batch>-<code>` as the key, the group's name, its orders and its order contents (*3 contents*).
- **Couriers**: the day's orders per courier, by name, most orders first; the orders no clue named are
  *Unknown*, last. The count the owner hands over to each courier's pickup.
- Then, from a new page, the **pick pages** of the whole day: the units per item over every live batch, as
  the summary PDF's pick pages, under the bar *Pick summary (units to take from stock)*.
- A date with no live batch has no day summary: the button is off.
- The file is written to `day summary.pdf.partial` and renamed over the old one. A day summary open in a
  PDF viewer that locks it stops the write with the file named; the old file stays.
