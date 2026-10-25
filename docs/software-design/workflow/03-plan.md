# Steps 7–10: from orders to the plan

## 7. Orders already saved

- Every order whose Order ID is in a live batch of this namespace and date (the fold of step 2) is taken
  out with all its copies, each recorded as left out with the reason `saved_earlier` and that batch. A
  repeat of such an order is not asked about. An amend leaves its own batch out of this set.
- The orders taken out are one warning: tracking ID, batch, group and time.

**Stops:** every order of the batch was saved earlier: "All `<n>` orders were already saved today".

**Keeps:** I2 — no order is written by two live batches of a date.

**Check:** the guard is by Order ID, within one namespace and date. A cancelled order that the platform
reissues with the same Order ID on the same date would be left out as already saved.

## 8. Repeats

An order that still has two or more copies is a **repeat** ([duplicates](../app/duplicates.md)).

1. Count them (orders, extra copies, extra pages) and compare the copies by page fingerprint (*same* or
   *differs*).
2. Inputs that repeat each other form **groups**; the [Repeated orders](../ui/duplicates.md) screen opens
   with every group on *Keep the first copies*.
3. One copy of each repeat stays — the first in reading order, or the one the owner picks. The others are
   recorded as left out with the reason `repeat` and the copy kept.
4. Each order is rebuilt from its kept copy alone: its pages, item lines and `Qty Total` check, so a doubled
   sum never shows as a false mismatch.

**Keeps:** I2 — after this step no Order ID has more than one copy.

**Check:** is *keep the first copy* the right default?

## 9. The plan

The batch's **sort rule** gives the label group rules in file order ([groups](../labels/groups.md)).

1. `remaining` = every order, in reading position.
2. For each group rule in order, unless skipped for this batch: the orders of `remaining` that its filter
   matches are the group's and leave `remaining`. A rule with **no filter matches every order left**. A
   filter is true for an order by the [filter language](../rules-file/conditions.md): an item field is true
   when **one** of the order's lines matches; text ignores case.
3. What `remaining` holds at the end goes to the **rest**: the last rule when it has no filter (it has taken
   them already); else, or when that rule is skipped, a group `?` *Uncategorised*
   ([D-UI-1](../decisions/ui.md)).
4. A group that took **no** order is dropped: no file, no runs.
5. The owner changes the plan for this batch only: skip, move, add a one-off group, rename a file. Every
   change repeats 1–4 on the whole plan; `packing-workflows.toml` is not touched.

**Keeps:** I3 — an order is taken once, by the first group that matches it, and `remaining` is what the
rest takes: no order is in two groups and none is in none.

**Check:** the order of the groups decides where an order lands (first match wins).

## 10. Runs and file names

- **Packing key** of an order: its lines as (product name, variation, quantity), sorted. Orders with the
  same packing key are packed the same way ([D-APP-20](../decisions/app-features.md)). The Seller SKU and the printed
  `Qty Total` are stored with the order's content but are not part of the key.
- **Runs**: inside a group, orders with the same packing key are one run. Runs are sorted by number of
  orders (most first), then by packing key as text, name first; inside a run the orders keep their reading
  position. Runs are numbered `01`, `02`, … and written `<batch>-<code>-<run>` (`2-A-05`).
- **Printing order** of the batch: group by group in plan order, run by run, order by order; an order's pages
  in their own order. This is the order of `orders` in the record ([records](../data/records.md)).
- **File names**: `{batch}-{code} {name} x{count}.pdf` by the default file-name rule; the owner may change
  one; two outputs of a batch cannot share a name.

**Keeps:** I6 — the batch number is the fold's next one.

**Check:**

- Two orders with the same name and variation but a different Seller SKU fall into one run.
- Runs of equal size are sorted as text, so a name in capitals sorts before one in lower case; a quantity
  compares as a number.
