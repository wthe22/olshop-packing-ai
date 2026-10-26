# Day summary (A4)

One file for the whole open date of a workspace, made on request: **Day summary** on the
[Day screen](../ui/day.md) writes `{workspace}/{date}/day summary.pdf` from the date's journal and opens
it in the PC's default PDF viewer. It is a view of the journal ([views](../data/views.md)): written again
on every request, never read for a decision, not recorded and not checked by the verifier. It holds the
batches, the couriers and the day's picks, not the reprints ([D-LABEL-17](../decisions/label.md)), and is
drawn in the [summary PDF](summary-pdf.md#the-layout)'s layout ([D-LABEL-18](../decisions/label.md)).

```
Day summary · Toko Contoh · 2026-10-07                                                 page 1/2
729 orders · 801 units · 2 batches · 5 groups · printed 17:05
▓ Batches                                                                                     ▓
  Batch 2 · saved 09:12 · sort rule default                 129 orders     129 pages
       2-A Sepatu x88.pdf                                    88 orders      88 pages       printed
       2-Z Lainnya x41.pdf                                   41 orders      41 pages   not printed
  Batch 1 · saved 07:40 · sort rule default · amended       600 orders     602 pages
       …
▓ Couriers                                                                                    ▓
  jnt                                                       412 orders
  spx                                                       314 orders
  unknown                                                     3 orders
```

- Header `Day summary · <workspace> · <date>`, then `<orders> orders · <units> units · <batches> batches ·
  <groups> groups · printed hh:mm`, and `page x/y`. Only live batches count. A count of one reads singular
  (*1 order*): nothing reads this file back.
- **Batches**: newest first, as on the Day screen. Each batch is a head row — its number, the time of its
  newest save, its sort rule, *amended* when it was — with its orders and pages; then each group PDF as a
  part: its file name, its orders and pages, and *printed* or *not printed* by its **Printed** tick, so a
  group PDF not yet printed shows at the end of the day.
- **Couriers**: the day's orders per courier code, most orders first; the orders no clue named are
  *unknown*, last. The count the owner hands over to each courier's pickup.
- Then, from a new page, the **pick pages** of the whole day: the units per item over every live batch, as
  the summary PDF's pick pages, under the bar *Pick summary (units to take from stock)*.
- A date with no live batch has no day summary: the button is off.
- The file is written to `day summary.pdf.partial` and renamed over the old one. A day summary open in a
  PDF viewer that locks it stops the write with the file named; the old file stays.
