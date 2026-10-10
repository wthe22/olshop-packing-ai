# 08 — PC App: Screens

The screens of the PC app ([07-pc-app.md](07-pc-app.md)). One window, designed for a laptop
screen of 1366 × 768 or larger (smallest usable size 1024 × 640). Sketches show the English texts and
made-up IDs and counts.

## Daily use in one picture

```
 Day screen ──[New batch]──► New batch ──[Read labels]──► (reading…) ──► Plan ──[Save batch]──┐
     ▲                         choose files                               adjust picks       │
     └────────────────────────────── back to the Day screen, new batch shown ◄────────────────┘
 From the Day screen: [Open] a saved PDF to print it · [Amend] / [Revert] the last batch ·
 [Categories] · [Settings]
```

A normal batch: **New batch** → **Add files…** (choose the new downloads) → **Read labels** →
(the plan is the rules file, as every day) **Save batch** → **Open** each saved PDF and the
packing list to print.

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
│  │  4  A  Sepatu              88 orders   3 runs     [Open]                         │  │
│  │  5  Z  Lainnya             41 orders  12 runs     [Open]                         │  │
│  │  Packing list [Open]   Warnings: 1 ▸                        [Open folder]        │  │
│  └──────────────────────────────────────────────────────────────────────────────────┘  │
│  ┌ Batch 1 · saved 07:40 · 3 files · 600 orders ────────────────────────────────────┐  │
│  │  1  A  Sepatu             172 orders   4 runs     [Open]                         │  │
│  │  2  B  Spion & Knalpot     35 orders   4 runs     [Open]                         │  │
│  │  3  Z  Lainnya            393 orders  38 runs     [Open]                         │  │
│  │  Packing list [Open]   Warnings: none                       [Open folder]        │  │
│  └──────────────────────────────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

- Newest batch on top. Each saved PDF: number, pick code and name, orders, runs.
- **Open** opens the saved PDF, or the batch's packing list, in the PC's default PDF viewer,
  where the owner prints it.
  **Open folder** opens the batch folder in Explorer.
