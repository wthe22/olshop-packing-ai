# Review: UI

The owner's feedback on the window and every screen. Open items only; a resolved item is deleted and
its answer goes into [../software-design/](../software-design/README.md).

## R-UI-1 First use creates no catalog by default

**Now.** On a first start with no data next to the program, the Start screen offers **Use this folder**,
and it writes the bundled `categories.toml` (the list of picks) and `couriers.toml` into that folder
([start.md](../software-design/ui/start.md)). The owner starts with a list the app chose.

**Wanted.** On a first use the app does not create a catalog by default: no `categories.toml` is written
without the owner's choice. The owner starts from an empty list of picks and makes the picks in
Categories.

**Reading to check against the design.** "Catalog" is read here as `categories.toml`, the list of picks.
If the owner means `couriers.toml` too (the bundled courier list), the same rule applies to it; the
owner should say which.

**Open detail for the owner.**
- With no `categories.toml`, the Start screen still asks for a folder; what it offers instead of
  **Use this folder** (for example, **Create an empty list** or **Choose a folder…**).
- A batch cannot be planned without picks: the Plan screen should say so and open Categories.
- Whether the bundled files stay available as an example to copy from.
