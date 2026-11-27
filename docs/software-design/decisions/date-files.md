# Decisions: a date's files

When a date's files are changed, missing or older than this version. The files themselves are
[D-APP-39](app.md); the step that writes them again is [21 Read labels again](../workflow/05-after-save.md#21-read-labels-again).

## D-APP-40 — A changed or missing file: Accept, Read only or Read labels again

**Decision.** `orders.csv` or `order-contents.toml` whose hash `day.toml` does not hold, or that is missing
while `day.toml` is there, is named on the Day screen. **Accept** (a changed file only) takes it as it is when
it reads and agrees with the other two; **Read only** writes nothing to the date until the program starts
again; **Read labels again** writes both files again from the stored inputs of the live batches. It replaces
D-APP-38's Rebuild and Ignore.

**Why.** The owner's answer to Q-APP-25 (2026-11-27): B, and read again when a file is missing too. With no
journal, the stored label PDFs are the second copy: an Order ID Excel broke is read again from its label.

**Applies to.** [save › When a date opens](../data/save.md#when-a-date-opens), [Day](../ui/day.md).

## D-APP-41 — Check data names what this version would read, and reads labels again

**Decision.** **Check data** on the Day screen names each empty cell of the date's `orders.csv` that this
version reads from a label (the shop, the receiver's area, the weight), and offers **Read labels again**. A
cell the label leaves empty (no sender) is not named. A date whose input PDFs are missing names the cells as
not recoverable. Nothing is written in read-only mode, nor without the owner's choice.

**Why.** The owner's answer to Q-APP-24 (2026-11-27): B. A date saved by an older version can lack what a
label holds, and only the stored label PDFs can fill it.

**Applies to.** [Day](../ui/day.md), [21 Read labels again](../workflow/05-after-save.md#21-read-labels-again).

## D-APP-43 — Archive deletes a date's PDFs and keeps its data; one page holds it

**Decision.** **Archive** deletes every PDF of a date folder and keeps its data files. A date is archived when
none of its live batches' inputs is there; nothing is stored to say so. Everything about archiving and
storage is on one [Archive](../ui/archive.md) page, opened from Settings: each date's size, ticks for one date
or a range (Shift+click), and *Automatic* for dates older than a number of days (`archive_after_days` of
`packing-app.toml`, off unless the owner sets it). The daily screens show only that a date is archived: a
muted panel row, one line on its Day screen, *archived* on its orders in Search. Today is never archived.
It replaces D-APP-42 (archive and sizes on the panel and in Settings).

**Why.** The owner's words (2026-11-27): an old date needs its orders and summaries, not its PDFs; the size of
each folder is not needed after every operation, so storage belongs on a page of its own, not spread over
the daily screens.

**Applies to.** [archive](../app/archive.md), [Archive page](../ui/archive.md), [panel](../ui/panel.md),
[Day](../ui/day.md).
