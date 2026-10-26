# Search

Finds saved orders of the open workspace and prints their labels again. **Search** in the title bar opens it over
the current screen; **Close** returns to it. Searching changes nothing; **Reprint** writes one file
([reprint](../app/reprint.md)).

```
┌ Search ────────────────────────────────────────────────────────────────────── [Close] ┐
│  ┌ All of these ▾ ───────────────────────────────────────────────────────────────┐   │
│  │ [Tracking ID ▾] [starts with ▾] [JY00000012     ]  [ ] not  [×]              │   │
│  │ [+ condition] [+ group]                                                       │   │
│  └───────────────────────────────────────────────────────────────────────────────┘   │
│  Scope: (•) This day  ( ) All days                                         [Search]   │
│  3 orders in 2 batches                                                                │
│  [–] Order ID            Tracking ID   Courier  Items          Saved in               │
│  [✓] 580000000000000042  JY0000001234  jnt      Sepatu ×1      Wed 7 Oct · 2-A [Open] ▸│
│  [ ] 580000000000000057  JY0000001235  jnt      Spion Beat ×2  Wed 7 Oct · 2-B [Open] ▸│
│      Reprinted                                                                        │
│      from 10-07_09-05-49_…_2.pdf p. 14 · courier jnt by logo 39b9… + form J[XY]\d{10} │
│  [✓] 580000000000000061  JY0000001240  jnt      Sepatu ×2      Tue 6 Oct · 1-A [Open] ▸│
│ ───────────────────────────────────────────────────────────────────────────────────── │
│  2 of 3 selected                                          [Clear]  [ Reprint 2 orders ] │
└───────────────────────────────────────────────────────────────────────────────────────┘
```

- **The filter** is made with the boxes of [Sort rules](sort-rules.md); the fields are those of the filter
  language ([fields](../rules-file/conditions.md#fields)). Matching ignores case; an item field matches when one
  line matches.
- **Scope**: *This day* (the open date, the default) or *All days* of the open workspace
  ([D-UI-3](../decisions/ui.md)). Results come from the search index ([reprint](../app/reprint.md#the-search-index));
  until it is built the screen says *Loading…*.
- **Enter** in a box searches, so a scanned tracking ID finds its order at once; the first box has the
  focus when Search opens.
- **Results**: the number of orders and batches, a head row (*Order ID · Tracking ID · Courier · Items ·
  Saved in*), then one row per order: Order ID · tracking ID · courier · items (name × quantity) · date ·
  group file. **Open** opens that group PDF. **▸** opens the row's trace line: its input and pages, the clue
  that named its courier.
- **Reprinted**: an order printed again before carries the mark *Reprinted* (*Reprinted 2×* for more);
  its tooltip names each reprint and its date.
- **Saved orders only**: an open draft's orders have no file yet and are not searched.
- **No match**: "No saved order matches."

## Reprint

- A tick at the left of each row; a click anywhere on the row's text ticks it too, and a ticked row is
  shaded. The head row's tick, in the same column, ticks every row the filter found or none; it shows a
  dash when only some are ticked. A search that finds one order ticks it.
- **The reprint bar** stays at the foot of the screen while the rows scroll: `<n> of <m> selected`,
  **Clear**, and **Reprint `<n>` orders**. When a ticked order was reprinted before, the bar says how many,
  and **Reprint** asks first: "2 of these orders were reprinted before. Reprint 3 orders anyway?"
  [Cancel] [Reprint 3 orders].
- **Reprint `<n>` orders** writes one PDF of their label pages, unchanged, in their original printing order,
  into today's date: `reprint 1 x2.pdf` ([reprint](../app/reprint.md#reprint)). It is verified before it is
  kept; a finding stops it and nothing is kept.
- When it is written, a green line over the bar says "Reprint 1 written: 2 orders, 2 pages." with **Open**;
  the rows are searched again, so the orders just printed carry *Reprinted*, and the Day screen of today
  lists it under **Reprints**.
- A reprint changes nothing else: the orders stay saved in their batches and their printed ticks stay.
