# Decisions: screens

Where a record names a former question (`Q-…`), the owner accepted its default.

## D-UI-1 — Unticking the rest sends its orders to `? Uncategorised`

**Decision.** Unticking **Use** on the rest row adds a last row `? Uncategorised` that takes its orders (Q-UI-1).

**Why.** Every order of the batch is still saved; to leave orders out, the owner changes the filters.

**Applies to.** [Plan](../ui/plan.md), [03 Plan](../workflow/03-plan.md#9-the-plan).

## D-UI-2 — No log file

**Decision.** There is no log file (Q-UI-2). *About* shows the versions; errors are shown on screen.

**Why.** The journal of each date already records every save, revert, mark and reprint.

**Applies to.** [Settings](../ui/settings.md).

## D-UI-3 — Search: this day by default, or all days of the namespace

**Decision.** Search's scope is *This day* (the default) or *All days* of the open namespace (Q-UI-3). Every saved
order is found by every field, since the journals keep them all.

**Why.** The open date covers daily use; a reprint of an older label needs the other dates.

**Applies to.** [Search](../ui/search.md), [reprint](../app/reprint.md#search).

## D-UI-4 — "No groups yet" on the Sort rules and Plan screens

**Decision.** When a sort rule is one group with no filter, the Sort rules screen and the Plan screen say **No
groups yet: all orders go into one PDF**; the Day screen does not (Q-UI-4).

**Why.** The line belongs where the groups are set and where the batch is saved.

**Applies to.** [Sort rules](../ui/sort-rules.md), [Plan](../ui/plan.md).

## D-UI-5 — A side panel of the namespace's dates

**Decision.** A panel that can be hidden lists the open namespace's dates, newest first, months folded except the
open and the current one (Q-UI-5); it replaces the title bar's day stepper (Q-UI-7). A row's menu offers **Open
folder** only: a date has no name, and the app renames no namespace, whose name is in its journals (Q-UI-6 asked
about renaming sessions, which D-APP-5 removed).

**Why.** The sessions panel's defaults, carried over to namespaces and dates.

**Applies to.** [Panel](../ui/panel.md).

## D-UI-6 — Repeats are looked for in the batch's label files only

**Decision.** The Repeated orders screen and the file marks cover the label files of the batch being prepared;
there is no check of saved files for repeats (Q-UI-8).

**Why.** The verifier proves that a saved batch holds no order twice.

**Applies to.** [Repeated orders](../ui/duplicates.md).

## D-UI-7 — The theme is the owner's choice

**Decision.** The title bar's theme menu offers **Light**, **Dark** and **System** (Windows' own, the default). The
choice is `theme` in `packing-settings.toml`; every screen and every control (text boxes, lists, ticks, radio
buttons) follows it, and so does the OS title bar.

**Why.** The owner asked for it (R-UI-12), and found the controls left light in dark mode (R-UI-11).

**Applies to.** [Window frame](../ui/README.md#window-frame), [data](../data/README.md).
