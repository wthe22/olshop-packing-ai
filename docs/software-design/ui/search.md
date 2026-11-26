# Search

Finds saved orders of every account and prints their labels again. **Search** in the title bar opens it over
the current screen; **Close** returns to it. Searching changes nothing; **Reprint** writes one file
([reprint](../app/reprint.md)).

```
┌ Search ───────────────────────────────────────────────────────────────────────────────────────────────────────────── [×] ┐
│ ┌──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐ │
│ │ [All of these ▾]  [ ] not                                                         │ Dates [Last 7 days          ▾]   │ │
│ │ [Tracking ID ▾] [contains ▾] [00000012                    ] [ ] not [×]           │ [21/10/2026] to [27/10/2026]     │ │
│ │ [+ condition] [+ group]                                                           │ [Reset] [       Search       ]   │ │
│ └──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘ │
│ 3 orders in 2 batches                                                                                                    │
│   [–] Order                         Tracking                        Saved              Label                             │
│ ┌──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐ │
│ │ [✓] Toko Alfa 580000000000000042  (J&T Express) JY0000001234      Wed, 7 Oct 09:12   Batch 2 (2-B)  Reprinted [Open] │ │
│ │     ──────────────────────────────────────────────────────────────────────────────────────────────────────────────── │ │
│ │     ×2  Sepatu Standar Samping Motor | Pelindung Rantai     Hitam, Beat Fi     SST-BEAT-01                           │ │
│ │         Honda Beat Scoopy Vario                                                                                      │ │
│ │     ×1  Spion Beat | Kaca Spion Motor Honda Beat            honda              SPI-01                                │ │
│ │     ──                                                                                                               │ │
│ │     3   TOTAL                                                                                                        │ │
│ │     ────────────────────────────────────────────────────────────────────────────────────────────────────────────── ⌄ │ │
│ └──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘ │
│ ┌──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐ │
│ │ [ ] Toko Beta 580000000000000057  (SPX Express) SPXID000000000057 Wed, 7 Oct 09:12   Batch 2 (2-C)            [Open] │ │
│ │     ──────────────────────────────────────────────────────────────────────────────────────────────────────────────── │ │
│ │     ×1  Jok Kulit Vario | Custom                            Merah              JOK-01                                │ │
│ │     ────────────────────────────────────────────────────────────────────────────────────────────────────────────── ⌃ │ │
│ │     ADDRESS     JAWA BARAT, KOTA CONTOH, KECAMATAN CONTOH                                                            │ │
│ │     SHIP BY     Thu, 8 Oct 2026                                                                                      │ │
│ │     WEIGHT      490 g                                                                                                │ │
│ │     INPUT FILE  [▯ labels pagi.pdf] page 3  SHA256: e731bbe96035                                                     │ │
│ │     GROUP       (2-C) Jok dan Cover Motor  pages 102–103                                                             │ │
│ └──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘ │
│ ┌──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐ │
│ │ [✓] Toko Alfa 580000000000000061  (Unknown) no tracking ID        Tue, 6 Oct 16:40   Batch 1 (1-A)            [Open] │ │
│ │     ──────────────────────────────────────────────────────────────────────────────────────────────────────────────── │ │
│ │     ×1  Kaca Spion                                                             SPI-RTR                               │ │
│ │     ────────────────────────────────────────────────────────────────────────────────────────────────────────────── ⌄ │ │
│ └──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘ │
│ ──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────── │
│ 2 of 3 selected                                                                                   [Clear]  [ Reprint ]   │
└──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

Where the results are 72 rem wide or more, the label column also holds the group's name and the page,
`Batch 2 (2-B) Sepatu page 14`, and the details have no *GROUP* line.

- **The filter** is made with the boxes of [Sort rules](sort-rules.md), in one box with the dates: the
  boxes at the left, the dates and **Search** at the right; when Search is narrower than 60 rem, the dates
  and **Search** stand in one line under the boxes, so the box stays low. The fields are those of the
  filter language ([fields](../rules-file/conditions.md#fields)). The first box is *Order ID contains*,
  so part of an ID finds its order ([D-UI-16](../decisions/ui.md)). *Account* and *Shop* narrow it to one
  account or one shop: *Account equals* picks an account from the list ([D-RULES-6](../decisions/rules.md)). Matching ignores case; an item field
  matches when one line matches. The Courier row works as there: *equals* picks a courier from the list,
  *contains* and *starts with* take typed text matched against the couriers' names
  ([D-RULES-4](../decisions/rules.md)). Search opens again with the boxes as the owner left them, for as
  long as the program runs; the dates start from *Viewed day* each time (R-UI-56). **Reset**, left of
  **Search**, empties the boxes to the first one and sets the dates back.
- **Dates**: *Viewed day* (the date the Day screen shows, the default; left out while that date is today),
  *Today*, *Yesterday*, *Last 7 days* and *Last 30 days* (today and the dates before it), *Custom range* or
  *All days* ([D-UI-3](../decisions/ui.md)). Every account is searched unless the filter names one.
  The range's first and last date stand beside the list, so each choice shows what it covers; picking a date
  there makes the range the owner's own (*Custom range*). *All days* hides them. Results come from the search
  index ([reprint](../app/reprint.md#the-search-index)); until it is built the screen says *Loading*.
- **Enter** in a box searches, so a scanned Order ID finds its order at once; the first box has the
  focus when Search opens.
- **Results**: the number of orders and batches, then the **heads** (*Order*, *Tracking*, *Saved*, *Label*,
  with **Select all** as the tick at their left) and one **card** per order under them
  ([D-UI-16](../decisions/ui.md), R-UI-58). The card's **head** is one row on the heads' columns, each as
  wide as its longest entry: its tick; its **shop** in grey (the account's name in grey italics when the
  label printed no sender; a long name ends with dots, whole in its tooltip) and its **Order ID** in bold;
  its courier's name as a pill (*Unknown* when the label named none) and the **tracking ID** (*no tracking
  ID* when the label printed none); the date and time its batch was saved (*Wed, 7 Oct 09:12*); the batch
  and the group's code as a chip (*Batch 2* `2-B`), then, where the results are 72 rem wide or more, the
  group's name (from the batch's sort rule, without the count its PDF's name carries) and the **page** the
  order is on in the group PDF (*page 14*, *pages 14–15* for a label with a second page), so the printed
  label is found in its stack; a long group name ends with dots before the page does. At the right the
  *Reprinted* mark and **Open**. Under a line, the **items** as a table without heads, one line each: the
  quantity, the name (it wraps and takes the room left), the variation and the **Seller SKU** in monospace,
  these two at fixed widths so they line up from card to card. When the order has several lines, or a Qty
  Total its one line does not add up to, the slip's **Qty Total** stands at their foot under the
  quantities, after a short stroke. **Open** opens the order's group PDF, written again first when it is not
  there any more (as on the [Day](day.md) screen).
- **Details**: a click on the card (not on its tick or **Open**), or on the bar under its items (a line with
  a chevron at its end), opens the card's details under the bar, one line per head; a click there again
  closes them, and a click inside the open details changes nothing, so their text can be selected. The
  lines: *ADDRESS* (the receiver's province, city and district as the label prints them); *SHIP BY*;
  *WEIGHT* (`1,250 g`); *PICKUP CODE* when the tracking ID is another; *ACCOUNT* when the shop has another
  name; *INPUT FILE*, the input PDF it was read from (its file name in a box with a file icon, in
  monospace), its pages there and its short hash named by its algorithm (*SHA256: e731bbe96035*, the one its
  file name carries); and, where the card's head has no room for them, *GROUP*: the group's code, its name
  and the page. A line the label does not print is left out. For an order saved before the address or the
  weight was read, in a data folder not updated since, *not read yet* stands in its place, grey, and its
  tooltip says to start the program again and choose **Update**. How the courier was named is not shown
  (the journal keeps it).
- **Reprinted**: an order printed again before carries the mark *Reprinted* left of **Open** (*Reprinted 2×* for more);
  its tooltip names each reprint and its date.
- **Saved orders only**: an open draft's orders have no file yet and are not searched.
- **No match**: "No saved order matches."

## Reprint

- A tick at the left of each card; only the tick ticks it (a click on the card opens its details), and a
  ticked card has a tinted ground, an accent border and an accent bar at its left. A drag or a double click
  on the card selects text instead (to copy an Order ID or a tracking ID) and leaves it as it was.
  **Select all**, the tick in the heads' row, ticks every order the filter found or none; it shows a dash
  when only some are ticked. A search that finds one order ticks it.
- **The reprint bar** stays at the foot of the screen while the cards scroll: `<n> of <m> selected`,
  **Clear**, and **Reprint `<n>` orders**. When a ticked order was reprinted before, the bar says how many,
  and **Reprint** asks first: "Reprint 3 orders?" — "2 of them were reprinted before." [Reprint]
  [Cancel].
- **Reprint `<n>` orders** writes one PDF of their label pages, unchanged, in their original printing order,
  into today's date: `Reprint 1 x2.pdf` ([reprint](../app/reprint.md#reprint)). It is verified before it is
  kept; a finding stops it and nothing is kept.
- When it is written, a green note at the bar's left, in the bar's line while both fit, says "Reprint 1
  written: 2 orders, 2 pages." with **Open**; the orders are searched again, so the orders just printed
  carry *Reprinted*, and the Day screen of today lists it under **Reprints**.
- A reprint changes nothing else: the orders stay saved in their batches and their printed ticks stay.
