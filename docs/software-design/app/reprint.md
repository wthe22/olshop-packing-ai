# Search, reprint and restore

A label is sometimes lost or damaged after printing, and the shop must print it again — never twice by
mistake. The app finds saved orders, prints their labels again from the stored inputs, and writes a lost
output again ([D-APP-14](../decisions/app-features.md), [D-APP-15](../decisions/app-features.md)).

## The search index

- Built in memory at start, on its own thread, by reading the `orders.csv` and `order-contents.toml` of every date of the data folder
  ([files](../data/files.md)); after each commit its date is read again. A year of dates
  reads in about a second; Search shows *Loading* until it is ready.
- One entry per order of a live batch: the fields a filter can use, its account and shop among them
  ([conditions](../rules-file/conditions.md#fields)), its date, batch and label group, the output file and
  the pages it fills there (the pages of the orders of its group before it, in printing order, plus one),
  and its **trace**: the input's file name and pages. The courier's clue is not stored: the page is read again
  to trace it ([Q-APP-26](../questions/app.md)).
- Each date's reprints: which orders (by their Order ID) each reprint
  printed. Search marks those orders *Reprinted*, and the Day screen lists a reprint's orders from the
  index. A reprint's commit refreshes its date in the index like any other commit.
- The index is only read. A date whose files are damaged is left out and named in the panel.

## Search

The [Search](../ui/search.md) screen takes a filter made with the boxes of the Sort rules screen and the
dates to search: a range (the viewed date by default) or all dates, of every account
([D-UI-3](../decisions/ui.md)); only the dates of the range are looked at. Each result row shows its shop and the order, its items, its date, batch and group
file and its page there, a tick, and when opened its details: the input PDF and pages it was read from,
the batch's save time and the group's code:

```
From [10-07_07-00-00_Shipping label+Packing slip_1.pdf] p. 12–13, saved at 09:12 into group 2-A
```

Orders of an open draft are not searched: they are not saved.

## Reprint

**Reprint** writes one PDF of the ticked orders' label pages.

1. **Sources.** For each chosen order, read its date's files (not the index) and take the copy its live
   batch wrote: the input and the page range. An input that is missing or changed stops the
   reprint with its name.
2. **Order.** The orders in their original printing order: by date, batch, then their place in
   `orders.csv`.
3. **Write** the pages, unchanged, into `.partial/` of **today's** date, named
   `Reprint {n} x{count}.pdf`, `n` counting the reprints of today.
4. **Verify**: the file's page *k* has the digest of the source page at *k*, and the page count adds up
   ([verification](verification.md)). A finding stops the reprint; nothing is kept.
5. **Commit** ([save](../data/save.md)): the reprint is added to today's `day.toml` with its order IDs, and the file moves into today's date folder.

A reprint changes no batch: the orders stay saved where they were, the already-saved guard is unchanged,
and no printed mark moves. The Day screen lists the day's reprints with **Open**.

## Restore

The open-time check lists an output of a live batch that is missing
([save](../data/save.md#when-a-date-opens)).
**Restore** on its batch card writes it again; **Open** on a group PDF or summary PDF that is not
there (missing when the date opened, or deleted since) writes it again first, then opens it:

- a group PDF from the stored inputs and the files — the reprint path, its pages in `orders.csv` order;
- the summary PDF from the batch's facts in `day.toml`, in the program's current layout: deleting an old batch's
  summary PDF and pressing **Open** shows its data in today's layout.

The file is verified like the original (V2, V5 or V6), written through `.partial/` and committed like any change ([save](../data/save.md)).
