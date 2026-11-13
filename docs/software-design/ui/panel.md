# Panel

The side panel lists the dates of the data folder, with the orders of every account together, and opens
another date ([D-UI-14](../decisions/ui.md)). It sits at the left of every screen; the owner of the app shows
or hides it.

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│ [▯] ◈ Packing v0.6                            Search  Accounts  Sort rules  Settings    │
├──────────────────────────┬─────────────────────────────────────────────────────────────┤
│ DATES                    │  Wed, 21 Oct 2026 (today) (2 batches) (729 orders)          │
│ ▾ October 2026           │                                         [ + New batch ]     │
│ ▌ Wed, 21 Oct · today 729│  ┌ Batch 2  saved 09:12 · 129 orders ── [Amend] ┐           │
│   Tue, 20 Oct        530 │  │  2-A  Sepatu         88 orders  [✓] [Open]   │           │
│   Mon, 19 Oct        412 │  │  2-Z  Lainnya        41 orders  [ ] [Open]   │           │
│ ▸ September 2026         │  └──────────────────────────────────────────────┘           │
│ ⚠ Sat, 3 Oct  damaged    │                                                             │
└──────────────────────────┴─────────────────────────────────────────────────────────────┘
```

## The panel

- **Dates** of the data folder, newest first (*Wed, 21 Oct*), each with its number of orders of every
  account, from the search index. Today is always listed, with *· today*, even before it has a folder. The
  open date is marked ▌ and shaded. One account's orders per date are on the [Accounts](accounts.md) screen.
- A date's row shows only the date and its orders: whether its group PDFs are printed is on the
  [Day](day.md) screen.
- **Months fold**: the month of the open date and the current month start open, the others closed.
- **Click a date** to open it ([01 Start and open](../workflow/01-start-and-open.md#2-open-a-date)). A draft
  being prepared asks first.
- **A damaged date** shows ⚠ and *damaged*; it opens for viewing only, with the message.
- **Row menu** (**⋯**, on hover): **Open folder**. A date is never renamed or deleted.
- **[▯]** in the title bar (or `Ctrl+B`) hides and shows the panel. Hidden, the screen takes the full width
  and the Day screen's title still names the date. It is shown at every start, and below the smallest window
  width it hides on its own.
- The list is read again when the window regains focus, so a folder added in Explorer shows up.

There is no day stepper in the title bar: the panel's *today* row is the way back.
