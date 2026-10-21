# Plan

Only the unfinished work. A finished task is deleted from this folder in the commit that finishes it; a
finished phase moves to `archive/`. Phase and task numbers live only here.

## Next step — the owner's test of the built program

The code refactor (steps R0 to R7) and the two changes of 2026-10-21 (2.30, 2.31) are done and
committed; the program does what it did. The portable folder `target/portable/Packing/` is built from
them. **Test it with the list below, then write findings into
[reviews/](../../reviews/README.md), one item each; a new session turns each into a task.** Until then
no feature work starts. The code rules apply to every task file in this folder: a task report states
*size gate passed, no new lint suppression, coverage not lower*.

**What is new since the last handoff** (the two commits of 2026-10-21):

- **The tracking-ID form comes from the data folder's `couriers.toml`** (2.30). It used to be read
  from the list built into the program, so an edit to the owner's own file changed nothing; J&T
  Express now names `JY` **and** `JX` + 10 digits, the second measured on
  `samples/2026-10-21 JX.pdf`. **An existing data folder keeps its own `couriers.toml`**: for the same
  there, its J&T Express `tracking` line must become `J[XY]\\d{10}`.
- **The Shopee-style label packs** (2.31, from the owner's *add support* for
  `samples/2026-10-15 spx.pdf`): an order of that style is keyed by its own 14-character `No.Pesanan`,
  its slip's `SKU` column holds the seller SKU and `Variasi` the variation, and its `Batas Kirim:
  dd-mm-yyyy` is the order's deadline. Its answer is [D-LABEL-5](../../software-design/decisions/label.md),
  the **Default** of Q-LABEL-4; **if the app should rather name the style and refuse it, say so** and
  the reader goes back to that option.
- **Open, not built: [2.32](2.32-text-object-split-word.md)** — a word the page's text objects break
  at a font change (the Shopee-style label's `Variasi`, `Yamaha`) is read with a space in it. It shows
  in the stored full product name only; the packing list prints the short display name, unaffected.

**What was done.** Complexity 3 of 5 before, 2 after. Rust files over the 800-line ceiling: 4 to 0; UI files
over 500: 3 to 0. Types: `Grouping`, `OrderId`, `TrackingId`, `BatchNumber`, `PdfNumber`, `FileName`, one
layout/scope enum, a plan-row kind; no function takes more than five parameters or nests deeper than four
levels; every ID comment, numbered design pointer and banner is gone (`tools/check-comments.mjs`, zero);
`crates/testkit` holds the shared test fixtures and the PDFium lock. Tests: Rust 273 to 191, TypeScript 53
to 33. Coverage on production code only ([coverage.md](../coverage.md)): Rust lines 82.70 to 84.22,
TypeScript lines 54.72 to 97.04.

**Findings of the reviews** (the minimal one after R2 and the full one at the end): none needed a fix. Left as
they are, each with its reason:

- `crates/engine/tests/day_state.rs` keeps one cognitive-complexity warning (a merged scenario, 43 against 25):
  split it when the file is next touched. `crates/pdf/tests/common/mod.rs` keeps `#![allow(dead_code)]`
  (each test binary uses a subset of the shared module) and `app/vite.config.js` one `@ts-expect-error` (no
  `@types/node`).
- `AppState.day` is an empty string until a day is chosen (internal, always set before use); the stored
  `state.json` and `settings.json` keep the key `set` for a grouping (renaming it is a migration).
- Several test files were not merged (`couriers_*`, `rules_*`, `pdf/tests`, `commands/{batch,search}` tests): about
  100 small tests, covered; merge them when next touched.

**Small changes a user could notice** (from the commit messages): the command-line usage text no longer ends
with a pointer to a design file. Nothing else on screen, in a saved PDF or in a stored file changed.

**Regression test, in this order** (the screens whose code moved most):

1. Start the program on the existing data folder; the Day screen lists the saved batches; a saved row's
   **Printed** tick keeps its state after closing and reopening; **Open** opens the saved PDF.
2. **New batch**: add label files, change the *Grouping*, **Read labels**; an unknown logo asks *Which courier
   is this logo?* and the answer is remembered.
