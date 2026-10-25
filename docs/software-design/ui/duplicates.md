# Repeated orders

A batch can hold the same order more than once: two downloads that overlap, or a label printed twice into one
file. The app finds these after reading, shows which files hold them and how many there are, groups the files
that repeat each other, and lets the owner choose which copy stays. How they are found is in
[duplicates](../app/duplicates.md), with the words *copy*, *repeat* and *group*. Only the label files of the
batch being prepared are looked at: a saved batch cannot hold a repeat, because the verifier proves it
([D-UI-6](../decisions/ui.md)).

## Where it appears

**Read labels** → the logo prompt, if any ([New batch](new-batch.md#the-logo-prompt)) → **Repeated orders**,
only when the batch holds a repeat → [Plan](plan.md).

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│  Repeated orders in this batch                                               [ Back ]  │
│  14 orders appear more than once · 14 extra copies · 14 extra pages · 3 of 5 files     │
├────────────────────────────────────────────────────────────────────────────────────────┤
│  ┌ Group 1 · files 1 and 3 · 10 orders ─────────────────────────────────────────────┐  │
│  │ File                       Pages   Copies   Kept   Left out                      │  │
│  │ 1  labels_1.pdf              200       10     10          0                      │  │
│  │ 3  labels_3.pdf              198       10      0         10                      │  │
│  │ (•) Keep the first copies   ( ) Keep the last copies   ( ) Take file 3 out       │  │
│  │ ▾ 10 orders                                              file 1  file 3          │  │
│  │   580000000000000042  JY0000001234  same on both             (•)   ( )           │  │
│  │ ⚠ 580000000000000099  JY0000001400  differs: JY…999 in 3     (•)   ( )           │  │
│  │   … 8 more                                                                       │  │
│  └──────────────────────────────────────────────────────────────────────────────────┘  │
│  ┌ Group 2 · file 2 only · 4 orders ────────────────────────────────────────────────┐  │
│  │ (•) Keep the first copies   ( ) Keep the last copies                             │  │
│  │ ▸ 4 orders, each twice in this file                                              │  │
│  └──────────────────────────────────────────────────────────────────────────────────┘  │
├────────────────────────────────────────────────────────────────────────────────────────┤
│  14 copies are left out; 986 of 1000 pages go on to the plan.  [ Continue to the plan ] │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

- **Summary line**: orders that appear more than once · extra copies · extra pages · files that hold a repeat.
- **A group card** is a set of files that repeat each other: one card is one problem solved on its own.
- **The file table**: the file's number in the batch, its pages, its copies of repeated orders, how many stay
  and how many are left out, following the choice below it.
- **The choice**: *Keep the first copies* (the default: the first in reading order), *Keep the last copies*,
  or *Take file 3 out*, offered for each file whose repeated orders all have a copy in another file. A file
  that also holds orders found nowhere else shows it disabled: "File 3 also holds 12 orders that are in no
  other file."
- **▾ orders** lists the repeated orders: Order ID, tracking ID, *same* or *differs*, and one radio per copy.
  A radio set by hand overrides the card's choice for that order (the card then reads *Chosen by hand*). A
  **⚠ differs** row names what differs; exactly one copy stays, there is no *keep both*
  ([D-APP-23](../decisions/app-features.md)).
- **Continue to the plan** applies every card's choice. The cards start on their defaults, so it works at
  once ([D-APP-22](../decisions/app-features.md)).
- **Back** returns to the file list with the files still listed; nothing is written. Choices made here are
  lost when **Read labels** is pressed again.

## After the choice

- The Plan header counts the copies left out: `… · 242 orders · 0 already saved · 14 copies left out ▸`;
  **▸** lists them (order, file, page, the copy kept); **Change…** returns here (plan edits are lost).
- The file list of [New batch](new-batch.md) marks each file that holds a repeat with its group chip (▌G1,
  an amber bar and the label, never colour alone) and the text "10 orders also in file 3".
- The saved batch's record keeps every copy left out; its Day card shows **Left out: 14 ▸**, and an amend
  preselects the kept copies.
