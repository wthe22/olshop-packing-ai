# The program's workflow

The whole program in order, from the start to a saved batch and what comes after it: what each step does,
the rule it follows and the algorithm behind it, and which claims it keeps. Read it top to bottom and check
each step against what the shop needs; a step that is wrong is a review item
([reviews/](../../reviews/README.md)). The words are in [Concepts](../requirements.md#concepts).

```
 1 Start ─► 2 Open workspace and date ─► 3 Choose files ─► 4 Read pages ─► 5 Copies and orders ─► 6 Checks, logo prompt
                                                                                                   │
 ┌─────────────────────────────────────────────────────────────────────────────────────────────────┘
 ▼
 7 Already saved ─► 8 Repeats ─► 9 Plan ─► 10 Order contents ─► 11 Store inputs ─► 12 Write ─► 13 Verify
                                                                                                   │
 ┌─────────────────────────────────────────────────────────────────────────────────────────────────┘
 ▼
 14 Commit ─► 15 Move outputs, views ─► 16 Printed · 17 Revert · 18 Amend · 19 Search, reprint · 20 Restore
```

| Steps | File | In → out |
|---|---|---|
| 1–2 | [01 Start and open](01-start-and-open.md) | Program folder → data folder, rules, lock → a folded date |
| 3–6 | [02 Read](02-read.md) | Chosen files → pages → copies → orders, warnings, courier names |
| 7–10 | [03 Plan](03-plan.md) | Orders → one copy each → label groups → order contents and file names |
| 11–15 | [04 Save](04-save.md) | Plan → inputs stored, outputs written and verified, the record committed |
| 16–20 | [05 After the save](05-after-save.md) | Printed marks, revert, amend, search and reprint, restore |

## What must always hold

| # | Always true |
|---|---|
| **I1** | Every page of an input belongs to exactly one copy, every copy to one order, and a copy's pages are consecutive in one input |
| **I2** | An order is written by at most one live batch of a workspace and date; a repeat or an order saved earlier is left out, never doubled |
| **I3** | Every order of a batch is in exactly one label group, so in one group PDF |
| **I4** | The ledger adds up: every page of every input of a batch is written exactly once or left out on purpose — *pages read = written + left out* |
| **I5** | An output page is its source page unchanged, and an order's pages are written together in their order |
| **I6** | Batch numbers are consecutive within a workspace and date; a `PdfID` names one input, a `sha256` one `PdfID` |
| **I7** | A batch is saved whole or not at all: the journal append is the only commit, and the outputs follow the journal |
| **I8** | The journal holds every saved fact once; each line is sealed, `seq` has no gap; nothing else decides |
| **I9** | An input is used only while its bytes have the SHA-256 the journal names |
| **I10** | The views and the search index say what the journal says |
| **I11** | A reprint holds exactly the chosen orders' pages, unchanged, in their original printing order |
| **I12** | The summary PDF's printed counts are the record's |

## Verification matrix

Who makes each claim true, and the independent check that proves it. The verifier shares no code with the
writer ([verification](../app/verification.md)); the store's checks re-read what is on disk.

| # | Made true by | Proved by | When |
|---|---|---|---|
| I1 | Engine: copies (step 5) | Verifier V3: the ledger over every input page | Every save |
| I2 | Engine: guard and repeats (steps 7–8) | Verifier V4: order IDs unique in the record and the date's live batches | Every save |
| I3 | Engine: the plan (step 9) | Verifier V4 and V5: each order in one group, its pages in that group's PDF only | Every save |
| I4 | Engine and the PDF writer (steps 9, 12) | Verifier V3 and V5: ledger and page digests | Every save |
| I5 | PDF writer (step 12) | Verifier V5: page digest at every position | Save, reprint, restore |
| I6 | Store: the fold numbers the batch (step 14) | Store: the fold's rules, at every open; verifier V1 | Open, save |
| I7 | Store: commit and move (steps 14–15) | Store: `.partial/` settled by the journal | Every open |
| I8 | Store: one sealed append (step 14) | Store fold: seals and `seq`; verifier V1 | Open, save |
| I9 | Store: inputs by content (step 11) | Store: SHA-256 at open; verifier V2 | Open, save, reprint |
| I10 | Store: views after every commit (step 15) | Store: `journal_seq` at open; the index is rebuilt at every open | Open |
| I11 | PDF writer (step 19) | Verifier V5 on the reprint | Every reprint |
| I12 | Summary writer (step 12) | Verifier V6: the counts read back as text | Save, restore |

What no check can prove — a label missing from the download, a filter that says the wrong thing — is what
the owner checks on the Plan screen, and what the known-answer tests pin
([code rules](../../development/code-rules.md#tests)).

## How to check a step

Each step lists what it does, its **Stops and warnings**, the claims it **Keeps**, and under **Check** the
questions whose answer is the owner's: *is this what the shop needs?*
