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

## D15 — The SiCepat labels (a courier the parser cannot name)

**Words.** An order's courier comes from the label (03 › *Courier deduction*): a text clue (J&T
labels print `www.jet.co.id`) or the tracking-ID form — but the form is only used when it belongs
to one courier. SiCepat labels print the name **only as the logo image** and their tracking ID is
12 digits starting `00`; `couriers.toml` marks that form as shared with J&T Cargo, so those orders
save with an empty courier and the day warns.

**Example.** 2026-10-11, 98 orders: J&T Express 74, SiCepat 23, IDX 1. The 23 SiCepat orders
(15 in batch 1, 8 in batch 2) are warned *Courier unknown*; a category whose condition uses
`courier` cannot see them. The 2026-10-07 sample is the same shape: 82 of 601 pages.

| Option | What you see |
|---|---|
| **1. Keep refusing the `00…` form** (**Built for now**) | The SiCepat orders save with an empty courier and are listed in the warning; a `courier` condition cannot separate them |
| 2. `00` + 10 digits means **SiCepat** | The 23 orders get `SiCepat` and a condition can use it. Risk: a courier sharing that 12-digit form (J&T Cargo) would be named SiCepat silently — no such label has appeared in the 699 pages read (2026-10-07 and 2026-10-11) |

Recommended: **2** — it needs one line of data and fixes today's 23 orders and the sample's 82
pages; if a J&T Cargo label ever appears, its form is added the same way.

**What the answer changes:** one line of `couriers.toml` (`tracking_unique`); the deduction
itself (`crates/engine/src/label.rs`) keeps working as it is.

## D16 — Where the courier data lives

**Words.** The courier list — name, text clues, tracking forms — is built into the program
(`pc/crates/engine/couriers.toml`, embedded in the binary). A new courier, or a change like D15,
needs a new program version. `categories.toml` shows the other way: the app writes it into the
data folder on first start and you edit it in the app.

| Option | What you see |
|---|---|
| **1. Built into the program** (**Built for now**) | Courier changes arrive with a new version; the file is not in the data folder (nothing to edit, break or lose) |
| 2. `couriers.toml` in the data folder | You edit the couriers yourself (e.g. one whose labels print no name); the app writes the built-in file on first start, like `categories.toml`, and refuses a broken file with the line |

**What the answer changes:** a data-folder path and loader in `label.rs`, the *Start*/Settings
handling of the data folder, and one line in 07 › *Inputs*.

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
