# Repeated orders

A batch can hold the same order more than once: two downloads that overlap, a file added under two
names, a label printed twice into one file. The app finds these after reading, shows which files hold
them and how many there are, groups the files that repeat each other, and lets the owner choose which
copy stays. How the repeats are found is in [duplicates](../app/duplicates.md); the words *copy*,
*repeat* and *group* are defined there.

## Where it appears

**Read labels** → the logo prompt, if any ([New batch](new-batch.md)) → **Repeated orders**, only when
the batch holds a repeat → [Plan](plan.md). With no repeat the screen never shows.

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│  Repeated orders in this batch                                               [ Back ]  │
│  14 orders appear more than once · 14 extra copies · 14 extra pages · 3 of 5 files     │
├────────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                        │
│  ┌ Group 1 · files 1 and 3 · 10 orders ─────────────────────────────────────────────┐  │
│  │ File                       Pages   Copies   Kept   Left out                      │  │
│  │ 1  labels_1.pdf              200       10     10          0                      │  │
│  │ 3  labels_3.pdf              198       10      0         10                      │  │
│  │ (•) Keep the first copies   ( ) Keep the last copies   ( ) Take file 3 out       │  │
│  │ ▾ 10 orders                                              file 1  file 3          │  │
│  │   580000000000000042  JY0000001234  same on both             (•)   ( )           │  │
│  │   580000000000000057  JY0000001301  same on both             (•)   ( )           │  │
│  │ ⚠ 580000000000000099  JY0000001400  differs: JY…999 in 3     (•)   ( )           │  │
│  │   … 7 more                                                                       │  │
│  └──────────────────────────────────────────────────────────────────────────────────┘  │
│  ┌ Group 2 · file 2 only · 4 orders ────────────────────────────────────────────────┐  │
│  │ File                       Pages   Copies   Kept   Left out                      │  │
│  │ 2  labels_2.pdf              200        8      4          4                      │  │
│  │ (•) Keep the first copies   ( ) Keep the last copies                             │  │
│  │ ▸ 4 orders, each twice in this file                                              │  │
│  └──────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                        │
├────────────────────────────────────────────────────────────────────────────────────────┤
│  14 copies are left out; 986 of 1000 pages go on to the plan.                          │
│                                                              [ Continue to the plan ]  │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

- **Summary line.** Orders that appear more than once · extra copies (copies beyond the first of each
  order) · extra pages · files that hold a repeat.
- **A group card** is a set of files that repeat each other ([duplicates](../app/duplicates.md#groups)):
  group 1 is files 1 and 3 (the same 10 orders in both); group 2 is file 2 alone (4 orders twice inside
  it). One card is one problem the owner solves on its own.
- **The file table** of a card: the file's number in the batch, its pages, how many copies of repeated
  orders it holds (*Copies*), how many of them stay (*Kept*) and how many are left out (*Left out*).
  The numbers follow the choice below the table.
- **The choice of a card** is one of: *Keep the first copies* (the default, the copy that comes first in
  download order), *Keep the last copies*, or *Take file 3 out*, offered for each file of the card
  whose repeated orders all have another copy elsewhere. A file that holds orders found nowhere else
  shows the choice disabled, "File 3 also holds 12 orders that are in no other file."
- **▾ orders** lists the repeated orders of the card: Order ID, tracking ID, whether the copies are the
  same, and one radio per copy to keep (*file 1*, *file 3*). A radio set by hand overrides the card's
  choice for that order and the card then reads *Chosen by hand*. A **⚠ differs** row is an order
  whose copies are not the same page (a different tracking ID, other item lines): it names what
  differs, and the owner decides ([Q-APP-7](../questions/duplicates.md): there is no *keep both*).
- **Footer.** What the choices give: the copies left out and the pages that go on to the plan.
  **Continue to the plan** opens [Plan](plan.md) with every card's choice applied. The cards start on
  their defaults, so the owner can press it at once ([Q-APP-6](../questions/duplicates.md)).
- **Back** returns to the file list ([New batch](new-batch.md)) with the files still listed; nothing is
  written. Choices made here are lost when **Read labels** is pressed again, as the Plan's are.

## The file list marks the files

After a read, the file list of [New batch](new-batch.md) marks every file that holds a repeat. The
same marks show before any read for a file whose bytes equal another file of the list.

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│  New batch 3   Grouping: [default ▾]                                    [Cancel]       │
│                                                                                        │
│  ⚠ 3 of 5 files hold repeated orders: groups G1, G2        [ Review the repeats ]      │
│                                                                                        │
│  1  labels_1.pdf     200 pages 08:41   ▌G1  10 orders also in file 3               [×] │
│  2  labels_2.pdf     200 pages 08:42   ▌G2   4 orders twice in this file           [×] │
│  3  labels_3.pdf     198 pages 08:43   ▌G1  10 orders also in file 1               [×] │
│  4  labels_4.pdf     200 pages 08:44                                               [×] │
│  5  labels_5.pdf     202 pages 08:44   same file as 4                              [×] │
│                                                                                        │
│  Selected: 5 files · 1000 pages                                        [ Read labels ] │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

- **▌G1** is the group chip: an amber bar and the group's label, so the mark never relies on colour.
  The text beside it says how many orders repeat and where.
- **same file as 4** appears at once, before **Read labels**, when two listed files have the same
  fingerprint ([data-folder](../app/data-folder.md)); the second one's **×** is the usual fix.
- **Review the repeats** opens **Repeated orders** again, after a read.

## After the choice

- The Plan header counts the copies left out: `Batch 3 · … · 242 orders · 0 already saved · 14 copies
  left out ▸`; **▸** lists them (order, file, page, the file that kept the copy). **Change…** returns
  to **Repeated orders**; the plan edits made so far are lost, as with **Back**.
- The Day screen's batch card shows **Left out: 14 copies ▸** beside *Warnings ▸*, from the batch's
  `state.json` ([duplicates](../app/duplicates.md#what-is-stored)).
- A saved batch can never hold an order twice, so there is no screen to find repeats in saved PDFs; the
  page-for-page check keeps it that way ([saved-pdf-check.md](../app/saved-pdf-check.md)). Checking
  the saved PDFs of an old batch is [Q-UI-8](../questions/duplicates.md).
