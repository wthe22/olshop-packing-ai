# Plan

Only the unfinished work. A finished task is deleted from this folder in the commit that finishes it; a
finished phase moves to `archive/`. Phase and task numbers live only here. The code rules apply to every
task: a task report states *size gate passed, no new lint suppression, coverage not lower*.

## Status (2026-10-25)

- **The work is the v0.2 rework**: [v0.2-rework.md](v0.2-rework.md) holds the decisions agreed with the owner
  and the order of work. **R1 is done**: [software-design/](../../software-design/README.md) describes v0.2,
  and every question open before it is answered by a decision record. **Next: R2.** A
  task gets its own file in this folder when it starts; until then its row in the order of work is the task.
- **Not blocked on the owner.** The owner may read the design at any time, starting with
  [the workflow](../../software-design/workflow/README.md) (each step against one page) and the points below;
  a finding goes into [reviews/app.md](../../reviews/app.md) under R-APP-2 and changes the design before the
  task that builds it.
- **The owner confirmed** (2026-10-25) the points R1 had to settle: the ship-by deadline is a date only
  (D-APP-21); orders are grouped by name, variation and quantity, the Seller SKU kept only for reference —
  names and SKUs can change during the day, and the order inside a group matters less than the grouping
  (D-APP-20); a `courier` filter compares the code; an amend uses the rule of the batch's name, else its
  snapshot (D-RULES-2); a continuation page must follow its label page (D-LABEL-12).
- **Open for the owner: [Q-LABEL-11](../../software-design/questions/label.md)**, what to call the orders with
  the same items inside a group PDF (now *run*). Answer in chat or in that file; then write the chosen word
  into the design (a decision record, the question removed) before R4 builds it. R2 and R3 do not wait for it.
- **Before R2: Node.** The coverage compare needs Node 26.7 (`setup.md`); the PC now has Node 24.20 in
  `C:\Program Files\nodejs`, which rejects `--test-coverage-include-all`. Install Node 26.7 again.
- **Kept for the rework**: [workflow-audit.md](workflow-audit.md) (where the built program differs from the
  design; R4 to R6 close it) and [2.32](2.32-text-object-split-word.md) (part of R4).
- The v0.1 program folder `target/portable/Packing/` is unchanged. Its regression list, in
  [the phase 2 plan](../../archive/handoff-2026-10-23.md), is where R8's checklist starts.

## Where to work

A **local** session on the owner's PC (Windows, Git Bash): it builds, runs `tools/make-portable.sh` and
commits. A cloud session cannot run commands on the PC; it can only read and write files there.

Outside Windows the packing-list tests fail, because the packing list loads its fonts from
`C:\Windows\Fonts` (ten tests of `packing-app`); that is not a regression.

## Reading for a new session

In this order.

1. [../../../AGENTS.md](../../../AGENTS.md) and [../README.md](../README.md): rules, commit and push, layout.
2. [v0.2-rework.md](v0.2-rework.md), then the task's own file when it has one.
3. [../../software-design/workflow/README.md](../../software-design/workflow/README.md) (the steps, the
   invariants I1–I12 and the verification matrix), then [data/](../../software-design/data/README.md) and the
   design files the task names.
4. [../code-rules.md](../code-rules.md) and [../testing.md](../testing.md).
5. [../../business-process/02-data-sources.md](../../business-process/02-data-sources.md) before touching
   any PDF code.
6. Code, after the documents: `crates/` and `app/`; then `git status` and `git log --oneline -5`.

`samples/` and `work/` are confidential: never commit them or copy real data from them.
