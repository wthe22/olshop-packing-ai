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

## D18 — Should the courier also be read from the logo image?

**Words.** The courier is named from the tracking-ID form and the service tag (03 › *Courier
deduction*). A third clue exists: the courier's **logo**, which is an embedded image on the label.
Read it and the name would survive a label that prints neither a readable tag nor a known form.

**Measured (2026-10-11, Python + pdfium — the same C++ core the app uses; a 1,269-page sample).**

| What to read | Cost | What it finds |
|---|---|---|
| the image's pixel size, no decoding | ~0 ms (part of walking the page's objects) | the logo's size; J&T Express 975×377 · SiCepat 184×60 · IDX 1200×859 · TikTok Shop 326×59 · J&T Cargo 680×156/683×157 |
| the image's encoded bytes, no decoding | 0.005–0.02 ms per image | one fixed hash per courier for the four unmasked logos (stable across all 9 files) |
| the image's decoded pixels | ~1.1 ms per MB (184×60 → 0.05 ms; 975×377 → 1.5 ms; a 2000×2000 image → 13 ms) | the same, but also survives a re-encode |
| render the label's header and compare the pixels | ~17 ms per page | everything, including a masked logo — but it is the most code and the slowest |

**Two catches.** The **J&T Cargo logo is a masked image** (`/SMask`): its decoded pixels come out
black, so its hash is meaningless unless the mask is applied or the label is rendered. And a
slip-continuation page has no logo at all (2 of 1,269 pages).

| Option | What you see |
|---|---|
| **1. Form + tag only** (**Built for now**) | J&T Cargo keeps working from `57` + `FastTrack`; nothing new to build. A label whose tag is unreadable stays unknown |
| 2. The logo as a **third confirming clue** | The name is used only when form, tag and logo agree; a logo that disagrees turns a name into a warning. About 30–50 lines and a small logo table (size + hash) in the courier data file; under 1 s added to a 600-page day with the decoded-pixel read, ~0 with the encoded-bytes read |
| 3. The logo as the **main clue**, form + tag as the check | Names the courier when the tag is unreadable — but J&T Cargo needs the mask handled, and a re-rastered logo asset (which J&T Cargo already shows: 680×156 vs 683×157) would have to fall back to the form anyway |

Recommended: **2** when a third clue is wanted — it is cheap, exact where it applies, and it can
only turn a name into a warning, never a warning into a wrong name.

**What the answer changes:** one function (`crates/engine/src/label.rs`) and one field in the
courier data file; nothing else.

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
