# 2.24 Amend opens the PDF selection page first

**Amend** on the newest batch no longer reads straight into the Plan screen. It opens a selection page
titled *Redo batch 2* with the batch's label PDFs (its `download/` copy) already listed, then **Read
labels** opens the Plan screen ([Amend last batch](../../software-design/app/batch.md),
[New batch](../../software-design/ui/new-batch.md#the-amends-selection-page)). **Add files…** adds PDFs
([Q-APP-2](../../software-design/questions/app.md), default: the file picker, any PDF on the PC, as in a
new batch; a drop works too), **×** takes a PDF off the list and drops its orders from the draft
([Q-APP-3](../../software-design/questions/app.md), default: they stay unsaved and the next batch reads
them again). **Read labels** reads the selected files with the duplicate guard as an amend does today
(the batch's own orders are not "already saved"). **Back** from the amend's Plan screen returns to this
page with the files still listed; **Cancel amend** (bottom) and the page's **Cancel** return to the Day
screen untouched and never ask — an amend writes nothing before **Save batch 2 again**, and Save replaces
the old batch as it does now.

- **Code:** `app/src-tauri/src/commands/batch.rs`: `start_amend` / `start_amend_impl` open the selection
  draft with the batch's files pre-selected out of its `download/` copy in the `state.json` order and
  write nothing (the read moves to **Read labels**); `read_labels` reads a selection (added files join,
  removed files are gone); `discard_draft` serves the amendment's **Cancel** at both places;
  `save_batch` is unchanged (it replaces the batch). `app/src/lib/NewBatchScreen.svelte`: the amend title,
  the pre-selected files, **Add files…**, **×**, **Read labels**, **Cancel**.
  `app/src/lib/PlanScreen.svelte` and the shell `app/src/routes/+page.svelte`: **Back** on an amend shows
  the selection page again with the files still listed. `app/src/texts.ts`: the amend selection page's
  texts, and the *for now* comment and the `planButtons` note about the amend's **Back** target go.
- **Tests:** Rust (`app/src-tauri/src/commands/batch.rs`, `crates/engine/`) — `start_amend` lists the
  batch's files pre-selected and leaves `state.json` byte-identical; reading a selection with one file
  removed gives a plan without that file's orders; adding a file adds its orders to the plan; **Save**
  replaces the batch (numbers reused, the old folder gone); **Cancel** at the selection page and
  **Cancel amend** both leave `state.json` byte-identical. Front-end (`app/tests/`, e.g.
  `planButtons.test.mjs`) — pure helper tests for the button sets: the amend's Plan **Back** targets the
  selection page and **Cancel amend** the Day screen, a new batch's Plan screen has neither.
- **Result:** (not started)
