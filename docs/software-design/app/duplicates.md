# Repeated orders: how they are found and resolved

The screens are in [Repeated orders](../ui/duplicates.md). This file is the algorithm.

| Word | Meaning |
|---|---|
| **Copy** | One appearance of an order's label in the files: its label page and the continuation pages that follow it. A copy is `(file, first page, last page)` |
| **Repeat** | An order with two or more copies in the batch |
| **Group** | A set of files that repeat each other ([Groups](#groups)) |
| **Left out** | A copy that is read but not written into any saved PDF |

## 1. Copies

The reader ([reading.md](../labels/reading.md)) walks the pages in download order and gives every
page its Order ID and whether it carries a **label part** — a tracking ID, or an image in the
header band (the courier's logo or a channel strip). A slip's continuation page carries neither
([business process 02](../../business-process/02-data-sources.md)).

- A page with an Order ID **and** a label part **starts a copy** of that order.
- A page with an Order ID and no label part **continues** the copy that page's Order ID names: the
  previous page's copy when the Order ID is the same, else the order's latest copy.
- A page with no Order ID continues the previous page's copy; a first page of a file with none stops
  the batch, as before.

The rule is [Q-LABEL-9](../questions/duplicates.md)'s default: a repeat is told from a continuation
by the page's content, so a second copy right after the first (`A A`) is still a repeat. Every page
belongs to exactly one copy, and every copy to exactly one order — the reader checks it, and a page
that would belong to none stops the batch (the *1:0* side of [saved-pdf-check.md](saved-pdf-check.md)).

## 2. Orders already saved

An order saved earlier in the session ([picks.md](../labels/picks.md)) loses **all** its copies, each
recorded with the reason *saved earlier*. Only the orders left go on; a repeat of an order that was
saved before is not asked about.

## 3. Repeats and what they hold

For every order with two or more copies the app compares the copies page by page with the **page
fingerprint** ([reading.md](../labels/reading.md#page-identity)): the copies are **the same** when
every page matches, else the order **differs**, and the app names the first difference it can
(tracking ID, a different count of item lines, a different page count).

The numbers shown: *orders that repeat* (N), *extra copies* (the copies beyond the first of each
order), *extra pages* (the pages of those copies) and *files that hold a repeat*.

## Groups

A file holds a repeat when one of its copies belongs to a repeated order. Files are joined into a
group when one repeated order has copies in both; the groups are the connected sets, so files 1 and 3
that share 10 orders are one group, and file 2 that repeats only itself is a group of one.
Groups are numbered `G1`, `G2`, … by their lowest file number. Every copy of an order lies in the
group of that order, so the groups never overlap and can be solved one by one.

## Choosing

| Choice | What it leaves out |
|---|---|
| **Keep the first copies** (default) | For every order of the group, every copy but the first in download order |
| **Keep the last copies** | Every copy but the last |
| **Take file *f* out** | Every copy in file *f*. Offered only when each repeated order of *f* has a copy kept in another file, so no order is lost; a file that also holds orders found nowhere else is disabled |
| **By hand** (an order's radio) | For that order, every copy but the chosen one |

Exactly one copy of each repeated order stays. A repeated order whose copies differ is listed with ⚠
and starts on the same default; the app never merges two copies ([Q-APP-7](../questions/duplicates.md)).

## What the plan receives

The orders after the choice: each with **one** copy's pages and **one** copy's item lines (today's
doubled lines and quantities come from reading both copies into one order). The plan, the runs and
the numbering see the orders as if the repeats had never been in the files.

## What is stored

Each batch entry of `state.json` gains `left_out`, one entry per copy left out; absent loads as empty:

| Field | Holds |
|---|---|
| `order_id`, `tracking_id` | The order |
| `file`, `first_page`, `pages` | Where the copy is: the file's number in the batch's `files`, the page, the count |
| `reason` | `repeat` or `saved_earlier` |
| `kept` | For a repeat: the file and page of the copy that stays. For `saved_earlier`: the batch and saved PDF |

The Day card shows them (**Left out ▸**), the page-for-page check counts them
([saved-pdf-check.md](saved-pdf-check.md)), and an **Amend** ([batch.md](batch.md)) reads the batch's
files again and preselects each repeat's kept copy from `kept`, so the owner's choices come back.
