# Archive

A date kept with as little storage as possible ([D-APP-42](../decisions/date-files.md)). **Archive** deletes
every PDF of the date folder: the label PDFs (inputs), the group PDFs, the summary PDFs, the reprints and the
summaries made on request. Every other file stays: `day.toml`, `orders.csv`, `order-contents.toml`, and an
update folder ([D-APP-36](../decisions/versions.md)).

## Which date is archived

A date is **archived** when it has a live batch and none of its live batches' inputs is in the date folder.
Nothing is stored to say so. A date with some inputs missing is not archived: it shows the missing input
([Messages](../ui/messages.md)). Today is never archived, by hand or on its own.

## Archiving

- **By hand**: **Archive** in a date's row menu on the [panel](../ui/panel.md), or **Archive** of a range of
  dates in [Settings › Storage](../ui/settings.md#storage). Each asks first, naming the dates, the number of PDFs
  and their size: *Archive Wed, 7 Oct 2026? Its 14 PDFs (210 MB) are deleted. Its orders and summaries stay; it
  can no longer be printed, amended, reverted, reprinted or read again.* [Archive] [Cancel]
- **On its own**: when Settings › Storage › *Archive automatically* is on, every date older than its number
  of days (counted from today) that is not archived yet is archived at each start, after the migration's
  question ([01 Start](../workflow/01-start-and-open.md#1-start)). Turning it on, or a lower number, asks first
  as above and archives at once. The number is `archive_after_days` of `packing-app.toml`
  ([data](../data/README.md#the-files)); missing: off.
- **How**: under the data folder's lock ([One writer](../data/README.md#one-writer)), date by date, oldest
  first: the outputs first, the inputs last, so a date that stops half way is not archived and can be archived
  again. A file Windows refuses to delete (open in a viewer) is left and named; the rest go on. More than one
  date shows its progress: *Archiving: 2026-10-07, 3 of 18*.
- A date with `.partial/`, with no live batch, or opened read-only is not archived. Read-only mode archives
  nothing and does not show **Archive**.

## An archived date

- **Panel**: its row is muted with the archive mark and its size ([panel](../ui/panel.md)).
- **Day screen**: a muted line *Archived: the PDFs of this date were removed.* Each batch card keeps its
  groups, counts, order contents, Sent times, printed ticks, warnings and **Left out**; **Print**, **Open**,
  **Restore**, the missing marks, **Amend**, **Revert** and **+ New batch** are not shown. **Day summary** and
  the summaries still work: they are read from the date's files.
- **Search**: its orders are found and shown; they have no tick for **Reprint** and no **Open**; the trace
  names the input and pages they came from.
- The already-saved guard still counts its orders as saved.
