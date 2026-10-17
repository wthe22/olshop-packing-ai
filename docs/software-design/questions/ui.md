# Open questions: screens

Questions: Q-UI-1, Q-UI-2, Q-UI-3, Q-UI-4.

## Q-UI-1 — Unticking the rest row

**Words.** The *rest row* is the last row of the [Plan screen](../ui/plan.md): the last
entry of the batch's set when it has no condition (e.g. *Z Lainnya*), else `? Uncategorised`. Unticking
**Use** on a row skips that pick for this batch: its orders fall to the rows below. The rest row has
no row below it.

**Example.** Batch 3 has 256 orders: 130 Sepatu, 20 Spion & Knalpot, 106 for *Z Lainnya*. The owner
unticks *Z Lainnya*.

| Option | What the owner sees |
|---|---|
| **1. The 106 orders go to `? Uncategorised`** (**Default**) | A new last row `? Uncategorised` with 106 orders; the batch still saves all 256 orders |
| 2. The rest row has no tick | *Z Lainnya* is always used; to leave orders out, change the conditions |
| 3. The 106 orders are left out of this batch | They are not saved; they count as "not saved today" and come back in the next batch with the same labels |

**What the answer changes:** the plan count in `plan.rs` and the Plan screen.

## Q-UI-2 — A log file

**Words.** The [Settings screen](../ui/settings.md) has an *About* row with **Open log file**. A log
file would be a text file in the data folder where the app writes what it did and any unexpected
error (e.g. `log.txt`: "07:40:12 batch 1 saved, 3 files, 600 orders"). Nothing else in the design
writes one; every batch already has its `summary.txt`.

| Option | What the owner sees |
|---|---|
| **1. No log file** (**Default**) | *About* shows the version and the PDFium version only; errors are shown on screen |
| 2. `log.txt` in the data folder | One line per batch saved, reverted or amended, plus unexpected errors; **Open log file** opens it |

**What the answer changes:** the *About* row of Settings and one small writer in the app.

## Q-UI-3 — Search: which days, and what an order saved before search can show

**Words.** The [Search panel](../ui/search.md) finds saved orders by the fields the rules can use
(tracking ID, item name, quantity, courier, …). Each saved order keeps those fields in `state.json`
([the data folder](../app/data-folder.md)). An order saved before search keeps only its IDs there;
its items can be read again only from its `download/` copy.

| Option | What the owner sees |
|---|---|
| **1. *This day* by default, *All days* as the other choice; an older order shows its IDs only** (**Default**) | Scope *This day* (the day in the title bar) or *All days* (every day folder's `state.json`). An order saved before search is found by Order ID or tracking ID only, and its row says *items not stored* |
| 2. *This day* only | No scope control. Cheapest; an order from an earlier day cannot be found in the app |
| 3. *All days*, older orders read again from their `download/` copy | Every order is searchable, old ones included. Each search over old days re-reads label files (about 4 MB per 600-order day), so searching many days is slow |

Recommended: **1** — the day on screen covers daily use, and the all-days search reads only small
JSON files; the IDs of an older order are still found.

**What the answer changes:** the scope control and the *items not stored* row of the Search panel,
and whether search reads `download/` at all.

## Q-UI-4 — Saying that the picks are unconfigured

**Words.** With no picks set configured, the picks are one catch-all entry that takes every order, so a
batch saves every order in one PDF set ([inputs](../app/inputs.md)); the owner adds picks in
[Categories](../ui/categories.md) when wanted. A line on screen can read the plain words
**No picks yet: all orders go into one PDF set**.

**Example.** The owner's first start writes no `categories.toml` ([Start](../ui/start.md)). One batch
is read: the plan has one row and every order goes into one PDF set.

| Option | What the owner sees |
|---|---|
| **1. The Categories screen and the Plan screen show the line; the Day screen does not** (**Default**) | Opening Categories before a pick is added, or preparing a batch, shows **No picks yet: all orders go into one PDF set** — where the picks are set and where the batch is saved. The Day screen stays as it is |
| 2. Every screen shows the line | The Day screen shows it too, until a pick is added; the owner sees it without opening Categories or a batch |
| 3. No line | Nothing extra; the Plan screen's one row (the catch-all entry, holding all the batch's orders) is read as everything in one PDF set |

**What the answer changes:** one line on the Categories screen, the Plan screen and the Day screen.
