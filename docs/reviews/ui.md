# Review: UI

The owner's feedback on the window and every screen. Open items only; a resolved item is deleted and
its answer goes into [../software-design/](../software-design/README.md).

## R-UI-1 First use creates no categories.toml by default

**Now.** On a first start with no data next to the program, the Start screen offers **Use this folder**,
and it writes the bundled `categories.toml` (the list of picks) and `couriers.toml` into that folder
([start.md](../software-design/ui/start.md)). The owner starts with a list the app chose.

**Wanted.** On a first use the app does not write `categories.toml`. `couriers.toml` is still written
as today. Without a `categories.toml`, the picks are unconfigured: one default entry with no condition
takes all orders, so a batch saves every order in one PDF set by default (like "save remaining orders").
The owner adds picks in Categories when wanted.

**Answer from the owner.** Only `categories.toml` is meant ("catalog" is that file). The default
behaviour of an unconfigured list is that all orders are picked.

**To fold into the design when the work is done.** [start.md](../software-design/ui/start.md) (Use this
folder writes `couriers.toml` only; the Start screen's first-use wording), and the rule that a missing
`categories.toml` means one catch-all entry, in [inputs.md](../software-design/app/inputs.md) or
[data-folder.md](../software-design/app/data-folder.md). The user guide's first start
([01-setup.md](../user-guide/01-setup.md)) follows.

**Open detail for the owner.**
- Whether the Day screen or Categories should say that the list is unconfigured and everything is one
  pick (for example, a line "No picks yet: all orders go into one PDF set").

## R-UI-2 Cancel on the Plan screen goes back to the PDF selection page

**Now.** After **Read labels**, the Plan screen's **Cancel** asks "Discard batch 3? Nothing has been saved."
and, on yes, throws the whole draft away: to change which PDFs the batch uses, the owner starts again from
*Add files…*. Amend opens the same Plan screen, so its **Cancel** also discards the redo.

**Wanted.** **Cancel** on the Plan screen does not discard. It goes back to the PDF selection page with the
files still listed, without a confirmation. The owner can then remove or add files, press **Read labels**
again, or leave with the selection page's **Cancel**. Only **Cancel** on the PDF selection page discards the
draft, as today. Amend follows the same rule: its Plan screen's **Cancel** returns to the amend selection
page ([R-APP-1](app.md)); the batch is untouched until **Save**.

**Reading to check against the design.** [new-batch.md](../software-design/ui/new-batch.md) (the Reading
section, which today says Cancel returns to the file list), [plan.md](../software-design/ui/plan.md) (the
Cancel line and its "Discard batch 3?" text), and [batch.md](../software-design/app/batch.md) (amend, step 3).
The draft lives only in memory until **Save**, so going back writes nothing.

**Answer from the owner.** The Plan edits (ticked or moved picks, added picks, file names) are lost when the
owner goes back and presses **Read labels** again: reading starts again from the files, as a first read does.
The next session writes this into the design files above and deletes this item.
