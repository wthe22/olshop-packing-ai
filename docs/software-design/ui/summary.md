# Summary

The summary of a batch or of the open date, shown in the app from the date's files, so it can be read
without writing or opening a PDF ([D-UI-19](../decisions/ui-screens.md)). It shows what the PDF shows, in the
same order and the same rows: the [summary PDF](../labels/summary-pdf.md) of a batch, the
[day summary](../labels/day-summary.md) of a date.

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│  Wed, 7 Oct 2026 (today)                                                               │
│  Summary · Batch 2     [All accounts ▾]                             [PDF]   [Back]     │
│  129 orders · 141 units · 2 groups · saved 09:12                                        │
├────────────────────────────────────────────────────────────────────────────────────────┤
│  ▓ Facts                                                                             ▓ │
│    2    labels_3.pdf                                130 pages              3f1c0a9be2d4 │
│    130 pages read = 129 written + 1 left out (1 repeats, 0 saved earlier)              │
│  ▓ 2-A  Sepatu                                                 88 orders · 96 units  ▓ │
│    Sepatu Standar Samping Motor ×1                               80 orders             │
│    …                                                                                   │
│  ▓ Picks                                                                             ▓ │
│    Sepatu Standar Samping Motor                                  96                    │
│    …                                                                                   │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

- **Opened** by **Summary** on a batch card (that batch) and **Day summary** on the
  [Day screen](day.md) (the open date). It reads `day.toml`, `orders.csv` and `order-contents.toml` and
  writes nothing, so it also works in read-only mode and on a date whose PDFs are gone.
- **The account list** shows *All accounts*, then each account of the batch or date; choosing one shows that
  account's orders only, as its summary PDF does ([summary PDF › One account](../labels/summary-pdf.md#one-account)).
  It is not shown when there is one account.
- **The parts** are those of the PDF, one under the other, each with its shaded head: the facts, each group
  with its order contents in printing order, the picks; for a date, the batches as their Day cards show them,
  the couriers, the inputs, the day's picks and order contents. The page scrolls; nothing is cut into pages.
- **PDF** writes that summary as a PDF and opens it in the PC's default PDF viewer, as the Day screen's
  buttons did: a batch's summary of all accounts is its `{batch} Summary.pdf` (written again when missing),
  an account's or a date's is written on this press ([day summary](../labels/day-summary.md),
  [summary PDF](../labels/summary-pdf.md#one-account)). It is off in read-only mode.
- **Back** returns to the Day screen as it was.
