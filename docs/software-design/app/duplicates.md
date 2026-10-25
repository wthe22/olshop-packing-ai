# Repeated orders: how they are found and resolved

The screens are in [Repeated orders](../ui/duplicates.md). This file is the algorithm; the workflow's
steps 5 and 8 ([02 Read](../workflow/02-read.md#5-copies-and-orders),
[03 Plan](../workflow/03-plan.md#8-repeats)) use it.

| Word | Meaning |
|---|---|
| **Copy** | One appearance of an order's label in the inputs: its label page and the continuation pages right after it. A copy is `(PdfID, page_start, page_end)` |
| **Repeat** | An order with two or more copies in the batch |
| **Group** | A set of inputs that repeat each other ([Groups](#groups)); not a label group |
| **Left out** | A copy that is read but written into no group PDF |

## 1. Copies

The reader ([reading](../labels/reading.md)) walks the pages of each input in reading order and gives
every page its Order ID and whether it carries a **label part** — a tracking ID, or an image in the header
band (a courier's logo or a channel strip). A slip's continuation page carries neither
([business process 02](../../business-process/02-data-sources.md)) ([D-LABEL-12](../decisions/label.md)).

- A page with an Order ID **and** a label part **starts a copy** of that order.
- A page with no label part **continues** the copy of the page right before it, in the same input. Its
  Order ID, when it has one, must be that copy's.
- A first page of an input that would continue a copy, or a continuation page whose Order ID is another
  order's, stops the batch with the input and the page: a copy's pages are always together.

So a second copy right after the first (`A A`) is a repeat, and every page belongs to exactly one copy
(I1, [workflow](../workflow/README.md#what-must-always-hold)).

## 2. Orders already saved

An order saved in a live batch of this namespace and date loses **all** its copies, each recorded as left
out with the reason `saved_earlier` and the batch that saved it. A repeat of such an order is not asked
about.

## 3. Repeats and what they hold

For every order with two or more copies the app compares the copies page by page with the **page
fingerprint** ([reading](../labels/reading.md#page-fingerprint)): the copies are **the same** when every
page matches, else the order **differs**, and the app names the first difference it can (tracking ID, the
number of item lines, the page count). The numbers shown: orders that repeat, extra copies (beyond one per
order), extra pages, inputs that hold a repeat.

## Groups

An input holds a repeat when one of its copies belongs to a repeated order. Inputs are joined into a group
when one repeated order has copies in both; the groups are the connected sets, numbered `G1`, `G2`, … by
their lowest input. Every copy of an order lies in that order's group, so the groups never overlap and can
be solved one by one.

## Choosing

| Choice | What it leaves out |
|---|---|
| **Keep the first copies** (default) | For every order of the group, every copy but the first in reading order |
| **Keep the last copies** | Every copy but the last |
| **Take input *f* out** | Every copy of the group's repeated orders that lies in input *f*; an order whose only copy is in *f* stays. Offered only when each repeated order of *f* has a copy in another input of the group |
| **By hand** (an order's radio) | For that order, every copy but the chosen one |

Exactly one copy of each repeated order stays; the app never writes two and never merges them
([D-APP-23](../decisions/app-features.md)). The screen opens with every group on its default, so **Continue** works
at once ([D-APP-22](../decisions/app-features.md)).

## What the plan receives, and what is stored

Each order with **one** copy's pages and **one** copy's item lines; the `Qty Total` check runs on that
copy. The plan, the order contents and the numbers see the orders as if the repeats had never been in the
inputs.

Every copy left out is a `LeftOutCopy` of the batch's record ([records](../data/records.md)): the order,
where the copy is, the reason, and the copy kept (`repeat`) or the batch that saved it (`saved_earlier`).
The Day card lists them (**Left out ▸**), the verifier counts them in the ledger
([verification](verification.md)), and an amend preselects each repeat's kept copy, so the owner's choices
come back.
