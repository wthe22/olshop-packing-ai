# Plan

Which orders go into which group PDF. It opens with the batch's sort rule applied in order (a proposal);
on a normal day the owner checks the counts and presses **Save batch**.

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│  Batch 3 · rule default · 2 files · 257 pages · 256 orders · 0 already saved   [Back]  │
│  Warnings: 2 ▸                                                                         │
├────────────────────────────────────────────────────────────────────────────────────────┤
│  Use  Group                           Orders  Runs   File name                         │
│  [✓]  3-A  Sepatu                        130     3   [3-A Sepatu x130            ]  ↑ ↓ ▸│
│            name contains "sepatu"                                                      │
│  [✓]  3-B  Spion & Knalpot                20     4   [3-B Spion Knalpot x20      ]  ↑ ↓ ▸│
│            not name contains "sepatu" and (name contains "spion" or name …)           │
│  [✓]  3-Z  Lainnya (the rest)            106    31   [3-Z Lainnya x106           ]      ▸│
│                                                                                        │
│  [+ Add a group for this batch]                      [Edit sort rule]                  │
├────────────────────────────────────────────────────────────────────────────────────────┤
│  Summary PDF: 3 summary.pdf                                        [ Save batch ]      │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

- **Rows** are the groups of the batch's sort rule ([New batch › The sort rule](new-batch.md#the-sort-rule)),
  in order; each takes its orders from what the rows above left ([03 Plan](../workflow/03-plan.md#9-the-plan)).
  The header names the rule. When the batch held repeats it also says `· 14 copies left out ▸`, with
  **Change…** back to [Repeated orders](duplicates.md).
- **Use** unticked skips the group for this batch only: its orders fall to the rows below. Unticking the
  rest row sends its orders to a new last row `? Uncategorised`, so every order is still saved
  ([D-UI-1](../decisions/ui.md)).
- **↑ ↓** move a group for this batch only. The rest row is always last: the rule's last group when it has
  no filter (here *Z Lainnya*), else `? Uncategorised`.
- **+ Add a group for this batch** opens a small form: code (default `X`), name, filter (the boxes editor of
  [Sort rules](sort-rules.md), with the count it would take at that position). It is added above the rest
  row and can be moved. **Also add to the sort rule** writes it into `packing-workflows.toml` too.
- **Edit sort rule** opens [Sort rules](sort-rules.md) on this batch's rule; a saved change rebuilds the plan.
- A group that takes 0 orders is grey with "0 orders — no file".
- **File name**: suggested `{batch}-{code} {name} x{count}` ([groups](../labels/groups.md#file-names)); the
  owner may change it. A name Windows does not allow, or one equal to another file of the batch (upper and
  lower case count as the same), shows why beside it and **Save batch** waits.
- **▸** expands a group to its runs: `3-A-01  Sepatu Standar Samping Motor ×1    96 orders`.
- **Warnings ▸** lists the read warnings in plain words ([Messages](messages.md)).
- When the sort rule is one group with no filter, a line above the rows reads **No groups yet: all orders go
  into one PDF** ([D-UI-4](../decisions/ui.md)).
- Every change (tick, move, add, edit the sort rule) recounts all rows at once.
- **Back** returns to the file list ([New batch](new-batch.md)) without asking and writes nothing; only
  **Cancel** there discards the draft.
- An amend uses the same screen, titled *Redo batch 2*, with **Save batch 2 again** and a bottom **Cancel
  amend**; **Back** opens the amend's file list. Neither asks: an amend writes nothing before it is saved.

## Saving

**Save batch** writes, verifies and commits on its own thread ([04 Save](../workflow/04-save.md)); the stage
and a progress bar stand above the button:

```
│  Verifying the outputs …  page 143 of 257                                              │
│  ██████████████████████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░            │
```

The stages are *Copying inputs*, *Writing*, *Verifying*, *Saving*. A finding of the verifier stops the save
and asks, the errors and the warnings in two lists, never one ([verification](../app/verification.md#two-severities-never-one-list)):

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│  The saved files do not match the plan.                                                │
│  Errors: 1                                                                             │
│  · page 13 of labels_1.pdf (order 580000000000000001) is in no output.                 │
│  Warnings: 1                                                                           │
│  · 3-A Sepatu x130.pdf: every page is there, but not in the run order.                 │
│  [ Save anyway ]   [ Cancel — nothing is saved ]                                       │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

**Save anyway** saves the batch with the findings on its Day card; **Cancel — nothing is saved** keeps the
draft on this screen.
