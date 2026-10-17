# Changing the last batch or the categories

## Revert and amend the last batch

Only the newest batch can be changed — later batches depend on it.

- **Revert** — undo the last batch. It asks:
  "Undo batch 2? Its 2 saved PDFs, packing list and download copies are deleted, and its 129
  orders can be saved again. If you already printed them, throw the printed labels away."
  [Cancel] [Undo batch 2]. The orders count as not saved again and the next batch reuses the
  numbers. Any ticks on its rows go with it. If a file is open in a PDF viewer, the app
  says which file to close and changes nothing.
- **Amend** — "I fixed a category, do the last batch again". It names the set the batch was packed
  with and asks:
  "Redo batch 2 with its set of categories (default)? Batch 2 was saved at 09:12. If you already
  printed it, print it again after saving." [Cancel] [Redo batch 2]. The batch's own label files are
  read again, with the set that batch used (whatever set you have used since), and the **Plan**
  screen opens (titled *Redo batch 2*, button *Save batch 2 again*). To change which PDFs go in,
  press **Back** (top): the file list shows the batch's PDFs already selected and its set fixed
  ("Set: default (fixed)"); add PDFs with **Add files…** or drop them, remove one with **×**, then
  press **Read labels** to open the Plan screen again. Removing a file leaves its orders unsaved, so
  the next batch picks them up again. **Save batch 2 again** writes it again over the old one;
  **Cancel amend** (at the bottom) or **Cancel** on the file list leaves the batch unchanged, and
  neither asks. A batch written again starts with its ticks cleared.

## Editing your categories

![The Categories screen](images/07-categories.png)

Open **Categories** (in the top bar, or **Edit categories** on the Plan screen). Categories are
your picks, in order: an order is taken by the first category whose condition matches. The last
category has no condition and takes the rest. **Edit categories** on the Plan screen opens the set
that batch uses; **Categories** in the top bar opens the set you used last.

- The **Set** row at the top lists your sets of categories and shows the file the chosen set is kept
  in ([Sets of categories](#sets-of-categories)). The counts **In batch 3** are the open batch's,
  counted with the set shown.
- Each category has a **code**, a **name** and a condition. The condition is built from boxes: a
  group is **All of these** (*and*) or **Any of these** (*or*) and holds rows of
  *field · operator · value*; **not** reverses a row or a group.
- **Text view** switches one category to typing the condition as text; **Boxes** switches back.
  **Edit file as text** shows the whole file. **Help: examples ▸** lists examples.
- **Save** writes the chosen set's file; **Cancel** leaves it unchanged. Your comments are kept.

Four example conditions (more in [the rules file](../software-design/rules-file/README.md)):

| You want | Condition |
|---|---|
| everything with "sepatu" in the product name | `name contains "sepatu"` |
| orders of 3 items or more | `total_quantity >= 3` |
| everything sent by J&T | `courier starts_with "J&T"` |
| spion or knalpot, but not sepatu | `not name contains "sepatu" and (name contains "spion" or name contains "knalpot")` |

## Sets of categories

A **set** is a named collection of categories. A batch is packed with exactly one set, chosen on the
[New batch](02-a-day.md#new-batch) screen; the set you used last is already chosen there, so a shop
that always uses one set never touches the control.

- The **Set** row at the top of Categories lists your sets and shows the file the chosen set is kept
  in. The set `default` is the data folder's `categories.toml`; every other set is a file in the data
  folder's `categories` folder, named after the set, so a set can be copied, renamed or edited by
  hand.
- **+ New set** asks for a name and writes that set's file, which starts with one category that
  takes the rest. **Delete set** asks first, then deletes that set's file; its categories fall back
  to that one catch-all category. The set `default` cannot be deleted.
- **In batch 3** counts the open batch with the set shown. When the set has no file yet, one
  category takes every order and the line **No picks yet: all orders go into one PDF set** is shown
  above it.
- A batch records the set it was packed with, so **Amend** redoes it with that same set
  ([Revert and amend the last batch](#revert-and-amend-the-last-batch)).
