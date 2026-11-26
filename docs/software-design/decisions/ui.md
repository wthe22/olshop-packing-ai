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

## D-UI-3 — Search: the viewed day by default, a range of dates, or all days

**Decision.** Search covers *Viewed day* (the date the Day screen shows, the default), *Today*, *Yesterday*,
*Last 7 days*, *Last 30 days*, a *Custom range* or *All days* (Q-UI-3), of every account; the ranges other
than *Viewed day* end today. *Viewed day* is left out while the viewed date is today. Never *This day*: the
owner found it could mean many things (R-UI-37). Every saved order is found by every field, since the
journals keep them all.

**Why.** The open date covers daily use; a reprint of an older label needs the other dates, and a short range
keeps a search of months of data small.

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
title bar has no theme menu (R-UI-31). The choice is `theme` in `packing-app.toml`; every screen and every
control (text boxes, lists, ticks, radio buttons) follows it, and so does the OS title bar. It replaces D-UI-7,
which put the menu in the title bar.

**Why.** The owner asked for a theme (R-UI-12) and found the controls left light in dark mode (R-UI-11); a
theme is chosen once, not every few minutes, so it belongs with the settings.

**Applies to.** [Settings](../ui/settings.md), [data](../data/README.md).

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

## D-UI-13 — The Accounts screen shows one account at a time

**Decision.** **Accounts** in the title bar, beside Search, Sort rules and Settings, opens a page in the look of
Sort rules: a tab per account and **+ New account**; the account's card with **Rename** and **Delete
account** in its head, its shops and its dates with its orders per date (Q-UI-9). Settings keeps the theme,
the data folder and *About*.

**Why.** The owner's words (2026-11-11): the daily order count and the facts of one account on a page of their
own, with the account's shops, rename and delete moved from Settings.

**Applies to.** [Accounts](../ui/accounts.md), [Settings](../ui/settings.md), [window frame](../ui/README.md#window-frame).

## D-UI-14 — A side panel of the data folder's dates

**Decision.** A panel that can be hidden lists the dates of the data folder, newest first, months folded except
the open and the current one; each date counts the orders of every account (Q-UI-10). A row's menu offers
**Open folder** only: a date has no name. It replaces D-UI-9, the panel of a workspace's dates.

**Why.** A date holds every account's batches, and the Day screen shows them all; one account's counts are on
the Accounts screen, so the panel needs no chooser.

**Applies to.** [Panel](../ui/panel.md).

## D-UI-15 — A batch on a date that is not today is asked for, and its screens show the date

**Decision.** On a date that is not today, the Day screen's **+ New batch** is muted and asks *Add a batch to
Wed, 7 Oct 2026?* — *Today is Mon, 23 Nov 2026. This adds the new batch to a past day, not to today.* [Add]
[Cancel] (R-UI-48, R-UI-51).
New batch, Repeated orders and Plan show the open date over their bar, as large as the Day screen's title,
with *today*, or *not today* in amber (R-UI-49).

**Why.** A batch is saved into the open date, and a date opened from the panel to look something up stays
open: the owner wants a batch there to be a choice, never a slip, and the date in view while it is made.

**Applies to.** [Day](../ui/day.md), [New batch](../ui/new-batch.md), [Plan](../ui/plan.md),
[Repeated orders](../ui/duplicates.md).


## D-UI-16 — Search shows each order as a card on columns all cards share; its first box is *Order ID contains*

**Decision.** Search's results are one card per order with no head row (R-UI-57). The card's head is two
lines on columns every card shares: the shop over the Order ID, the courier over the tracking ID, the date and
time its batch was saved over the batch, the ship-by date over the weight; then **Open** and the *Reprinted*
mark. Under it the items as a table: the quantity, the name, then the variation and the Seller SKU at fixed
widths, so they line up from card to card; the Qty Total at their foot when there are several lines. A bar
under the items opens the details: the receiver's address, the input PDF with its pages and short hash, the
group's code and name and the order's page in its PDF; the pickup code and the account only when the head
does not show them already; not the clue that named the courier. On a narrower window the date and the
parcel move under the shop and the courier. The filter and the dates share one box side by side. The first
box is *Order ID contains* (R-UI-50), not *starts with* (R-UI-33). It replaces D-UI-12, one row per order
under heads.

**Why.** The owner drew this card (2026-11-26): the row repeated facts in its details and kept the parcel's
weight out of sight. The cards the owner found hard to search through (2026-10-29) scattered the fields over
a grid; these keep each field in one column down the list, and fit the owner's 15-inch screen at 125 %.
*Starts with* surprised the owner: a part of an ID typed or pasted from the middle found nothing. The owner
looks an order up by its Order ID first (2026-11-23).

**Applies to.** [Search](../ui/search.md).