- **Amend** and **Revert** appear only on the newest batch ([07 › Revert and amend](07-pc-app.md#revert-and-amend-only-the-last-batch)):
  - Revert asks: "Undo batch 2? Its 2 saved PDFs, packing list and download copies are deleted, and its 129
    orders can be saved again. If you already printed them, throw the printed labels away."
    [Cancel] [Undo batch 2]
  - Amend asks: "Redo batch 2 with the current categories? Batch 2 was saved at 09:12. If you
    already printed it, print it again after saving." [Cancel] [Redo batch 2] → Plan screen.
- **Warnings ▸** expands the batch's warnings (same texts as on the Plan screen).
- A day with no batches shows only "Nothing saved on this day yet." and **New batch**.
- After **Save batch**, the Day screen shows the new batch on top with a green line
  "Batch 3 saved: 3 PDFs, 1 packing list." for a few seconds.

## 2. New batch

Choose the label files of this batch, then read them.

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│  New batch 3                                                            [Cancel]       │
│                                                                                        │
│  ┌ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┐       │
│     Drop label PDFs here, or  [Add files…]                                             │
│  └ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┘       │
│                                                                                        │
│  10-07_11-02-10_Shipping label+Packing slip_1.pdf 200 pages 11:02                  [×] │
│  10-07_11-02-21_Shipping label+Packing slip_2.pdf  57 pages 11:02                  [×] │
│  10-07_09-05-49_Shipping label+Packing slip_2.pdf  41 pages 09:05  used in batch 2 [×] │
│                                                                                        │
│  Selected: 3 files · 298 pages                                         [ Read labels ] │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

- **Add files…** opens the Windows file dialog (several files at once; it opens in the folder
  used last time). Dropping files onto the window does the same.
- Each listed file shows its page count (quick: only the page count is read). A file read in an
  earlier batch today shows *used in batch n* in orange: it can still be read, its orders are
  then found as already saved and left out. **×** takes a file off the list; a file added twice
  is listed once.
- Files are read in the order of the list, sorted by file name (= download time and part
  number). **Read labels** needs at least one file.

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
│  [✓]  8  Z  Lainnya (the rest)           106    31   [8 Z Lainnya ×106           ]      ▸│
│                                                                                        │
│  [+ Add a pick for this batch]                       [Edit categories]                 │
├────────────────────────────────────────────────────────────────────────────────────────┤
│  Packing list:  ( ) one per PDF  (•) one for the batch  ( ) none                       │
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
- **+ Add a pick for this batch** opens a small form: code (default `X`), name, condition (the
  boxes editor of [Categories](#5-categories), with the count it would take at that position).
  It is added above the rest row and can be moved. Not saved to the rules file; the form has a
  **Also add to categories** tick for that.
- A pick that takes 0 orders is shown grey with "0 orders — no PDF" and gets no number.
- **File name**: suggested `<n> <code> <name> ×<orders>`; the owner may change the whole name,
  number included (keeping the number in front keeps the files in printing order). The packing
  list shows the number `#` whatever the name. Two saved PDFs of the batch cannot get the same
  name (upper and lower case count as the same, as in Windows): the second shows "Same name as PDF 6" and **Save batch** waits until one is changed.
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

Edit `categories.toml` (the picks, in order) without typing the condition language: each
condition is built from boxes, as in the Android design
([06 › Condition editor](06-app-ui.md#condition-editor-scan-filters-and-categories)). A text
view is there for typing a condition instead.

```
┌ Categories ───────────────────────────────────────────────────────────────── [Close] ┐
│  Order matters: an order is taken by the first category whose condition matches.     │
│                                                                                      │
│  [A ] [Sepatu           ]                               In batch 3: 130   ↑ ↓ 🗑      │
│   ┌ All of these ▾ ────────────────────────────────────────────────────────────┐     │
│   │ [Product name ▾] [contains ▾] [sepatu          ]  [ ] not  [×]             │     │
│   │ [+ condition] [+ group]                                                    │     │
│   └────────────────────────────────────────────────────────────────────────────┘     │
│   [Text view]                                                                        │
│                                                                                      │
│  [B ] [Spion & Knalpot  ]                               In batch 3:  20   ↑ ↓ 🗑      │
│   ┌ All of these ▾ ────────────────────────────────────────────────────────────┐     │
│   │ [Product name ▾] [contains ▾] [sepatu          ]  [✓] not  [×]             │     │
│   │ ┌ Any of these ▾ ─────────────────────────────────────────────── [ ] not ┐ │     │
│   │ │ [Product name ▾] [contains ▾] [spion           ]  [ ] not  [×]         │ │     │
│   │ │ [Product name ▾] [contains ▾] [knalpot         ]  [ ] not  [×]         │ │     │
│   │ │ [+ condition] [+ group]                                   [× group]    │ │     │
│   │ └────────────────────────────────────────────────────────────────────────┘ │     │
│   │ [+ condition] [+ group]                                                    │     │
│   └────────────────────────────────────────────────────────────────────────────┘     │
│   [Text view]                                                                        │
│                                                                                      │
│  [Z ] [Lainnya          ]  no condition: takes the rest In batch 3: 106              │
│                                                                                      │
│  [+ Add category]                         [Edit file as text]   [Cancel]  [Save]     │
└──────────────────────────────────────────────────────────────────────────────────────┘
```

One category switched to **Text view**, with an error:

```
│  [B ] [Spion & Knalpot  ]                               In batch 3:   ✗   ↑ ↓ 🗑      │
│   ┌────────────────────────────────────────────────────────────────────────────┐     │
│   │ not name contains "sepatu"                                                 │     │
│   │ and (name contains "spion" or name contains "knalpot"                      │     │
│   └────────────────────────────────────────────────────────────────────────────┘     │
│   ✗ line 2, col 54: expected ")"                                                     │
│   [Boxes] (after the error is fixed)   Help: examples ▸                              │
```

- **A category** is a code box, a name box, its condition, the count it takes in the open batch,
  **↑ ↓** to move it and **🗑** to delete it (asks first). The last category has no condition
  (it takes the rest) and stays last. **+ Add category** adds one above it.
- **A group** (box) is either **All of these** (*and*) or **Any of these** (*or*), switched with
  its **▾**. It holds rows and other groups. **+ condition** adds a row, **+ group** a group
  inside; **×** removes a row, **× group** a group with everything in it. An empty group is an
  error ("This group is empty"). Every condition starts as one *All of these* group.
- **A row** is one comparison, field · operator · value:
  - **Field**: the fields of [07 › Fields the conditions can use](07-pc-app.md#fields-the-conditions-can-use),
    with plain names and the field name in small text: Product name (`name`), Display name
    (`display_name`), Variation, Seller SKU, Line quantity, Total quantity, Different items
    (`distinct_items`), Courier, Tracking ID, Ship by.
  - **Operator**: depends on the field. Text: contains, equals, starts with. Number and
    date/time: =, ≠, <, ≤, >, ≥.
  - **Value**: typed, without quotes. Number fields take a whole number; Ship by takes
    `2026-10-07`, `14:00` or `2026-10-07 14:00` ([04 › Date and time values](04-rules-file.md#date-and-time-values)).
    A wrong value is shown red with the 04 message.
- **not** (per row and per group) reverses it: the first row of B above reads "name does not contain
  sepatu".
- **Text view** switches one category between the boxes and its condition as text (the 04
  language), checked while typing, errors with line and column; **Help: examples ▸** shows the
  examples of 04. **Boxes** switches back once the text has no error. Every valid text can be
  shown as boxes: nested groups of the same kind are merged (`a and (b and c)` is one *All of
  these* group with three rows), a single comparison is a group with one row.
- **Boxes and text agree**: the boxes are made by the engine's own parser and turned back into
  text by its printer ([07 › Commands](07-pc-app.md#commands-between-window-and-rust)). A
  category built or changed in boxes is written in the printer's standard one-line form
  (`not name contains "sepatu" and (name contains "spion" or name contains "knalpot")`); a
  category not changed keeps its text exactly as in the file, line breaks included.
- **In batch 3**: when a draft is open, the count each category takes there (in order, from
  what is left), so a change can be judged before saving. No draft → the counts are hidden.
- **Edit file as text** shows the whole file as text (comments included) for larger edits; the
  same checks apply.
- **Save** is possible only when every category is valid. It writes `categories.toml` in the
  data folder (comments kept), and the open draft's plan is rebuilt from it. Per-batch changes
  on the Plan screen (skips, added picks) are kept where they still apply.

## 6. Settings

| Setting | Control | Default |
|---|---|---|
| Data folder | Folder + [Change…] + [Open] | The program's folder ([Start](#start)) |
| Packing list default | Scope radio + layout ticks | one for the batch · pick |
| About | Version, PDFium version, [Open log file] | — |

**Change…** switches the data folder until the app closes; it does not move existing files.
The app asks "Use `<folder>` until the app closes? Existing days stay in `<old folder>`." The
categories file of the new folder is used (a copy of the current one is put there if it has
none). The next start looks in the program folder again.

## Start

At every start the app looks for `categories.toml` in its own folder
([07 › Program folder](07-pc-app.md#program-folder-portable)):

1. **Found** → that folder is the data folder; the Day screen of today opens.
2. **Not found** (first start, or the program was copied without its data) → "No packing data
   next to the program (`D:\Packing\`)." [Use this folder] [Choose a folder…]
   - **Use this folder** writes the built-in `categories.toml` (the repository's) there; every
     later start finds it. If the folder cannot be written (e.g. under `Program Files`), the
     message says so and only **Choose a folder…** is left.
   - **Choose a folder…**: a folder with a `categories.toml` is used as it is; an empty one gets
     the built-in file. It is used until the app closes, so the next start asks again.
3. If `categories.toml` has errors, Categories opens before the Day screen.
