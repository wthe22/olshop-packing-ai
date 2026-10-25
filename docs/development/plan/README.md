# Plan

Only the unfinished work. A finished task is deleted from this folder in the commit that finishes it; a
finished phase moves to `archive/`. Phase and task numbers live only here. The code rules apply to every
task: a task report states *size gate passed, no new lint suppression, coverage not lower*.

## Status (2026-10-25)

- **The work is the v0.2 rework**: [v0.2-rework.md](v0.2-rework.md) holds the decisions agreed with the owner
  and the order of work, R1 to R9. **Next: R1.** A task gets its own file in this folder when it starts; until
  then its row in the order of work is the task.
- **No open choice waits for the owner.** The owner accepted the documented default of every question in
  [software-design/questions/](../../software-design/questions/README.md); R1 writes each into the design as a
  decision and empties that folder.
- **Kept for the rework**: [workflow-audit.md](workflow-audit.md) (where the built program differs from the
  workflow; R4 to R6 close it) and [2.32](2.32-text-object-split-word.md) (part of R4).
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
3. The owner's draft of the data: [../../reviews/design+workflow.md](../../reviews/design+workflow.md).
4. [../../software-design/workflow/README.md](../../software-design/workflow/README.md) (the steps and the
   invariants I1–I7), [saved-pdf-check.md](../../software-design/app/saved-pdf-check.md),
   [labels/README.md](../../software-design/labels/README.md).
5. [../code-rules.md](../code-rules.md) and [../testing.md](../testing.md).
6. [../../business-process/02-data-sources.md](../../business-process/02-data-sources.md) before touching
   any PDF code.
7. Code, after the documents: `crates/` and `app/`; then `git status` and `git log --oneline -5`.

`samples/` and `work/` are confidential: never commit them or copy real data from them.
