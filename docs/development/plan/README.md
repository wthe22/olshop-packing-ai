# Plan

Only the unfinished work. A finished task is deleted from this folder in the commit that finishes it; a
finished phase moves to `archive/`. Phase and task numbers live only here. The code rules apply to every
task: a task report states *size gate passed, no new lint suppression, coverage not lower*.

## Status (2026-10-25)

- **The work is the v0.2 rework**: [v0.2-rework.md](v0.2-rework.md) holds the decisions agreed with the owner
  and the order of work. **R1 is done**: [software-design/](../../software-design/README.md) describes v0.2,
  every open question is answered by a decision record, and the questions folder is empty. **Next: R2.** A
  task gets its own file in this folder when it starts; until then its row in the order of work is the task.
- **Not blocked on the owner.** The owner may read the design at any time, starting with
  [the workflow](../../software-design/workflow/README.md) (each step against one page) and the points below;
  a finding goes into [reviews/app.md](../../reviews/app.md) under R-APP-2 and changes the design before the
  task that builds it.
- **Points of R1 for the owner to check** (each follows the owner's draft or an accepted default; say if one
  is wrong):
  1. The ship-by deadline is a **date only**, as the draft's `ship_by_date` says, not Q-APP-9's default of an
     optional time ([D-APP-21](../../software-design/decisions/app-features.md)).
  2. Runs group orders by name, variation and quantity (Q-APP-8's default); the draft's stored content also
     holds the Seller SKU and the printed total, so `session.toml` gains a `run` per content
     ([views](../../software-design/data/views.md)).
  3. A run is written `2-A-05` (batch, group, run), following the draft's file name `{batch}-{code} …`.
  4. A filter on `courier` compares the courier's **code** (`jnt`), picked by name on screen.
  5. An amend uses the sort rule of the batch's name as it is now, or the batch's snapshot when the name is
     gone ([D-RULES-2](../../software-design/decisions/rules.md)).
  6. A continuation page must follow its label page; anywhere else it stops the batch
     ([D-LABEL-12](../../software-design/decisions/label.md)).
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
