# 02 — Design Questions

Questions that need the owner's choice. Each explains the problem, gives options with an
example of what you would see, and marks one **Built for now**: what the code does until the
owner decides (so development is not blocked). Add new ones as the next free `D` number; remove
one once its answer is written into the other documents.

Owner decisions that change the design but are not designed yet are listed under **Decided**,
below, until a later pass writes them into the other documents.

## Decided (not yet designed)

These concern the Android app; they are designed in the app revision ([D1](#d1--the-android-app-and-the-pick-flow)).

- **The Android app does no PDF work.** Reading, sorting and grouping the label PDFs, and
  printing the packing list, stay on the PC. The app never opens the label PDFs.
- **The Android workflow does not follow the PC workflow.** The app is its own checking
  workflow, not a mirror of the picks. The app design in
  [01-requirements.md](01-requirements.md) (A-sections),
  [05-app-architecture.md](05-app-architecture.md) and [06-app-ui.md](06-app-ui.md) is the old
  one and is revised later.
- **Scan and the filter (Android).** When the item scanned is not the filtered one, warn that it
  does not match **and** offer to change the filter so it matches the package scanned: filter on
  the product only, or on product **and** quantity. (Changes
  [01-requirements.md](01-requirements.md) A2 and the filter-rejected message in
  [06-app-ui.md](06-app-ui.md#scan-messages).)

## D1 — The Android app and the pick flow

The PC side now works with picks ([07-pc-app.md](07-pc-app.md)); the app design is still the
old one: batches from the orders CSV, pack groups, group numbers and scan filters on
batch/group ([01-requirements.md](01-requirements.md) A-sections,
[05-app-architecture.md](05-app-architecture.md), [06-app-ui.md](06-app-ui.md)). Before phase 3
the app is revised as its own checking workflow — not the pick flow, and with no PDF work (see
**Decided**). Open until then.

## D13 — Unticking the rest row

**Words.** The *rest row* is the last row of the Plan screen ([08 › 3. Plan](08-pc-app-ui.md#3-plan)):
the last `categories.toml` entry when it has no condition (e.g. *Z Lainnya*), else
`? Uncategorised`. Unticking **Use** on a row skips that pick for this batch: its orders fall to
the rows below. The rest row has no row below it.

**Example.** Batch 3 has 256 orders: 130 Sepatu, 20 Spion & Knalpot, 106 for *Z Lainnya*. You
untick *Z Lainnya*.

| Option | What you see |
|---|---|
| **1. The 106 orders go to `? Uncategorised`** (**Built for now**) | A new last row `? Uncategorised` with 106 orders; the batch still saves all 256 orders |
| 2. The rest row has no tick | *Z Lainnya* is always used; to leave orders out, change the conditions |
| 3. The 106 orders are left out of this batch | They are not saved; they count as "not saved today" and come back in the next batch with the same labels |

**What the answer changes:** the plan count in `plan.rs` and the Plan screen (task 2.10).

## D14 — A log file

**Words.** [08 › 6. Settings](08-pc-app-ui.md#6-settings) has an *About* row with
**Open log file**. A log file would be a text file in the data folder where the app writes
what it did and any unexpected error (e.g. `log.txt`: "07:40:12 batch 1 saved, 3 files, 600
orders"). Nothing else in the design writes one; every batch already has its `summary.txt`.

| Option | What you see |
|---|---|
| **1. No log file** (**Built for now**) | *About* shows the version and the PDFium version only; errors are shown on screen |
| 2. `log.txt` in the data folder | One line per batch saved, reverted or amended, plus unexpected errors; **Open log file** opens it |

**What the answer changes:** the *About* row of Settings and one small writer in the app.

## D18 — The J&T Cargo logo (the one the logo clue cannot read)

**Words.** The owner's rule (2026-10-11) names the courier from the **logo** (its image hash) and
the **tracking-ID form**, both agreeing, and asks the owner once about a logo never seen before.
Four couriers' logos are plain images whose encoded bytes hash to one stable value each across all
9 sample files (J&T Express 975×377, SiCepat 184×60, IDX 1200×859, TikTok Shop 326×59).
**J&T Cargo's is not one of them**: it is a *masked* image (`/SMask`) whose base is a flat colour
(331 bytes encoded) and whose raster changes between the two sample labels (680×156 on one,
683×157 on the other — base and mask hashes both differ), so no hash of it can be stable. Its
label does print `FastTrack` as real text.

| Option | What you see |
|---|---|
| **1. `FastTrack` is J&T Cargo's second clue** (**Built for now**) | The samples' 2 Cargo orders are named; a Cargo label whose text is unreadable stays unknown. The tag stays unused for every other courier |
| 2. Compare the logo by appearance, not by hash | Render the label's header (~17 ms a page) and compare it with a stored picture, tolerating the re-raster. Names Cargo too, for the most code in the deduction |
| 3. J&T Cargo stays unknown until its logo is readable | Nothing to build; every Cargo order saves with an empty courier and a warning |

Recommended: **1** — this is the case the owner asked for ("unless I found a case where it is
helpful"): `FastTrack` is the courier's printed brand line, not a service code, and it decided
both Cargo labels in the samples.

**What the answer changes:** one line of the courier data (J&T Cargo's text clue), or the
deduction's comparison.

## D20 — How the app writes a taught logo into `couriers.toml`

**Words.** When the owner names an unseen logo in the prompt ([08 › the logo prompt](08-pc-app-ui.md#the-logo-prompt)),
the answer is written into `couriers.toml` — a file the owner also edits by hand (D19, answered
2026-10-11: one file, no second one). The only question left is *how* the app writes it, so that a
hand edit is never lost.

| Option | What you see |
|---|---|
| **1. A `[logo]` table appended at the end of the file** (**Built for now**) | The app adds one line (`"hash" = "SiCepat"`) at the end and never touches the entries above it, so your own comments and order survive. The same courier's logos then live in two places — its `logo` list and this table |
| 2. The hash is added to that courier's own `logo` list | One place per courier, but the app has to edit the line you wrote, and a hand-typed comment on it can be lost |
| 3. Nothing is written | Simplest: the answer names that batch's orders only, and the same logo is asked about again next batch |

Recommended: **1** — the app only ever appends, so nothing you typed can be lost.

**What the answer changes:** what the app does after the owner names a logo (append a line, edit a
list, or nothing).

## D17 — The check of the saved PDFs: every batch or only a large one

**Words.** Task 2.18 makes the app read its own output back before a batch is finished: every order
exactly once, in one saved PDF, page for page, plus the packing list's counts. Errors (a page
missing, a page twice, an order in two PDFs or in none, a page holding another order's label) and
warnings (the pages are all there, only out of order) are listed separately, *Save anyway* is
allowed, and what was found stays on the day's batch card. The check costs about 1–2 s on the
owner's biggest day (600 pages, 5 saved PDFs).

**Example.** A normal morning batch is 2 files, about 100 pages, 100 orders — well under a second.

| Option | What you see |
|---|---|
| **1. Every batch** (**Built for now**) | Every *Save batch* runs the check; on the 600-page day the save takes 1–2 s longer, on a normal batch nothing noticeable |
| 2. Only a batch above a size | Batches under, say, 200 pages save without the check and the day says *not checked*: small batches stay quick, but a bug that only hits small batches is never seen |

**What the answer changes:** one condition around the check in `pdf/src/batch.rs` and one line on
the Day card. Both options keep the whole check for the big batch.
