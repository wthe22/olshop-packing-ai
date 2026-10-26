# Search

Finds saved orders of the open namespace and prints their labels again. **Search** in the title bar opens it over
the current screen; **Close** returns to it. Searching changes nothing; **Reprint** writes one file
([reprint](../app/reprint.md)).

```
┌ Search ────────────────────────────────────────────────────────────────────── [Close] ┐
│  ┌ All of these ▾ ───────────────────────────────────────────────────────────────┐   │
│  │ [Tracking ID ▾] [starts with ▾] [JY00000012     ]  [ ] not  [×]              │   │
│  │ [+ condition] [+ group]                                                       │   │
│  └───────────────────────────────────────────────────────────────────────────────┘   │
│  Scope: (•) This day  ( ) All days                                                    │
│  3 orders in 2 batches                                       [ ] Select all results  │
│  [✓] 580000000000000042  JY0000001234  jnt  Sepatu ×1         7 Oct · 2-A  [Open] ▸   │
│  [ ] 580000000000000057  JY0000001235  jnt  Spion Beat ×2     7 Oct · 2-B  [Open] ▸   │
│      from 10-07_09-05-49_…_2.pdf p. 14 · courier jnt by logo 39b9… + form J[XY]\d{10} │
│  [✓] 580000000000000061  JY0000001240  jnt  Sepatu ×2         7 Oct · 1-A  [Open] ▸   │
│                                                            [ Reprint 2 orders ]      │
└───────────────────────────────────────────────────────────────────────────────────────┘
```

- **The filter** is made with the boxes of [Sort rules](sort-rules.md); the fields are those of the filter
  language ([fields](../rules-file/conditions.md#fields)). Matching ignores case; an item field matches when one
  line matches.
- **Scope**: *This day* (the open date, the default) or *All days* of the open namespace
  ([D-UI-3](../decisions/ui.md)). Results come from the search index ([reprint](../app/reprint.md#the-search-index));
  until it is built the screen says *Loading…*.
- **Results**: the number of orders and batches, then one row per order: Order ID · tracking ID · courier ·
  items (name × quantity) · date · group file. **Open** opens that group PDF. **▸** opens the row's trace
  line: its input and pages, the clue that named its courier.
- **Saved orders only**: an open draft's orders have no file yet and are not searched.
- **No match**: "No saved order matches."

## Reprint

- A tick on each row, and **Select all results** for every row the filter found.
- **Reprint `<n>` orders** writes one PDF of their label pages, unchanged, in their original printing order,
  into today's date: `reprint 1 x2.pdf` ([reprint](../app/reprint.md#reprint)). It is verified before it is
  kept; a finding stops it and nothing is kept.
- When it is written, a line says "Reprint 1 written: 2 orders, 2 pages." with **Open**, and the Day screen
  of today lists it under **Reprints**.
- A reprint changes nothing else: the orders stay saved in their batches and their printed ticks stay.
