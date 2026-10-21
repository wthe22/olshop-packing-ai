# The program's workflow

The whole program in order, from opening a session to a saved batch: what each step does, the rule
it follows and the algorithm behind the rule. Read it top to bottom and check each step against what
the owner expects; a step that is wrong is a review item ([reviews/](../../reviews/README.md)). The
words are in [Concepts](../requirements.md#concepts); the screens are in [ui/](../ui/README.md).

```
 1 Open a session ─► 2 Choose files ─► 3 Read pages ─► 4 Copies and orders ─► 5 Checks and logo prompt
                                                                                         │
 ┌───────────────────────────────────────────────────────────────────────────────────────┘
 ▼
 6 Already saved ─► 7 Repeats ─► 8 Plan (picks) ─► 9 Runs and numbers ─► 10 Save (write) ─► 11 Check
                                                                                              │
 ┌────────────────────────────────────────────────────────────────────────────────────────────┘
 ▼
 12 Commit (rename, state.json) ─► 13 Day screen ─► 14 Revert or amend the last batch
```

| Step | File | In → out |
|---|---|---|
| 1 Open a session | [01](01-session-to-orders.md#1-open-a-session) | Data folder, session folder → `state.json` loaded |
| 2 Choose files | [01](01-session-to-orders.md#2-choose-the-files) | Paths → the list of files, fingerprints, hints |
| 3 Read pages | [01](01-session-to-orders.md#3-read-the-pages) | Files → per page: Order ID, label fields, slip lines, fingerprint |
| 4 Copies and orders | [01](01-session-to-orders.md#4-copies-and-orders) | Pages → copies → orders |
| 5 Checks and logo prompt | [01](01-session-to-orders.md#5-checks-and-the-logo-prompt) | Orders → warnings, stops, courier names |
| 6 Already saved | [02](02-orders-to-plan.md#6-orders-already-saved) | Orders → orders not saved in this session |
| 7 Repeats | [02](02-orders-to-plan.md#7-repeats) | Orders → one copy per order, copies left out |
| 8 Plan | [02](02-orders-to-plan.md#8-the-plan) | Orders, categories → picks with their orders |
| 9 Runs and numbers | [02](02-orders-to-plan.md#9-runs-and-numbers) | Picks → saved PDFs with runs and numbers |
| 10 Save | [03](03-save-and-check.md#10-write-the-batch) | Plan → files in a temporary folder |
| 11 Check | [03](03-save-and-check.md#11-the-check-before-saving) | Written files, input pages → findings |
| 12 Commit | [03](03-save-and-check.md#12-commit) | Temporary folder → batch folder, `state.json` |
| 13–14 Day, revert, amend | [03](03-save-and-check.md#13-the-day-screen-revert-and-amend) | The last batch undone or redone |

## What must always hold

These are the claims of the workflow; each step below says which it keeps.

| # | Always true |
|---|---|
| **I1** | Every input page belongs to exactly one copy, and every copy to exactly one order (steps 3–4) |
| **I2** | Every order reaches the plan exactly once: a repeat or an order saved earlier is left out, never doubled (steps 6–7) |
| **I3** | Every order of the plan is in exactly one pick, so in exactly one saved PDF (step 8) |
| **I4** | Every input page is written exactly once or left out on purpose: **pages read = pages written + pages left out** (steps 10–11) |
| **I5** | The pages of one order are written together, unchanged, in their original order (step 10) |
| **I6** | Saved-PDF numbers and batch numbers are unique in the session (step 9) |
| **I7** | A batch is written whole or not at all (steps 10–12) |

## How to check a step

Each step lists what it does, its **Stops and warnings** and, under **Check**, the questions whose
answer is the owner's: *is this what the shop needs?*
