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
