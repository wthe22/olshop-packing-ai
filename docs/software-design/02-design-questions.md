# 02 — Design Questions

Questions that need the owner's choice. Each lists options; the one marked **Built for now**
is what the code does until the owner decides. Add new ones as `D1`, `D2`, …; remove one once
its answer is written into the other documents.

## D1 — The Android app and the pick flow

The PC script now works with picks ([03-pc-script.md](03-pc-script.md)); the app design is still
the old one: batches, pack groups, group numbers and scan filters on batch/group
([01-requirements.md](01-requirements.md) A-sections,
[05-app-architecture.md](05-app-architecture.md), [06-app-ui.md](06-app-ui.md)). Before phase 2
the app is revised to match the pick flow. Open until then.

## D3 — Packing-list default

The owner chooses the packing-list scope per invocation (`per-pdf`, `whole`, `none`), and the
layouts `full`, `summary`, `pick` keep their look ([03-pc-script.md](03-pc-script.md)). What is
open is the default:

1. `per-pdf`, layout `full` (a sheet with tracking IDs for every saved PDF). **Built for now.**
2. `none`; print on request.
3. Ask at the end of every run.

## D4 — A CSV that holds none of the label orders

E.g. yesterday's CSV with today's labels.

1. Warn once, carry on with the item data from the labels. **Built for now.**
2. Stop: a CSV that matches nothing is probably the wrong file.

## D5 — Plain labels (no packing slip) and no CSV

The script then knows no items.

1. Allowed: picks on label fields (`courier`, `tracking_id`, `ship_by`) work, pages stay in
   download order, a pick on an item field stops with an error. **Built for now.**
2. Not allowed: stop and ask for labels with packing slip or the CSV.

## D6 — Redo after fixing categories.toml

Running the same labels again finds every order already saved today (duplicate guard), so a
plain re-run saves nothing.

1. `--redo`: forget the day's last invocation in `state.json`, delete its saved PDFs and
   packing lists, then run as normal. **Built for now.**
2. No redo; the owner deletes the day folder by hand.

## D7 — Names in the day folder

1. Saved PDFs `3 Z Lainnya ×393.pdf`, packing lists `packing-list-3.pdf` (per-pdf) or
   `packing-list.pdf` (whole), all directly in `work/<day>/`. **Built for now.**
2. One subfolder per invocation (`work/<day>/run-1/ …`), numbering still through the day.

## D8 — Example rules file and labels-only runs

The repo's `categories.toml` has pick C `product_category contains "sepeda motor"`. Product
category is only in the CSV, so with labels alone the run stops
(`pick "C" uses field "product_category", which needs the orders CSV`).

1. Keep the error; the owner writes picks on label/slip fields (name, variation, courier, …)
   when running without the CSV. **Built for now.**
2. Rewrite pick C in the example file on `name` (e.g. `name contains "motor"`).
3. Skip a pick whose field has no source (warning), and carry on with the next one.
4. Remember product name → category from every CSV seen, and use it when the CSV is missing.