3. **Plan**: rows with their runs expand; **Skip**, **Use**, move up and down, rename a file, **Add a category
   for this batch**, the packing-list scope and layouts, **Save** with its progress bar, the saved-PDF check
   dialog, **Back**.
4. **Day**: **Amend** the newest batch (Plan titled *Redo batch N*, **Cancel amend**), **Revert**.
5. **Categories**: choose, **+ New grouping**, **Delete grouping**, edit a category's name, condition and
   boxes, the text view, the live counts.
6. **Search**: by tracking ID, item name and quantity; the courier cell reads as before.
7. **Settings**: the data folder, *About* reads *Version v0.1*.
8. Print one packing list; check the header, `×`, and the summary page.

**To decide (the owner):** the owner-visible word *pick* is still on screens and in printed text (*+ Add a pick
for this batch*, the *Pick* layout, the summary's *Pick* lines) while the code and the glossary say *category*;
the stored layout name `pick` would then be a migration. Default until the owner decides: keep the words.

## Next step, also — the owner's review of the feature design (written 2026-10-21, no code yet)

The owner asked for sessions in a side panel, repeated orders found and grouped, a page-for-page
check, and a full workflow to verify ([R-UI-4, R-UI-5, R-LABEL-1, R-APP-1, R-APP-2](../../reviews/README.md)).
Only documents were written; **no code was changed**.

- **Read first:** [the workflow](../../software-design/workflow/README.md) (every step, the claims
  I1–I7 and the *Check* questions), then [Sessions panel](../../software-design/ui/sessions.md) and
  [Repeated orders](../../software-design/ui/duplicates.md).
- **Answer** the ten questions in [sessions](../../software-design/questions/sessions.md) and
  [duplicates](../../software-design/questions/duplicates.md) (each has a default the design already
  follows), and write what is wrong in the workflow as items in [reviews/](../../reviews/README.md).
- **Then** a session writes the answers into `software-design/`, records them in `decisions/`, deletes
  the answered questions and writes the tasks. Where the code differs from the workflow:
  [workflow-audit](workflow-audit.md) (W-1 to W-8; W-5 and W-7 can lose files and are worth a test first).

## Phase 2 — PC app (Rust + Tauri)

A Windows program for the daily work: label downloads in, picks adjusted on screen, saved PDFs and packing
lists out, revert and amend of the last batch. The program is written in Rust, with the screens in Svelte +
TypeScript in a Tauri 2 window, as a portable program folder with no installer
([architecture](../../software-design/app/architecture.md)).

Reference for the reading and sorting rules: the Python package `tools/analysis/packing/` (frozen; the
first implementation of the same rules, tested). Port it module by module when a rule is missing
(`rules.py` → `rules.rs`, `slip.py` → `slip.rs`, …).

Every task of the phase (2.1–2.31) is finished; the only task file here is
[2.32](2.32-text-object-split-word.md). What the tasks built is in
[software-design/](../../software-design/README.md); how to run, build and test it is in
[../README.md](../README.md).

Open choices of the owner, each with a default the app follows until the owner decides:
[Q-LABEL-1 to Q-LABEL-3, Q-LABEL-5, Q-LABEL-7 and Q-LABEL-8](../../software-design/questions/label.md),
[Q-UI-1 to Q-UI-4](../../software-design/questions/ui.md),
[Q-APP-1 to Q-APP-3](../../software-design/questions/app.md),
[Q-APP-4 to Q-APP-7, Q-LABEL-9, Q-LABEL-10, Q-UI-5 to Q-UI-8](../../software-design/questions/README.md) (the feature round above) and
[Q-RULES-1](../../software-design/questions/rules.md) (the sets of categories; task 2.27 builds its default).

## Status

Phase 2 is finished (2.1–2.31) and the portable program folder `target/portable/Packing/` is built from
it. The owner's findings are built and wait for the test above:

- **R-UI-1** — the version *v0.1* in the window title, the title bar and *About*, from one place (2.25).
- **R-UI-2** — a saved **Printed** tick beside each saved-PDF row of the Day screen (2.26).
- **R-RULES-1** — several named sets of categories (2.27), built to the default of
  [Q-RULES-1](../../software-design/questions/rules.md).
- **R-UI-3** — the Printed tick as large as *Open* with no word (2.28); the owner-visible word for a set
  of categories is *Grouping* everywhere (2.29). Open: [R-RULES-2](../../reviews/rules.md) (a grouping can be
  renamed) waits for the owner's answer to [Q-RULES-2](../../software-design/questions/rules.md); build
  it after the answer is written into `software-design/`.

**Not checked on a running window** (only by tests): the Printed tick (click, close, reopen), the *Grouping*
control on New batch, the Categories grouping row (**+ New grouping**, **Delete grouping**), and the Plan header naming
the grouping. The seven in-app pictures of the user guide (`images/02-day.png` to `08-day-saved.png`,
`images/03-new-batch.png`, `05-plan.png`, `07-categories.png`) show the old title bar; `02-day.png` and
`08-day-saved.png` lack the **Printed** tick, `03-new-batch.png`, `05-plan.png` and `07-categories.png`
lack the grouping parts. They are retaken from the built program.

**Decided, open in detail:** the app remembers taught logos ([D-LABEL-4](../../software-design/decisions/label.md)); where they are written is [Q-LABEL-7](../../software-design/questions/label.md), default `couriers.toml`.

## Owner review at the end of the phase

What the owner checks, after 2.15–2.31 (the last items are 2.25–2.31):

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
- After 2.30: a J&T Express page printing `JX` + 10 digits is named *J&T Express* with its tracking ID
  read and no warning; editing the data folder's `couriers.toml` changes which forms are read.
- After 2.31: a Shopee-style download packs — its orders show their own 14-character number, the
  courier name and the deadline; its items show the seller SKU and the variation in the right fields
  (`samples/2026-10-15 spx.pdf`).

The owner writes findings in [`../../reviews/`](../../reviews/README.md), one file per scope.

## Reading for a new session

In this order.

1. [../../../AGENTS.md](../../../AGENTS.md) and [../README.md](../README.md): rules, commit and push, repository layout.
2. [../../reviews/README.md](../../reviews/README.md) and the items open there: the owner's findings on the
   built program, to turn into tasks or questions. The items open there are the latest dev review.
3. The task file here, [2.32](2.32-text-object-split-word.md) at present, and the checklist above.
4. [../../software-design/app/inputs.md](../../software-design/app/inputs.md) (inputs and `couriers.toml`),
   [batch.md](../../software-design/app/batch.md), [saved-pdf-check.md](../../software-design/app/saved-pdf-check.md),
   [data-folder.md](../../software-design/app/data-folder.md) (day folder and `state.json`),
   [architecture.md](../../software-design/app/architecture.md) and [program.md](../../software-design/app/program.md).
5. The screens: [../../software-design/ui/README.md](../../software-design/ui/README.md), then
   [day](../../software-design/ui/day.md), [new batch](../../software-design/ui/new-batch.md) (the logo prompt),
   [plan](../../software-design/ui/plan.md), [search](../../software-design/ui/search.md),
   [start](../../software-design/ui/start.md).
6. The label rules: [../../software-design/labels/README.md](../../software-design/labels/README.md) (per-order data,
   multi-page orders, packing slip, courier deduction, picks, packing list).
7. [../../software-design/questions/](../../software-design/questions/README.md): the open choices.
8. [../../software-design/requirements.md](../../software-design/requirements.md) for concepts and words.
9. [../../software-design/rules-file/conditions.md](../../software-design/rules-file/conditions.md): the condition
   language (fields, operators, the item-field "some line" rule).
10. [../../business-process/02-data-sources.md](../../business-process/02-data-sources.md): read before touching any CSV
    or PDF code; [01-packing-workflow.md](../../business-process/01-packing-workflow.md) for the daily steps.
11. [../setup.md](../setup.md), [../app.md](../app.md), [../cli.md](../cli.md), [../testing.md](../testing.md).
12. [../../user-guide/README.md](../../user-guide/README.md): update it in the commit that changes what the owner sees.
13. Code, after the documents: `crates/` (`engine`, `pdf`, `cli`), `app/` (Svelte in `src/`, Rust in `src-tauri/`),
    `testdata/` (made-up fixtures, `make_testdata.py`), `crates/engine/couriers.toml` (the bundled courier list);
    then `git status` and `git log --oneline -5`.

`samples/` and `work/` are confidential: never commit them or copy real data from them.
