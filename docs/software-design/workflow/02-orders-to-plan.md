# Steps 6–9: from orders to saved PDFs

## 6. Orders already saved

- Every order whose Order ID is in the session's `state.json` (`saved[]`) is taken out, with the
  batch, the saved-PDF number and the time it was saved ([picks](../labels/picks.md)). All its copies
  go: a repeat of an order saved earlier is not asked about.
- The orders taken out are a warning (Order ID, tracking ID, batch, PDF, time) and each copy is
  recorded as left out with the reason *saved earlier* ([duplicates](../app/duplicates.md#what-is-stored)).
- **Stop:** when every order of the batch was saved earlier, the batch stops ("All `<n>` orders were
  already saved today").

**Keeps:** I2 (no order is saved in two batches of one session).

**Check:** the guard is by Order ID only, in this session only
([Q-APP-5](../questions/sessions.md)). A cancelled order that the platform later reissues with
the same Order ID would be left out as already saved.

## 7. Repeats

An order that still has two or more copies is a **repeat** ([duplicates](../app/duplicates.md)).

1. The app counts them: orders, extra copies, extra pages; and compares the copies by page
   fingerprint (*same* or *differs*).
2. Files that repeat each other form **groups**; the owner sees [Repeated orders](../ui/duplicates.md).
3. One copy of each repeat stays — the first by default, or the one the owner picks. The others are
   recorded as left out with the reason *repeat*.
4. Each order is rebuilt from its kept copy alone: its pages, its item lines, its `Qty Total` check
   (step 5 runs on the kept copy, so a doubled sum never shows as a false mismatch).

**Keeps:** I2. After this step **no Order ID has more than one copy**.

**Check:** is *keep the first copy* the right default? (The first is the earliest in download order.)

## 8. The plan

The **plan** is the grouping's categories in file order ([picks](../labels/picks.md), [rules
file](../rules-file/README.md)); each entry is a **pick**.

1. `remaining` = all orders, in download order.
2. For each entry in order, unless skipped for this batch: the orders of `remaining` that match its
   condition are the pick's orders and leave `remaining`. A **no-condition entry matches every order
   left**. A condition is true for an order by the [condition language](../rules-file/conditions.md):
   an item field is true when **one** of the order's lines matches; matching ignores case.
3. What `remaining` holds at the end is the **rest** pick, last of all: when the grouping's last
   entry has no condition it has taken every order left already; else (or when that entry is skipped)
   a pick coded `?` named *Uncategorised* takes them.
4. A pick that took **no** order is dropped: it gets no number, no PDF and no row of orders.
5. The owner changes the plan for this batch only: skip, move, add a one-off pick, rename the file.
   Every change repeats steps 1–4 on the whole plan; the rules file is not touched.

**Keeps:** I3 — an order is taken once, by the first entry that matches it, and `remaining` is
what the rest pick takes; no order is in two picks and none is in none.

**Check:** the order of the entries decides which pick an order lands in (first match wins). With no
grouping file, one catch-all entry takes everything.

## 9. Runs and numbers

- **Saved-PDF numbers.** The first pick that took orders gets the session's next number (the last
  saved number + 1); each following pick gets the next. A skipped or empty pick takes none.
- **Contents** of an order: its lines as (product name, variation, quantity), sorted. **The seller SKU
  is not part of it.** Orders with equal contents are packed the same way.
- **Runs.** Inside a pick, orders with equal contents form a run. Runs are sorted by the number of
  orders (most first), then by contents in line order, name first, as plain text (upper case before
  lower case). Inside a run the orders keep their download position. Runs are numbered `01`, `02`, …
  in printing order and written `<pdf>-<run>` (`3-05`).
- **File names.** Suggested `<n> <code> <name> ×<orders>.pdf`; characters Windows does not allow
  become `-`; a name over 100 characters is cut; two saved PDFs of one batch cannot share a name.

**Keeps:** I6 — numbers continue from the session's last saved number, so they are unique in the
session.

**Check:**

- Two products with the same name and variation but different seller SKUs fall into one run.
- Runs of equal size are sorted by product name as text, so a name in capitals sorts before one in
  lower case; the quantity is compared as a number.
