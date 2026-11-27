# Plan

Which orders go into which group PDF. It opens with the sort rule used last applied in order (a proposal);
on a normal day the owner checks the counts and presses **Save batch**.

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│  Wed, 7 Oct 2026 (today)                                                               │
│  Batch 3 · Sort rule: [default ▾] · 2 files · 257 pages · 256 orders · 0 already saved [Back] │
│  Toko Alfa 200 · Toko Beta 56                                                          │
│  Warnings: 2 ▸                                                                         │
├────────────────────────────────────────────────────────────────────────────────────────┤
│  Use  Group                    Orders  Order contents  File name                       │
│  [✓]  3-A  Sepatu                 130               3  [3-A Sepatu x130          ]  ↑ ↓ ▸│
│            name contains "sepatu"                                                      │
│  [✓]  3-B  Spion & Knalpot         20               4  [3-B Spion Knalpot x20    ]  ↑ ↓ ▸│
│            not name contains "sepatu" and (name contains "spion" or name …)           │
│  [✓]  3-Z  Lainnya (the rest)     106              31  [3-Z Lainnya x106         ]      ▸│
│                                                                                        │
│  [Add Group]                                         [Edit sort rule]                  │
├────────────────────────────────────────────────────────────────────────────────────────┤
│  Summary PDF (all accounts): 3 Summary.pdf                         [ Save batch ]      │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

- **Rows** are the groups of the chosen sort rule ([The sort rule](#the-sort-rule)), in order; each takes its orders from what the rows above left ([03 Plan](../workflow/03-plan.md#9-the-plan)).
  Under the header the batch's accounts, each with its orders, most first. When the batch held repeats it also says `· 14 copies left out ▸`, with
  **Change** back to [Repeated orders](duplicates.md).
- **Use** unticked skips the group for this batch only: its orders fall to the rows below. Unticking the
  rest row sends its orders to a new last row `? Uncategorised`, so every order is still saved
  ([D-UI-1](../decisions/ui.md)).
- **↑ ↓** move a group for this batch only. The rest row is always last: the rule's last group when it has
  no filter (here *Z Lainnya*), else `? Uncategorised`.
- **Add Group** opens a small form: code (default `X`), name, filter (the boxes editor of
  [Sort rules](sort-rules.md), with the count it would take at that position). It is added above the rest
  row and can be moved.
- **Edit sort rule** opens [Sort rules](sort-rules.md) on the chosen rule (on a *Custom* plan, the rule it
  started from); a saved change rebuilds the plan from that rule.
- A group that takes 0 orders is grey with "0 orders — no file".
- **File name**: suggested `{batch}-{code} {name} x{count}` ([groups](../labels/groups.md#file-names)); the
  owner may change it. A name Windows does not allow, or one equal to another file of the batch (upper and
  lower case count as the same), shows why beside it and **Save batch** waits.
- **▸** expands a group to its order contents in printing order:
  `Sepatu Standar Samping Motor ×1 · 96 orders`.
- **Warnings ▸** lists the read warnings in plain words ([Messages](messages.md)), under the name of the file
  each is about (*labels_2.pdf, page 13*); one about no one file comes last.
- When the sort rule is one group with no filter, a line above the rows reads **No groups yet: all orders go
  into one PDF** ([D-UI-4](../decisions/ui.md)).
- Every change (tick, move, add, edit the sort rule) recounts all rows at once.
- **Back** returns to the file list ([New batch](new-batch.md)) without asking and writes nothing; only
  **Cancel** there discards the draft.
- An amend uses the same screen, titled *Redo batch 2*, with the same **Save batch** and a bottom **Cancel
  amend**; **Back** opens the amend's file list. Neither asks: an amend writes nothing before it is saved.

## The sort rule

**Sort rule** lists the rules of `config.toml` that are on ([rules file](../rules-file/README.md); a rule
switched off on the Sort rules screen is not listed, [D-RULES-5](../decisions/rules.md)). The rule of the
data folder's newest batch is preselected, on whichever date it was saved, so an owner who always uses one
rule never touches it. A rule that is off, gone, or *Custom* is never preselected: the rule it came from is,
else the first rule that is on ([D-RULES-7](../decisions/rules.md)). Choosing another rule builds the plan
again from it; changes made on this screen go.

A tick, a move or **Add Group** makes the plan **Custom** ([D-RULES-7](../decisions/rules.md)): the list
reads *Custom (from default)*. It never changes a rule of `config.toml`; `day.toml` keeps the batch's groups
as saved, under the name *Custom*. An amend preselects the rule of the batch's name, or its groups as saved
when it was *Custom* or no rule has that name any more ([D-RULES-2](../decisions/rules.md)).

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
│  · 3-A Sepatu x130.pdf: every page is there, but not in the printing order.            │
│  [ Save anyway ]   [ Cancel — nothing is saved ]                                       │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

**Save anyway** saves the batch with the findings on its Day card; **Cancel — nothing is saved** keeps the
draft on this screen.
