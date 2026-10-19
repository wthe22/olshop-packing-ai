# Plan

Only the unfinished work. A finished task is deleted from this folder in the commit that finishes it; a
finished phase moves to `archive/`. Phase and task numbers live only here.

## Next step — the code refactor

Before any new feature: bring the code to the [code rules](../code-rules.md) in the steps R0–R7 of
[refactor.md](refactor.md), without changing what the program does. The first step listed there is the
work; it ends with the owner's regression list in the handoff below. **The code rules apply to every task
file in this folder**: a task report states *size gate passed, no new lint suppression, coverage not lower*.

## Phase 2 — PC app (Rust + Tauri)

A Windows program for the daily work: label downloads in, picks adjusted on screen, saved PDFs and packing
lists out, revert and amend of the last batch. The program is written in Rust, with the screens in Svelte +
TypeScript in a Tauri 2 window, as a portable program folder with no installer
([architecture](../../software-design/app/architecture.md)).

Reference for the reading and sorting rules: the Python package `tools/analysis/packing/` (frozen; the
first implementation of the same rules, tested). Port it module by module when a rule is missing
(`rules.py` → `rules.rs`, `slip.py` → `slip.rs`, …).

Every task of the phase (2.1–2.29) is finished, so this folder holds no task file. What the tasks built
is in [software-design/](../../software-design/README.md); how to run, build and test it is in
[../README.md](../README.md).

Open choices of the owner, each with a default the app follows until the owner decides:
[Q-LABEL-1 to Q-LABEL-5, Q-LABEL-7 and Q-LABEL-8](../../software-design/questions/label.md),
[Q-UI-1 to Q-UI-4](../../software-design/questions/ui.md),
[Q-APP-1 to Q-APP-3](../../software-design/questions/app.md) and
[Q-RULES-1](../../software-design/questions/rules.md) (the sets of categories; task 2.27 builds its default).

## Status

Phase 2 is finished: tasks 2.1–2.29 are done and the portable program folder `target/portable/Packing/`
is built from them. The owner's three findings are built, and wait for the owner's test of the running
program (the checklist below):

- **R-UI-1** — the version *v0.1* in the window title, the title bar and *About*, from one place (2.25).
- **R-UI-2** — a saved **Printed** tick beside each saved-PDF row of the Day screen (2.26).
- **R-RULES-1** — several named sets of categories (2.27), built to the default of
  [Q-RULES-1](../../software-design/questions/rules.md).

**Dev review of the built program (done):** the Printed tick is as large as *Open* with no word (2.28), and a set of
categories is called a *Grouping* everywhere the owner reads it (2.29). Open: [R-RULES-2](../../reviews/rules.md)
(a grouping can be renamed) waits for the owner's answer to [Q-RULES-2](../../software-design/questions/rules.md); build
it after the answer is written into `software-design/`. **Next step — the owner's:** test the program folder with the
checklist below; findings go into [reviews/](../../reviews/README.md), one item each. A new session turns each into a
task (2.30 onward) or a question.

**Not checked on a running window** (only by tests): the Printed tick (click, close, reopen), the *Grouping*
control on New batch, the Categories grouping row (**+ New grouping**, **Delete grouping**), and the Plan header naming
the grouping. The seven in-app pictures of the user guide (`images/02-day.png` to `08-day-saved.png`,
`images/03-new-batch.png`, `05-plan.png`, `07-categories.png`) show the old title bar; `02-day.png` and
`08-day-saved.png` lack the **Printed** tick, `03-new-batch.png`, `05-plan.png` and `07-categories.png`
lack the grouping parts. They are retaken from the built program.

**Decided, open in detail:** the app remembers taught logos ([D-LABEL-4](../../software-design/decisions/label.md)); where they are written is [Q-LABEL-7](../../software-design/questions/label.md), default `couriers.toml`.

## Owner review at the end of the phase

What the owner checks, after 2.15–2.27 (the last three items are 2.25–2.27):

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
- After 2.24: **Amend** on the newest batch reads the batch's own label PDFs again and opens the Plan
  screen, titled *Redo batch 2*. **Back** shows the selection page with the batch's PDFs already listed.
  Remove one PDF and press **Read labels**: its orders are not in the plan. **Back** returns to the list with
  the files still there, and **Cancel amend** (or the list's **Cancel**) leaves the batch unchanged and asks
  nothing.
- After 2.25: the window title and the title bar read *Packing v0.1*, and Settings › *About* reads
  *Version v0.1*. The number appears nowhere else: changing the workspace `version` in `Cargo.toml` and
  building again moves all three.
- After 2.26: tick **Printed** on a row of a batch, close the app, open it again: the tick is still
  there. An unticked row opens and prints as before.
- After 2.27: **New batch** shows the *Grouping* control with the set used last preselected; a second set
  file in the data folder's `categories/` can be chosen and its categories make the plan; the Day
  screen's **Amend** redoes a batch with its own set; the data folder's `categories.toml` is still
  where it was.

The owner writes findings in [`../../reviews/`](../../reviews/README.md), one file per scope.

## Reading for a new session

In this order.

1. [../../../AGENTS.md](../../../AGENTS.md) and [../README.md](../README.md): rules, commit and push, repository layout.
2. [../../reviews/README.md](../../reviews/README.md) and the items open there: the owner's findings on the
   built program, to turn into tasks or questions. The items open there are the latest dev review.
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
