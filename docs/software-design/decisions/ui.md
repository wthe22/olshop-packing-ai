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

## D-UI-3 — Search: this day by default, a range of dates, or all days of the workspace

**Decision.** Search covers *This day* (the default), *Yesterday*, *Last 7 days*, *Last 30 days*, a *Custom
range* or *All days* of the open workspace (Q-UI-3); the ranges other than *This day* end today. Every saved
order is found by every field, since the journals keep them all.

**Why.** The open date covers daily use; a reprint of an older label needs the other dates, and a short range
keeps a search of a workspace with months of data small.

**Applies to.** [Search](../ui/search.md), [reprint](../app/reprint.md#search).

## D-UI-4 — "No groups yet" on the Sort rules and Plan screens

**Decision.** When a sort rule is one group with no filter, the Sort rules screen and the Plan screen say **No
groups yet: all orders go into one PDF**; the Day screen does not (Q-UI-4).

**Why.** The line belongs where the groups are set and where the batch is saved.

**Applies to.** [Sort rules](../ui/sort-rules.md), [Plan](../ui/plan.md).

## D-UI-6 — Repeats are looked for in the batch's label files only

**Decision.** The Repeated orders screen and the file marks cover the label files of the batch being prepared;
there is no check of saved files for repeats (Q-UI-8).

**Why.** The verifier proves that a saved batch holds no order twice.

**Applies to.** [Repeated orders](../ui/duplicates.md).

## D-UI-8 — The theme is the owner's choice, in Settings

**Decision.** Settings › **Theme** offers **Light**, **Dark** and **System** (Windows' own, the default); the
title bar has no theme menu (R-UI-31). The choice is `theme` in `packing-settings.toml`; every screen and every
control (text boxes, lists, ticks, radio buttons) follows it, and so does the OS title bar. It replaces D-UI-7,
which put the menu in the title bar.

**Why.** The owner asked for a theme (R-UI-12) and found the controls left light in dark mode (R-UI-11); a
theme is chosen once, not every few minutes, so it belongs with the settings.

**Applies to.** [Settings](../ui/settings.md), [data](../data/README.md).

## D-UI-9 — A side panel of the workspace's dates

**Decision.** A panel that can be hidden lists the open workspace's dates, newest first, months folded except the
open and the current one (Q-UI-5); it replaces the title bar's day stepper (Q-UI-7). A row's menu offers **Open
folder** only: a date has no name. A workspace is renamed or deleted in Settings
([D-APP-24](app-features.md)). It replaces D-UI-5, under which the app renamed no workspace.

**Why.** The sessions panel's defaults, carried over to workspaces and dates (Q-UI-6 asked about renaming
sessions, which D-APP-5 removed).

**Applies to.** [Panel](../ui/panel.md).

## D-UI-11 — The look: warm neutrals, a clay accent, cards with depth

**Decision.** The colours follow Claude Code's own window: warm charcoal surfaces in layers with cream text in
the dark theme, ivory and white in the light one, a clay-orange accent for the main button, the open date and
the ticks, and the logo's blue for the group codes (`2-A`). Cards have rounded corners and a soft shadow; a
card's head is shaded; the Day screen's counts are pills beside its title; the **Printed** tick is green.
Every colour is a token of `app.css` (R-UI-34).

**Why.** The owner found the app "too dull even with decent layout, especially in dark mode" (2026-10-29):
the flat grey sets gave every surface the same weight, so nothing led the eye. A navy dark theme was tried
and turned down the same day; the owner named Zed and Claude Code as the look to follow.

**Applies to.** [Screens](../ui/README.md#the-look), every screen.

## D-UI-12 — Search shows each order as a compact row; its first box is *Tracking ID contains*

**Decision.** Search's results are one row per order (R-UI-35): the tick; the Order ID over the tracking ID;
the items, which take the width the other columns leave; the group PDF's code, name and page over the courier
and the date; **Open**, the trace chevron and the *Reprinted* mark. The ID column and the group PDF column are
as wide as their longest entry and no wider. On a narrower window the items move to a line of their own under
the IDs; narrower still, the group PDF line goes under the IDs too. The filter and the dates share one box
side by side. The first box is *Tracking ID contains*, not *starts with* (R-UI-33). It replaces D-UI-10, which
showed the results as cards.

**Why.** The owner found the cards hard to search through (2026-10-29): rows read down one column, cards
scatter the same field over a grid. The table before the cards (R-UI-30) gave each long ID a column of its own
and squeezed the items between eight columns on the owner's 15-inch screen at 125 %, often a window less than
full width; two IDs over each other and the group PDF over the courier leave the items most of a row.
*Starts with* surprised the owner: a part of a tracking ID typed or pasted from the middle found nothing.

**Applies to.** [Search](../ui/search.md).
