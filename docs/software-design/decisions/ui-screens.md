# Decisions: the batch screens

How New batch, the boxes and the summaries look. The window and the other screens are in [ui.md](ui.md).

## D-UI-17 — No examples in a box

**Decision.** An empty input box shows its name (*Account name*) or nothing; it never shows an example
(*e.g. Toko Alfa*). A box for a date shows its form, `yyyy-mm-dd`.

**Why.** The owner's words (2026-11-27): simple and minimal.

**Applies to.** [Accounts](../ui/accounts.md), [New batch › The shop prompt](../ui/new-batch.md#the-shop-prompt),
[Sort rules](../ui/sort-rules.md).

## D-UI-18 — Reading stays on the file list; a stop stands under its file

**Decision.** While labels are read, New batch keeps its drop area and file list, and the progress bar
stands under the list. A stop stands under the file it is about, whose row is marked red, and the files stay
listed; a stop about no one file stands under the list. The Plan screen's warnings stand under the name of
the file each is about.

**Why.** The owner's words (2026-11-27): a screen that moves away and a wall of messages with no place make
it hard to see which file is wrong and what to do.

**Applies to.** [New batch › Reading](../ui/new-batch.md#reading), [Plan](../ui/plan.md).

## D-UI-19 — A summary is read in the app, without a PDF

**Decision.** **Summary** on a batch card and **Day summary** open the [Summary](../ui/summary.md) screen:
the summary of a batch or of the date, with an account list, read from the date's files. Its **PDF** writes
and opens the PDF. A group PDF's row on the Day screen lists its order contents (**▸**).

**Why.** The owner's words (2026-11-27): a summary should be viewable without making PDFs; and a date whose
PDFs are gone ([archive](../app/archive.md)) must still show what it holds.

**Applies to.** [Summary](../ui/summary.md), [Day](../ui/day.md).

## D-UI-20 — A screen says what the owner needs now, in a line or two

**Decision.** A question or a progress card says what will happen and how much in one or two lines (*29
dates were saved by an older Packing*), never a list of every item it touches; progress is one line and a
bar. A daily screen shows only what the daily work needs: storage, sizes and other upkeep live on their own
page ([Archive](../ui/archive.md)).

**Why.** The owner's words (2026-11-27): the owner does not read walls of text, nor needs a folder's size
after every operation.

**Applies to.** [Start › Update the data folder?](../ui/start.md#update-the-data-folder), [Archive](../ui/archive.md),
[panel](../ui/panel.md).
