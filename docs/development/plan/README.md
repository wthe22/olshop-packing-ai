# Plan

Only the unfinished work. A finished task is deleted from this folder in the commit that finishes it; a
finished phase moves to `archive/`. Phase and task numbers live only here.

## Phase 2 — PC app (Rust + Tauri)

A Windows program for the daily work: label downloads in, picks adjusted on screen, saved PDFs and packing
lists out, revert and amend of the last batch. The program is written in Rust, with the screens in Svelte +
TypeScript in a Tauri 2 window, as a portable program folder with no installer
([architecture](../../software-design/app/architecture.md)).

Reference for the reading and sorting rules: the Python package `tools/analysis/packing/` (frozen; the
first implementation of the same rules, tested). Port it module by module when a rule is missing
(`rules.py` → `rules.rs`, `slip.py` → `slip.rs`, …).

| Task | Content | File |
|---|---|---|
| 2.23 | Plan screen: Back at the top, no discard button, Cancel amend at the bottom | [2-23-plan-back.md](2-23-plan-back.md) |

Open choices of the owner, each with a default the app follows until the owner decides:
[Q-LABEL-1 to Q-LABEL-5, Q-LABEL-7 and Q-LABEL-8](../../software-design/questions/label.md) and
[Q-UI-1 to Q-UI-4](../../software-design/questions/ui.md),
[Q-APP-1](../../software-design/questions/app.md).

## Status

Task 2.23 is open (it follows 2.21 and the finished 2.22); every other task of the phase is
finished. The owner's review at the end of the phase waits for it and for the owner's answer to
R-APP-1.

**Next step.** 2.23 (the Plan screen's **Back** at the top, no discard button, an amend's **Cancel
amend** at the bottom). It ends with a commit that deletes its file from this folder, and a push to
`origin main`.

**Handoff for the review session.**

- **What to review:** the running app — a portable build, or `app/` under `npm run tauri dev` —
  against the checklist below; the first start writes no `categories.toml` (every order of a batch
  goes into one PDF set until the owner adds picks), and the Plan screen's **Back** / an amend's
  **Cancel amend** behave as [plan.md](../../software-design/ui/plan.md) says.
- **Where the answers go:** `docs/reviews/<scope>.md` (`label.md`, `rules.md`, `app.md`), one item
  per finding as `R-<SCOPE>-n` ([reviews/README.md](../../reviews/README.md)): what the owner sees and what
  the owner wants.
- **What the next session does with each answer:** a fix or a feature becomes a task file in this folder; an
  open choice becomes a question in [software-design/questions/](../../software-design/questions/README.md)
  with options and a default; a settled design change is written into `software-design/`. The review item is
  deleted once its answer is in the design.
- **The one thing the owner must answer:** [R-APP-1](../../reviews/app.md) (amend lets the owner add
  and remove the batch's label PDFs): where an added PDF comes from (any PDF on the PC, or only files
  already read on this day), and what happens to the orders of a removed PDF (left unsaved, or moved to
  another PDF). The owner writes the answer in [app.md](../../reviews/app.md).

**Decided, open in detail:** the app remembers taught logos ([D-LABEL-4](../../software-design/decisions/label.md)); where they are written is [Q-LABEL-7](../../software-design/questions/label.md), default `couriers.toml`.

## Owner review at the end of the phase

What the owner checks, after 2.15–2.23:

- Open the same day's packing list: the header must stand clear of the body; a `summary` or `full` list must
  print `×`, not `ÿ`.
- Press *Open folder* on a batch card (the batch folder must open, not Documents) and on the Settings screen
  (the data folder).
- The Day screen shows no *Courier unknown* warning (SiCepat and J&T Cargo orders are named by logo and
  tracking form). A label with a logo the app has not seen asks *Which courier is this logo?* before the Plan
  screen (2.16); the answer is written into the data folder's `couriers.toml`, so that logo is never asked
  about again and the file can be edited by hand.
- Save one batch: the check shows its progress and, on a clean day, reports nothing (2.18).
- Print one packing list.
- After 2.21: find one order by its tracking ID, one by an item name and one by a quantity, and check that
  *Open* opens that order's saved PDF.
- After 2.22: a first start writes no `categories.toml`; until a pick is added, every order of a batch goes
  into one PDF set.
- After 2.23: the Plan screen's top button is **Back** (it returns to the file list without discarding), and
  an amend's **Cancel amend** leaves the batch as it was.

The owner writes findings in [`../../reviews/`](../../reviews/README.md), one file per scope.

## Reading for a new session

In this order.

1. [../../../AGENTS.md](../../../AGENTS.md) and [../README.md](../README.md): rules, commit and push, repository layout.
2. The task files in this folder (2.23) and [../../reviews/README.md](../../reviews/README.md) with the item open there ([R-APP-1](../../reviews/app.md), amend).
3. [../../software-design/app/inputs.md](../../software-design/app/inputs.md) (inputs and `couriers.toml`),
   [batch.md](../../software-design/app/batch.md), [saved-pdf-check.md](../../software-design/app/saved-pdf-check.md),
   [data-folder.md](../../software-design/app/data-folder.md) (day folder and `state.json`),
   [architecture.md](../../software-design/app/architecture.md) and [program.md](../../software-design/app/program.md).
4. The screens: [../../software-design/ui/README.md](../../software-design/ui/README.md), then
   [day](../../software-design/ui/day.md), [new batch](../../software-design/ui/new-batch.md) (the logo prompt),
   [plan](../../software-design/ui/plan.md), [search](../../software-design/ui/search.md),
   [start](../../software-design/ui/start.md).
5. The label rules: [../../software-design/labels/README.md](../../software-design/labels/README.md) (per-order data,
   multi-page orders, packing slip, courier deduction, picks, packing list).
6. [../../software-design/questions/](../../software-design/questions/README.md): the open choices.
7. [../../software-design/requirements.md](../../software-design/requirements.md) for concepts and words.
8. [../../software-design/rules-file/conditions.md](../../software-design/rules-file/conditions.md): the condition
   language (fields, operators, the item-field "some line" rule).
9. [../../business-process/02-data-sources.md](../../business-process/02-data-sources.md): read before touching any CSV
   or PDF code; [01-packing-workflow.md](../../business-process/01-packing-workflow.md) for the daily steps.
10. [../setup.md](../setup.md), [../app.md](../app.md), [../cli.md](../cli.md), [../testing.md](../testing.md).
11. [../../user-guide/README.md](../../user-guide/README.md): update it in the commit that changes what the owner sees.
12. Code, after the documents: `crates/` (`engine`, `pdf`, `cli`), `app/` (Svelte in `src/`, Rust in `src-tauri/`),
    `testdata/` (made-up fixtures, `make_testdata.py`), `crates/engine/couriers.toml` (the bundled courier list);
    then `git status` and `git log --oneline -5`.

`samples/` and `work/` are confidential: never commit them or copy real data from them.
