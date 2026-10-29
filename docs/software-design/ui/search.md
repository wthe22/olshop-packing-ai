# Search

Finds saved orders of the open workspace and prints their labels again. **Search** in the title bar opens it over
the current screen; **Close** returns to it. Searching changes nothing; **Reprint** writes one file
([reprint](../app/reprint.md)).

```
┌ Search ──────────────────────────────────────────────────────────────────────────────────────── [×] ┐
│ ┌─────────────────────────────────────────────────────────────────────────────────────────────────┐ │
│ │ [All of these ▾]  [ ] not                                   │ Dates [Last 7 days            ▾] │ │
│ │ [Tracking ID ▾] [contains ▾] [00000012          ] [ ] not [×] │ [21/10/2026] to [27/10/2026]     │ │
│ │ [+ condition] [+ group]                                     │ [            Search            ] │ │
│ └─────────────────────────────────────────────────────────────────────────────────────────────────┘ │
│ 3 orders in 2 batches   [–] Select all                                                              │
│ ┌─────────────────────────────────────────────────────────────────────────────────────────────────┐ │
│ │     Order ID · Tracking ID  Items                            Group PDF · Courier                │ │
│ │ [✓] 580000000000000042      ×1  Sepatu Standar Samping Motor (2-A) Sepatu x88 page 14 [Open] ›  │ │
│ │     JY0000001234                — honda                      (J&T Express) Wed, 7 Oct Reprinted │ │
│ │ ─────────────────────────────────────────────────────────────────────────────────────────────── │ │
│ │ [ ] 580000000000000057      ×2  Spion Beat                   (2-B) Spion x35 page 3   [Open] ⌄  │ │
│ │     JY0000001235                                             (J&T Express) Wed, 7 Oct           │ │
│ │     from 10-07_09-05-49_…_2.pdf p. 3 · courier J&T Express by logo 39b9… + form J[XY]\d{10}     │ │
│ │ ─────────────────────────────────────────────────────────────────────────────────────────────── │ │
│ │ [✓] 580000000000000061      ×1  Kaca Spion                   (1-B) Spion x12 page 7   [Open] ›  │ │
│ │     no tracking ID          ×1  Spion Beat                   (Unknown) Tue, 6 Oct               │ │
│ └─────────────────────────────────────────────────────────────────────────────────────────────────┘ │
│ ─────────────────────────────────────────────────────────────────────────────────────────────────── │
│ 2 of 3 selected                                                      [Clear]  [ Reprint 2 orders ]  │
└─────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

On a narrower window the items take a line of their own under the IDs:

```
[✓] 580000000000000042                      (2-A) Sepatu x88 page 14  [Open] ›
    JY0000001234                            (J&T Express) Wed, 7 Oct Reprinted
    ×1  Sepatu Standar Samping Motor Pelindung Rantai Honda Beat — honda
```

- **The filter** is made with the boxes of [Sort rules](sort-rules.md), in one box with the dates: the
  boxes at the left, the dates and **Search** at the right; when Search is narrower than 60 rem, the dates
  and **Search** stand in one line under the boxes, so the box stays low. The fields are those of the
  filter language ([fields](../rules-file/conditions.md#fields)). The first box is *Tracking ID contains*,
  so part of an ID finds its order ([D-UI-12](../decisions/ui.md)). Matching ignores case; an item field
  matches when one line matches. The Courier row works as there: *equals* picks a courier from the list,
  *contains* and *starts with* take typed text matched against the couriers' names
  ([D-RULES-4](../decisions/rules.md)).
- **Dates**: *This day* (the open date, the default), *Yesterday*, *Last 7 days* and *Last 30 days* (today
  and the dates before it), *Custom range* or *All days* of the open workspace ([D-UI-3](../decisions/ui.md)).
  The range's first and last date stand beside the list, so each choice shows what it covers; picking a date
  there makes the range the owner's own (*Custom range*). *All days* hides them. Results come from the search
  index ([reprint](../app/reprint.md#the-search-index)); until it is built the screen says *Loading…*.
- **Enter** in a box searches, so a scanned tracking ID finds its order at once; the first box has the
  focus when Search opens.
- **Results**: the number of orders and batches and **Select all**, then one **row** per order under a
  head that names the columns ([D-UI-12](../decisions/ui.md)): its tick; the Order ID in bold over the
  tracking ID (*no tracking ID* when the label printed none); its items, one line each (quantity, then
  name); the group PDF that holds it, its code (`2-A`) as a chip, its name and the **page** it is on there
  (*page 14*, *pages 14–15* for a label with a second page), so the printed label is found in its stack,
  over the courier's name as a pill (*Unknown* when the label named none) and the date it was saved on;
  **Open** and the chevron. The ID and group PDF columns are as wide as their longest entry, so the items
  keep the rest of the row and a long product name wraps in one wide column. Results narrower than
  54 rem (a window much less than full width) put the items on a line of their own under the IDs; narrower
  than 34 rem, the group PDF line too, and the head is hidden. **Open** opens that group PDF, written again
  first when it is not there any more (as on the [Day](day.md) screen). The chevron opens the row's trace
  line under it: its input and pages there, the clue that named its courier, a logo's hash and the
  tracking-ID form in monospace.
- **Reprinted**: an order printed again before carries the mark *Reprinted* under **Open** (*Reprinted 2×* for more);
  its tooltip names each reprint and its date.
- **Saved orders only**: an open draft's orders have no file yet and are not searched.
- **No match**: "No saved order matches."

## Reprint

- A tick at the left of each row; a click on the row's text ticks it too, and a ticked row has a tinted
  ground and an accent bar at its left. A drag or a double click selects text instead (to copy an Order ID or
  a tracking ID) and leaves the tick as it was. **Select all** ticks every order the filter found or none;
  its tick shows a dash when only some are ticked. A search that finds one order ticks it.
- **The reprint bar** stays at the foot of the screen while the rows scroll: `<n> of <m> selected`,
  **Clear**, and **Reprint `<n>` orders**. When a ticked order was reprinted before, the bar says how many,
  and **Reprint** asks first: "Reprint 3 orders?" — "2 of them were reprinted before." [Reprint 3 orders]
  [Cancel].
- **Reprint `<n>` orders** writes one PDF of their label pages, unchanged, in their original printing order,
  into today's date: `reprint 1 x2.pdf` ([reprint](../app/reprint.md#reprint)). It is verified before it is
  kept; a finding stops it and nothing is kept.
- When it is written, a green note at the bar's left, in the bar's line while both fit, says "Reprint 1
  written: 2 orders, 2 pages." with **Open**; the rows are searched again, so the orders just printed
  carry *Reprinted*, and the Day screen of today lists it under **Reprints**.
- A reprint changes nothing else: the orders stay saved in their batches and their printed ticks stay.
