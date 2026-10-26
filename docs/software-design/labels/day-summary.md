# Day summary (A4)

One file for the whole open date of a workspace, made on request: **Day summary** on the
[Day screen](../ui/day.md) writes `{workspace}/{date}/day summary.pdf` from the date's journal and opens
it in the PC's default PDF viewer. It is a view of the journal ([views](../data/views.md)): written again
on every request, never read for a decision, not recorded and not checked by the verifier. What it holds
is open: [Q-LABEL-12](../questions/label.md); the default is below.

```
Day summary · Toko Contoh · 2026-10-07                                                 page 1/2
729 orders · 801 units · 2 batches · 5 group PDFs · 1 reprint · printed 17:05
▓ Couriers                                                                                    ▓
  jnt · 412 orders
  spx · 314 orders
  unknown · 3 orders
▓ Batches                                                                                     ▓
  Batch 2 · saved 09:12 · 129 orders · 141 units · sort rule default
    2-A Sepatu · 88 orders | 3 contents · printed
    2-Z Lainnya · 41 orders | 12 contents · not printed
  Batch 1 · saved 07:40 · 600 orders · 660 units · sort rule default · amended
    …
▓ Reprints                                                                                    ▓
  reprint 1 x3.pdf · 09:40 · 3 orders · 3 pages
    580000000000000042 · saved 2026-10-07 in batch 2
    …
```

- Header `Day summary · <workspace> · <date>`, then `<orders> orders · <units> units · <batches> batches ·
  <group PDFs> group PDFs · <reprints> reprints · printed hh:mm`, and `page x/y`; the same header layout as
  the [summary PDF](summary-pdf.md). Only live batches count; a reprint's orders are not counted again.
- **Couriers**: the day's orders per courier code, most orders first; the orders no clue named are
  *unknown*. The count the owner hands over to each courier's pickup.
- **Batches**: newest first, as on the Day screen; each group PDF with its counts and whether its
  **Printed** tick is set, so a group PDF not yet printed shows at the end of the day.
- **Reprints**: each reprint of the date with its orders, the date and batch each was saved in; *none*
  when there is none.
- Then, from a new page, the **pick pages** of the whole day: the units per item over every live batch, as
  the summary PDF's pick pages, under the bar *Pick summary (units to take from stock)*.
- A date with no live batch has no day summary: the button is off.
- The file is written to `day summary.pdf.partial` and renamed over the old one. A day summary open in a
  PDF viewer that locks it stops the write with the file named; the old file stays.
