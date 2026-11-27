# Archive

One page for keeping old dates small ([archive](../app/archive.md), [D-APP-43](../decisions/date-files.md)).
It is opened from Settings › Data › **Archive**; nothing about storage shows on the daily screens.

```
┌ Archive ──────────────────────────────────────────────────────────────────────── [×] ┐
│ AUTOMATIC                                                                            │
│ ┌──────────────────────────────────────────────────────────────────────────────────┐ │
│ │ Archive dates older than [ 60 ] days                              [Off] [On]     │ │
│ └──────────────────────────────────────────────────────────────────────────────────┘ │
│ DATES                                                                 3.2 GB in all  │
│ ┌──────────────────────────────────────────────────────────────────────────────────┐ │
│ │      DATE                 ORDERS        SIZE                                     │ │
│ │ ▾ November 2026                                                                  │ │
│ │      Fri, 27 Nov · today     338     61.0 MB                                     │ │
│ │ [ ]  Thu, 26 Nov             358     58.2 MB                                     │ │
│ │ ▾ October 2026  [✓]                                                              │ │
│ │ [✓]  Tue, 27 Oct             941     72.4 MB                                     │ │
│ │      Mon, 26 Oct           1,612      0.2 MB    Archived                         │ │
│ ├──────────────────────────────────────────────────────────────────────────────────┤ │
│ │ 1 date · 72.3 MB of PDFs                                             [Archive]   │ │
│ └──────────────────────────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────────────────────┘
```

- **Automatic**: *Off* to begin with. **On** keeps the number (at least 1, 60 to begin with), asks first when
  it would archive any date now, and writes `archive_after_days` into `packing-app.toml`; **Off** removes it.
  A changed number is written when the box loses focus, asking first the same way.
- **Dates**: every date of the data folder, newest first, under month heads that fold as on the panel; each
  with its orders and its size (every file of its folder; `KB`, `MB` or `GB`, one decimal under 10). The total
  stands at the right of the group's head. An archived date says *Archived* in grey.
- **The tick** of a date that can be archived (it has a saved batch, is not archived, not today, not damaged).
  **Shift**+click ticks every date between the last tick and this one. A month's tick ticks or clears its dates
  that can be archived, and shows a dash while only some are ticked. The other dates have no tick.
- **The foot** stays in view: the dates ticked and the size of their PDFs, and **Archive**, off while nothing
  is ticked. **Archive** asks: *Archive 3 dates? Their PDFs (210 MB) are deleted; their orders and summaries
  stay.* [Archive] [Cancel]. While it runs, the dialog *Archiving: 2026-10-07, 2 of 3*; a file that could not
  be deleted is named after it.
- In read-only mode the page shows the dates and sizes only: no ticks, no foot, *Automatic* not changeable.
