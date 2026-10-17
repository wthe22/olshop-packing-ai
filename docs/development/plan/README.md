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

Every task of the phase (2.1–2.24) is finished, so this folder holds no task file. What the tasks built
is in [software-design/](../../software-design/README.md); how to run, build and test it is in
[../README.md](../README.md).

Open choices of the owner, each with a default the app follows until the owner decides:
[Q-LABEL-1 to Q-LABEL-5, Q-LABEL-7 and Q-LABEL-8](../../software-design/questions/label.md),
[Q-UI-1 to Q-UI-4](../../software-design/questions/ui.md) and
[Q-APP-1 to Q-APP-3](../../software-design/questions/app.md).

## Status

Phase 2 is finished: tasks 2.1–2.24 are done. The owner reviewed the running app and wrote three findings
([reviews/](../../reviews/README.md)):

- **R-UI-1** — the version in the window title and in *About*, for the release **v0.1** ([ui.md](../../reviews/ui.md)).
- **R-RULES-1** — several named sets of categories ([rules.md](../../reviews/rules.md)).
- **R-UI-2** — an optional checkbox beside each category row of the Day screen, saved, to mark it as printed (a note only) ([ui.md](../../reviews/ui.md)).

**Next step — the next session's:** turn each open item into work, as [development/README.md](../README.md) says:
R-UI-2 needs no question (the owner settled it): it is written into the Day screen design and the saved state, then a task; R-UI-1 becomes a task file in this folder (the number in one place, shown in both spots, then the portable
build); R-RULES-1 becomes a question in `software-design/questions/` (which set a batch uses, where the sets
are kept; options and a default), and the design is written before any code.

**Handoff.** Nothing is waiting for the owner. The open choices are the owner's to answer once the question
file exists; the next session writes the answers into `software-design/`, deletes each review item once its
answer is in the design, and then starts the tasks.

**Decided, open in detail:** the app remembers taught logos ([D-LABEL-4](../../software-design/decisions/label.md)); where they are written is [Q-LABEL-7](../../software-design/questions/label.md), default `couriers.toml`.

## Owner review at the end of the phase

What the owner checks, after 2.15–2.24:

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

The owner writes findings in [`../../reviews/`](../../reviews/README.md), one file per scope.

## Reading for a new session

In this order.

1. [../../../AGENTS.md](../../../AGENTS.md) and [../README.md](../README.md): rules, commit and push, repository layout.
2. [../../reviews/README.md](../../reviews/README.md) and the items open there: the owner's findings, to
   turn into tasks or questions. Two items are open (R-UI-1, R-RULES-1); no task file is left. The Status block above says
   what to do with them.
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
