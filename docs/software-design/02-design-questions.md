# 02 — Design Questions

Questions that need the owner's choice. Each lists options; the one marked **Built for now**
is what the code does until the owner decides. Add new ones as `D1`, `D2`, …; remove one once
its answer is written into the other documents.

Owner decisions that change the design are listed under **Decided**, below, until a later pass
writes them into the other documents.

## Decided (not yet designed)

- **The Android app does no PDF work.** Reading, sorting and grouping the label PDFs, and
  printing the packing list, stay on the PC. The app never opens the label PDFs.
- **The Android workflow does not follow the PC workflow.** The app is its own checking
  workflow, not a mirror of the picks. The app design in
  [01-requirements.md](01-requirements.md) (A-sections),
  [05-app-architecture.md](05-app-architecture.md) and [06-app-ui.md](06-app-ui.md) is the old
  one and is revised later ([D1](#d1--the-android-app-and-the-pick-flow)).
- **The PC also gets a UI: Tauri 2 + Svelte + TypeScript.** The command-line script
  ([03-pc-script.md](03-pc-script.md)) is the engine under it.
- **Scan and the filter (Android).** When the item scanned is not the filtered one, warn that it
  does not match **and** offer to change the filter so it matches the package scanned: filter on
  the product only, or on product **and** quantity. (Changes
  [01-requirements.md](01-requirements.md) A2 and the filter-rejected message in
  [06-app-ui.md](06-app-ui.md#scan-messages).)
- **A batch's label download comes in clusters of up to 200 pages** — several files per batch
  ([business-process/01](../business-process/01-packing-workflow.md)).
- **The orders CSV is dropped from the PC script for now.** The script works from the label PDF
  and its packing slip only; item data comes from the slip. (Makes D4 and D8 void;
  [03-pc-script.md](03-pc-script.md) and the CSV-only fields in
  [04-rules-file.md](04-rules-file.md) are updated in the later design pass.) Note the slip has
  **no `SKU ID`** ([business-process 02](../business-process/02-data-sources.md#label--packing-slip-export-option)),
  so a slip-only run identifies items by product name + variation, not by SKU ID.
- **`product_category` is never used.** It is the platform's category, not the shop's
  (resolves [D8](#d8--example-rules-file-and-labels-only-runs) even if the CSV returns).

## D1 — The Android app and the pick flow

The PC script now works with picks ([03-pc-script.md](03-pc-script.md)); the app design is still
the old one: batches, pack groups, group numbers and scan filters on batch/group
([01-requirements.md](01-requirements.md) A-sections,
[05-app-architecture.md](05-app-architecture.md), [06-app-ui.md](06-app-ui.md)). Before phase 3
the app is revised as its own checking workflow — not the pick flow, and with no PDF work
(see **Decided**). Open until then.

## D3 — Packing-list default

The owner chooses the packing-list scope per invocation (`per-pdf`, `whole`, `none`), and the
layouts `full`, `summary`, `pick` keep their look ([03-pc-script.md](03-pc-script.md)). What is
open is the default:

1. `per-pdf`, layout `full` (a sheet with tracking IDs for every saved PDF).
2. `none`; print on request.
3. Ask at the end of every run.

**Decided: scope `per-pdf`, layout `pick`.**

## D4 — A CSV that holds none of the label orders

**Void: there is no CSV any more (see Decided).**

1. Warn once, carry on with the item data from the labels.
2. Stop: a CSV that matches nothing is probably the wrong file.

## D5 — Plain labels (no packing slip)

The script then knows no items.

1. Allowed: picks on label fields (`courier`, `tracking_id`, `ship_by`) work, pages stay in
   download order, a pick on an item field stops with an error.
2. Not allowed: stop and ask for labels with a packing slip.

**Decided: option 2 — not allowed.** Without a packing slip there is no item data (the CSV is
gone too), so the run stops and asks for the "Shipping label + Packing slip" export.

## D6 — Redo after fixing categories.toml

Running the same labels again finds every order already saved today (duplicate guard), so a
plain re-run saves nothing.

1. `--redo`: forget the day's last invocation in `state.json`, delete its saved PDFs and
   packing lists, then run as normal. **Built for now.**
2. No redo; the owner deletes the day folder by hand.

**Decided: like git — offer revert and amend of the last run.** The owner can revert the last
run (undo it) or amend it (redo it in place) instead of deleting the day folder by hand.

## D7 — Names in the day folder

**Decided: `labels/<day>/<batch>/<my group names>.pdf`.**
A top-level `labels` folder, one folder per day, one subfolder per batch, and inside it each
group's file is named by the owner (`<my group names>.pdf`; a suggested name may be offered).
The numbering still runs through the day.

The current behaviour, for reference:

1. Saved PDFs `3 Z Lainnya ×393.pdf`, packing lists `packing-list-3.pdf` (per-pdf) or
   `packing-list.pdf` (whole), all directly in `work/<day>/`. **Built for now.**
2. One subfolder per invocation (`work/<day>/run-1/ …`), numbering still through the day.

## D8 — Example rules file and labels-only runs

**Decided: `product_category` is never used.** It is the platform's category, not the shop's, so
the example file must not rely on it (pick C is changed or dropped). The CSV is also gone
(see Decided), so the labels-only case is the only case.

1. Keep the error; the owner writes picks on label/slip fields (name, variation, courier, …).
2. Rewrite pick C in the example file on `name` (e.g. `name contains "motor"`).
3. Skip a pick whose field has no source (warning), and carry on with the next one.
4. Remember product name → category from every CSV seen, and use it when the CSV is missing.
