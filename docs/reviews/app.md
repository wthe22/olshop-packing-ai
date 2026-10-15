# Review: APP

The owner's feedback on batches, the data folder, revert and amend, and the program folder. Open items
only; a resolved item is deleted and its answer goes into [../software-design/](../software-design/README.md).

## R-APP-1 Amend lets the owner add and remove label PDFs

**Now.** Amend reads the batch's own label files again from its `download/` copy, in the stored order,
and opens the Plan screen with those orders. The owner cannot change which label PDFs the batch uses:
a PDF that should be in the batch but was not part of it, or one that should be left out, cannot be
fixed by amend.

**Wanted.** Amend first opens a page where the owner selects the label PDFs for the batch. The page
lists the batch's current PDFs, already selected. The owner can add PDFs (to the batch) and remove PDFs
(from the batch). Only then does the Plan screen open, with the orders of the selected PDFs, the current
`categories.toml`, and the duplicate guard as today. Save and Cancel behave as today.

**Reading to check against the design.** This changes the design of [batch.md](../software-design/app/batch.md)
(*Amend last batch*, step 1: the files are no longer only the batch's own) and of
[day.md](../software-design/ui/day.md) and [plan.md](../software-design/ui/plan.md) (the amend
confirmation leads to this page). It also touches the earlier owner answer that the batch's files come
from `download/` ("Add files only" in the first design round). The owner's answer decides which files
can be added: a file from Downloads, or only files already in the day's `download/` copies.

**Open detail for the owner.**
- Where an added PDF comes from: the file picker (any PDF on the PC), or only PDFs that were already
  read on this day.
- What happens to the orders of a removed PDF: they stay unsaved (and go to the next batch), or they
  must be moved to another PDF first.
