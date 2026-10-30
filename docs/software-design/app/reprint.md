# Search, reprint and restore

A label is sometimes lost or damaged after printing, and the shop must print it again — never twice by
mistake. The app finds saved orders, prints their labels again from the stored inputs, and writes a lost
output again ([D-APP-14](../decisions/app-features.md), [D-APP-15](../decisions/app-features.md)).

## The search index

- Built in memory when a workspace opens, on its own thread, by folding every journal of the workspace
  ([journal](../data/journal.md#the-fold)); after each commit the new record is added. A year of dates
  folds in about a second; Search shows *Loading…* until it is ready.
- One entry per order of a live batch: the fields a filter can use
  ([conditions](../rules-file/conditions.md#fields)), its date, batch and label group, the output file and
  the pages it fills there (the pages of the orders of its group before it, in printing order, plus one),
  and its **trace**: the input's file name and pages, and the courier's clue.
- Each date's reprints: which orders (by the date they were saved on and their Order ID) each reprint
  printed. Search marks those orders *Reprinted*, and the Day screen lists a reprint's orders from the
  index. A reprint's commit refreshes its date in the index like any other commit.
- The index is only read. A date whose journal is damaged is left out and named in the panel.

## Search

The [Search](../ui/search.md) screen takes a filter made with the boxes of the Sort rules screen and the
dates to search: a range (the viewed date by default) or all dates of the open workspace
([D-UI-3](../decisions/ui.md)); only the dates of the range are looked at. Each result row shows the order, its items, its date, batch and group
file and its page there, a tick, and when opened its details: the batch's save time and the input PDF and
pages it was read from:

```
Batch 2 · saved 09:12    from [10-07_07-00-00_Shipping label+Packing slip_1.pdf] p. 12–13
```

Orders of an open draft are not searched: they are not saved.

## Reprint

**Reprint** writes one PDF of the ticked orders' label pages.

1. **Sources.** For each chosen order, fold its date's journal (not the index) and take the copy its live
   batch wrote: the input's SHA-256 and the page range. An input that is missing or changed stops the
   reprint with its name.
2. **Order.** The orders in their original printing order: by date, batch, then their place in the
   batch's record.
3. **Write** the pages, unchanged, into `output/.partial/` of **today's** date of the workspace, named
   `reprint {n} x{count}.pdf`, `n` counting the reprints of today.
4. **Verify**: the file's page *k* has the digest of the source page at *k*, and the page count adds up
   ([verification](verification.md)). A finding stops the reprint; nothing is kept.
5. **Commit** `reprint_made` in today's journal, with every order's date, batch, input and pages; then
   move the file into `output/`.

A reprint changes no batch: the orders stay saved where they were, the already-saved guard is unchanged,
and no printed mark moves. The Day screen lists the day's reprints with **Open**.

## Restore

The open-time check lists an output of a live batch that is missing or changed
([journal](../data/journal.md#when-a-date-opens)).
**Restore** on its batch card writes a changed one again; **Open** on a group PDF or summary PDF that is not
there (missing when the date opened, or deleted since) writes it again first, then opens it:

- a group PDF from `input/` and the batch's record — the reprint path, its pages in the recorded order;
- the summary PDF from the batch's record, in the program's current layout: deleting an old batch's
  summary PDF and pressing **Open** shows its data in today's layout.

The file is verified like the original (V2, V5 or V6), written through `output/.partial/`, and committed as
`output_restored` with its new size and SHA-256, which the open-time check uses from then on.
