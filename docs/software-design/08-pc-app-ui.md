# 08 — PC App: Screens

The screens of the PC app ([07-pc-app.md](07-pc-app.md)). One window, designed for a laptop
screen of 1366 × 768 or larger (smallest usable size 1024 × 640). Sketches show English texts
*(D8)* and made-up IDs and counts.

Open choices are marked *(D…)* and explained in
[02-design-questions.md](02-design-questions.md).

## Daily use in one picture

```
 Day screen ──[New batch]──► New batch ──[Read labels]──► (reading…) ──► Plan ──[Save batch]──┐
     ▲                         choose files                               adjust picks       │
     └────────────────────────────── back to the Day screen, new batch shown ◄────────────────┘
 From the Day screen: [Open] a saved PDF to print it · [Amend] / [Revert] the last batch ·
 [Categories] · [Settings]
```

A normal batch is four clicks: **New batch** → (the day's new downloads are already ticked)
**Read labels** → (the plan is the rules file, as every day) **Save batch** → **Open** each
saved PDF to print.

## Window frame

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│ Packing  ·  ◀  Wed 7 Oct 2026  ▶  [Today]           [Categories]  [Settings]           │
├────────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                        │
│                               (the current screen)                                     │
│                                                                                        │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

- **Day** in the title bar: the day folder everything is saved to. ◀ ▶ step a day; **Today**
  goes back. Changing the day while a draft is open asks first ("Discard the batch being
  prepared?").
- **Categories** and **Settings** open over the current screen; closing them returns to it
  (an open draft is kept and recounted).

## 1. Day

The start screen: what has been saved today, and the way to the next batch.

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│ Packing  ·  ◀  Wed 7 Oct 2026  ▶  [Today]           [Categories]  [Settings]           │
├────────────────────────────────────────────────────────────────────────────────────────┤
│  Today: 2 batches · 729 orders · 5 saved PDFs                       [ + New batch ]    │
│                                                                                        │
│  ┌ Batch 2 · saved 09:12 · 2 files · 129 orders ─────────── [Amend] [Revert] ───────┐  │
│  │  4  A  Sepatu              88 orders   3 runs     [Open]   packing list [Open]   │  │
│  │  5  Z  Lainnya             41 orders  12 runs     [Open]   packing list [Open]   │  │
│  │  Warnings: 1 ▸                                              [Open folder]        │  │
│  └──────────────────────────────────────────────────────────────────────────────────┘  │
│  ┌ Batch 1 · saved 07:40 · 3 files · 600 orders ────────────────────────────────────┐  │
│  │  1  A  Sepatu             172 orders   4 runs     [Open]   packing list [Open]   │  │
│  │  2  B  Spion & Knalpot     35 orders   4 runs     [Open]   packing list [Open]   │  │
│  │  3  Z  Lainnya            393 orders  38 runs     [Open]   packing list [Open]   │  │
│  │  Warnings: none                                             [Open folder]        │  │
│  └──────────────────────────────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

- Newest batch on top. Each saved PDF: number, pick code and name, orders, runs.
- **Open** opens the PDF in the PC's default PDF viewer, where the owner prints it *(D7)*.
  **Open folder** opens the batch folder in Explorer.
- **Amend** and **Revert** appear only on the newest batch ([07 › Revert and amend](07-pc-app.md#revert-and-amend-only-the-last-batch)):
  - Revert asks: "Undo batch 2? Its 2 saved PDFs and packing lists are deleted, and its 129
    orders can be saved again. If you already printed them, throw the printed labels away."
    [Cancel] [Undo batch 2]
  - Amend asks: "Redo batch 2 with the current categories? Batch 2 was saved at 09:12. If you
    already printed it, print it again after saving." [Cancel] [Redo batch 2] → Plan screen.
- **Warnings ▸** expands the batch's warnings (same texts as on the Plan screen).
- A day with no batches shows only "Nothing saved on this day yet." and **New batch**.
- After **Save batch**, the Day screen shows the new batch on top with a green line
  "Batch 3 saved: 3 PDFs, 3 packing lists." for a few seconds.

## 2. New batch

Choose the label files of this batch, then read them.

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│  New batch 3                                                            [Cancel]       │
│                                                                                        │
│  Today's label downloads (Downloads folder)                                            │
│   [✓] 10-07_11-02-10_Shipping label+Packing slip_1.pdf   200 pages   11:02             │
│   [✓] 10-07_11-02-21_Shipping label+Packing slip_2.pdf    57 pages   11:02             │
│   [ ] 10-07_09-05-40_Shipping label+Packing slip_1.pdf    88 pages   09:05  used in batch 2 │
│   [ ] 10-07_09-05-49_Shipping label+Packing slip_2.pdf    41 pages   09:05  used in batch 2 │
│                                                                                        │
│  ┌ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─┐       │
│     Drop label PDFs here, or  [Add files…]                                             │
│  └ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─┘       │
│                                                                                        │
│  Selected: 2 files · 257 pages                                      [ Read labels ]    │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

- **Today's label downloads** *(D9)*: PDFs in the downloads folder changed on the shown day
  whose name contains `Shipping label`. Files not used yet are ticked; files used in an earlier
  batch today show *used in batch n* and are not ticked (they can still be ticked: their
  orders are then found as already saved and left out).
- **Add files…** opens the Windows file dialog (several files at once). Dropping files onto the
  window does the same. Added files appear in the list, ticked.
- Files are read in the order of the list (by file name = download time and part number).
- Page counts are shown as soon as a file is listed (quick: only the page count is read).

### Reading

```
│  Reading labels …  page 143 of 257 (file 1 of 2)                                       │
│  ███████████████████████████████████████░░░░░░░░░░░░░░░░░░░░░░░░░         [Cancel]    │
```

Cancel stops reading and returns to the file list; nothing is written. When reading ends, the
Plan screen opens — or, if the batch cannot be used, a stop message (see *Messages*) with a
**Back** button.

## 3. Plan

The heart of the app: which orders go into which saved PDF. It opens with the rules file
applied in order (the same result as the script without `--interactive`). For a normal day
the owner checks the counts and presses **Save batch**.

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│  Batch 3 · 2 files · 257 pages · 256 orders · 0 already saved          [Cancel]        │
│  Warnings: 2 ▸                                                                         │
├────────────────────────────────────────────────────────────────────────────────────────┤
│  Use  #  Pick                         Orders  Runs   File name                         │
│  [✓]  6  A  Sepatu                       130     3   [6 A Sepatu ×130            ]  ↑ ↓ ▸│
│          name contains "sepatu"                                                        │
│  [✓]  7  B  Spion & Knalpot               20     4   [7 B Spion & Knalpot ×20    ]  ↑ ↓ ▸│
│          not name contains "sepatu" and (name contains "spion" or name …)             │
│  [ ]  –  C  Motor lainnya                  –     –   skipped for this batch      ↑ ↓   │
│  [✓]  8  Z  Lainnya (the rest)           106    31   [8 Z Lainnya ×106           ]      ▸│
│                                                                                        │
│  [+ Add a pick for this batch]                       [Edit categories]                 │
├────────────────────────────────────────────────────────────────────────────────────────┤
│  Packing list:  (•) one per PDF  ( ) one for the batch  ( ) none                       │
│                 [ ] full   [ ] summary   [✓] pick                                      │
│                                                                    [ Save batch ]      │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

- **Rows** are the picks in order; each takes its orders from what the rows above left. The
  numbers `#` are the saved-PDF numbers this batch will get (continuing the day).
- **Use** unticked = skip this pick for this batch only (its orders fall to the rows below).
  The rules file is not changed.
- **↑ ↓** move a pick for this batch only. The rest row is always last: the last
  `categories.toml` entry when it has no condition (here *Z Lainnya*), else `? Uncategorised`.
- **+ Add a pick for this batch** opens a small form: code (default `X`), name, condition
  (text, checked as you type, with the count it would take at that position). It is added
  above the rest row and can be moved. Not saved to the rules file; the form has a
  **Also add to categories** tick for that.
- A pick that takes 0 orders is shown grey with "0 orders — no PDF" and gets no number.
- **File name**: suggested `<n> <code> <name> ×<orders>` *(D5)*; the owner may type another
  name. The number stays in front, so the files sort in printing order.
- **▸** expands a pick to its runs — what the packing list will show:

  ```
  │  6-01  Sepatu Standar Samping Motor ×1                                     96 orders │
  │  6-02  Sepatu Standar Samping Motor ×2                                     26 orders │
  │  6-03  Sepatu Standar Samping Motor ×3                                      8 orders │
  ```
- **Warnings ▸** lists the read warnings in plain words (see *Messages*), e.g. couriers not
  recognised and orders already saved today with their batch and time.
- Every change (tick, move, add, edit categories) recounts all rows immediately.
- **Save batch** writes the batch ([07 › What happens in one batch](07-pc-app.md#what-happens-in-one-batch))
  and returns to the Day screen. **Cancel** asks "Discard batch 3? Nothing has been saved."
- Amend uses the same screen, titled "Redo batch 2", with the button **Save batch 2 again**.

## 4. Messages

Every message says what happened and what to do. Stop messages end the batch with nothing
written; warnings are listed and the batch can still be saved.

| Kind | When | Message |
|---|---|---|
| Stop | Pages without a packing slip | **These labels have no packing slip.** `<n>` of `<m>` pages are plain shipping labels (`<file>`, pages `<list>`). In the seller centre, download the labels again with **Shipping label + Packing slip**. |
| Stop | First page of a file with no Order ID | **Page `<p>` of `<file>` has no Order ID.** This does not look like a TikTok Shop label download. Check the file. |
| Stop | A file cannot be opened | **`<file>` cannot be read as a PDF.** It may be damaged or still downloading. Download it again. |
| Stop | Rules file error | **The categories have an error:** `<line, col: message>`. Fix it in Categories. [Open Categories] |
| Stop | Every order already saved today | **All `<n>` orders were already saved today** (batch `<b>`). These labels were prepared before; nothing to do. |
| Warning | Orders already saved today | **`<n>` orders were already saved today** and are left out: `<tracking ID>` (batch `<b>`, PDF `<k>`, `<time>`), … |
| Warning | Courier not recognised | **Courier unknown for `<n>` orders:** `<tracking IDs>`. They are sorted normally; only a condition on `courier` cannot see them. |
| Warning | `Qty Total` differs from the lines | **Slip total differs** for `<tracking ID>`: printed `<a>`, lines add up to `<b>`. Check that order's slip when packing. |
| Warning | Tracking ID not found on a page | **No tracking ID read** for Order `<order ID>`. Its label is saved as usual; only a condition on `tracking_id` cannot see it. |
| Error at save | A file is open in another program | **`<file>` is open in another program.** Close it (e.g. the PDF viewer) and press Save again. |
| Error at save | Disk full or folder not writable | **Cannot write to `<folder>`:** `<reason>`. Nothing was saved. |

## 5. Categories

Edit `categories.toml` (the picks, in order) without a text editor *(D12)*.

```
┌ Categories ─────────────────────────────────────────────────────────────── [Close] ┐
│  Order matters: an order is taken by the first category whose condition matches.   │
│                                                                                    │
│  Code  Name               Condition                                  In batch 3    │
│  [A ]  [Sepatu         ]  [name contains "sepatu"                 ]   130     ↑ ↓ 🗑 │
│  [B ]  [Spion & Knalpot]  [not name contains "sepatu"             ]    20     ↑ ↓ 🗑 │
│                           [and (name contains "spion" or name …   ]                │
│  [C ]  [Motor lainnya  ]  [product_category contains "sepeda motor"]   ✗      ↑ ↓ 🗑 │
│        ✗ line 1, col 1: field "product_category" needs the orders CSV, which the   │
│          PC app does not read                                                      │
│  [Z ]  [Lainnya        ]  (no condition: takes the rest)            106            │
│                                                                                    │
│  [+ Add category]                           [Edit as text]   [Cancel]  [Save]      │
│  Fields: name, display_name, variation, seller_sku, line_quantity, total_quantity, │
│  distinct_items, courier, tracking_id, ship_by   ·  Help: examples ▸               │
└────────────────────────────────────────────────────────────────────────────────────┘
```

- Each condition is checked as it is typed; an error shows under it with line and column and
  what was expected (texts of [04-rules-file.md](04-rules-file.md)). **Save** is possible only
  when every entry is valid.
- **In batch 3**: when a draft is open, the count each category takes there (in order, from
  what is left), so a change can be judged before saving. No draft → the column is hidden.
- **Edit as text** shows the whole file as text (comments included) for larger edits; the same
  checks apply.
- **Save** writes `categories.toml` in the data folder (comments kept), and the open draft's
  plan is rebuilt from it. Per-batch changes on the Plan screen (skips, added picks) are kept
  where they still apply.
- **Help: examples ▸** shows the examples of 04 (text, quantity, courier, dates).

## 6. Settings

| Setting | Control | Default |
|---|---|---|
| Data folder | Folder + [Change…] + [Open] | *(D4)* |
| Downloads folder | Folder + [Change…] | `%USERPROFILE%\Downloads` |
| Packing list default | Scope radio + layout ticks | one per PDF · pick |
| Language | Drop-down | *(D8)* |
| About | Version, PDFium version, [Open log file] | — |

Changing the data folder does not move existing files; the app asks "Use `<folder>` from now
on? Existing days stay in `<old folder>`." The categories file of the new folder is used (a
copy of the current one is put there if it has none).

## First start

1. "Choose where the app keeps its files" with the default of *(D4)* — [Use this folder]
   [Choose another…].
2. If the folder has no `categories.toml`, the app copies in the repository's
   `categories.toml` (built into the program) and opens Categories when it has errors (pick C
   today, *(D10)*).
3. The Day screen of today.
