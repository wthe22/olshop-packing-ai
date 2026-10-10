# 07 — PC App: Behaviour and Architecture

The **PC app** is a Windows desktop program with windows and buttons. It replaces the Python
command-line script for daily use: the owner adds the day's label downloads, sees the picks
with their counts, adjusts them if needed, and saves the label PDFs and packing lists — without
a terminal. The screens are in [08-pc-app-ui.md](08-pc-app-ui.md).

The whole program is written in **Rust** (engine and window), with the screens in
**Svelte + TypeScript** inside a **Tauri 2** window. The Python script of phase 1
([03-pc-script.md](03-pc-script.md)) is not used by the app. It stays in the repository for
good, **frozen** (no new features):

- as the **reference implementation**: the Rust engine must give the same picks, runs and pages
  on the same input;
- as a **fallback** when the app cannot start (run from Git Bash as in 03). The script and the
  app do not share day state (`work/<day>/` vs the data folder): orders saved by the script are
  unknown to the app's duplicate guard, and the other way round.

Rule features added later go into the app only. The fixtures the script is tested against
(`testdata/`) are not changed for them; app-only cases go into separate fixture files.

The PDF library is **PDFium** for all three PDF jobs: read the text with its positions, copy
pages unchanged, write the packing list. Its packing lists are larger than the script's —
about 1.5 MB per batch instead of about 60 KB, because PDFium writes the whole of each font
into the file — and that is accepted; they print the same. Only the packing-list writer could
later be replaced by one that embeds only the letters used, if the size ever becomes a
problem; nothing else in the program would change.

## What carries over from the script, what changes

The reading and sorting rules of [03-pc-script.md](03-pc-script.md) carry over unchanged:

| Rule | Where it is described |
|---|---|
| Order ID per page (two-step search), multi-page orders | 03 › *Order ID from the label*, *Multi-page orders* |
| Reading the packing slip by text position (columns, rows, wrapped cells, `Qty Total` check) | 03 › *Reading the packing slip* |
| Courier deduction from the label text and the tracking-ID form (`couriers.toml`) | 03 › *Courier deduction* |
| Picks taken in order from what is left; the rest | 03 › *Picks* |
| Runs inside a saved PDF (identical contents together, sort order) | 03 › *A saved PDF* |
| Saved-PDF numbers through the day, run numbers `3-05` | 03 › *Numbering and file names* |
| Duplicate guard (an order saved earlier the same day is left out, warned) | 03 › *Picks* |
| The three packing-list layouts `full`, `summary`, `pick` and their look | 03 › *Packing list (A4)* |
| `categories.toml` and the condition language | [04-rules-file.md](04-rules-file.md) |

What is different in the app (these are owner decisions):

| Topic | Script (phase 1) | App |
|---|---|---|
| How it is used | Command typed in Git Bash | Windows program, buttons ([08](08-pc-app-ui.md)) |
| Orders CSV | Optional input | **Not read at all.** All order data comes from the label and its packing slip |
| Plain labels (no packing slip) | Allowed with a warning | **Refused**: the batch stops and asks for the *Shipping label + Packing slip* download |
| Rules file vs interactive menu | Two modes (`--interactive`) | One screen: the rules are applied in order as a proposal; the owner can skip, move or add a pick for this batch before saving ([08 › Plan](08-pc-app-ui.md#3-plan)) |
| Undo a run | `--redo` (forget the last run) | **Revert** (undo the last batch) and **Amend** (redo the last batch in place), like git |
| Output folder | `work/<day>/` flat | `labels/<day>/batch <b>/` inside the data folder, one subfolder per batch, with a copy of the batch's label downloads |
| Saved-PDF name | Fixed `<n> <code> <name> ×<orders>.pdf` | Suggested the same way; the owner can change the whole name before saving |
| Packing-list default | `per-pdf`, layout `full` | One per batch (`whole`), layout `pick` (changeable per batch) |
| Fields in conditions | Includes the CSV-only fields | Only fields the label/slip carry (below) |

## Words used in this document

| Word | Meaning |
|---|---|
| **Batch** | One round of the daily steps ([business-process 01](../business-process/01-packing-workflow.md#daily-steps)): arrange shipment, download the labels (one or more files of up to 200 pages), prepare them in the app. Numbered 1, 2, 3, … within the day. In the app, one **Save** = one batch |
| **Draft** | A batch that has been read but not saved yet. It lives only in the app's memory; closing the app forgets it and nothing is written |
| **Plan** | The list of picks of a draft, in order, with their counts. Starts as the `categories.toml` entries in file order |
| **Data folder** | The folder where the app keeps `categories.toml` and the `labels/` folder with every day's output. By default the program's own folder ([Program folder](#program-folder-portable)) |
| **Pick**, **saved PDF**, **run**, **contents** | As in [01-requirements.md](01-requirements.md#concepts) |

## Inputs

| Input | Content |
|---|---|
| Label PDFs | The *Shipping label + Packing slip* files of one batch ([business-process 02](../business-process/02-data-sources.md#label--packing-slip-export-option)). One or more files; the seller centre splits a download into files of at most 200 pages |
| `categories.toml` | In the data folder. Each entry is one pick ([04-rules-file.md](04-rules-file.md)). Edited inside the app ([08 › Categories](08-pc-app-ui.md#5-categories)) or by hand |
| `couriers.toml` | The courier-deduction data, built into the program (same content as `script/packing/couriers.toml`) |
| The day's `state.json` | What earlier batches of the same day saved (below) |

### Every page must carry a packing slip

The packing slip is the only source of item data (product name, variation, Seller SKU,
quantity). A page that is not a continuation page and has no slip table stops the batch:

> **These labels have no packing slip.** 200 of 200 pages are plain shipping labels. In the
> seller centre, download the labels again with **Shipping label + Packing slip**.

Nothing is written. If only some pages lack the slip, the message names the file and the page
numbers.

### Fields the conditions can use

Only fields with a source on the label or slip:

| Field | Level | Source |
|---|---|---|
| `name`, `display_name`, `variation`, `seller_sku`, `line_quantity` | item | slip |
| `total_quantity`, `distinct_items` | order | computed from the slip |
| `tracking_id`, `courier`, `ship_by` | order | label |

A condition that uses a CSV field (`sku_id`, `product_category`, `channel`, `paid_time`,
`rts_time`, `created_time`) is an error in the rules check, with line and column:

> `line 18, col 8: field "product_category" needs the orders CSV, which the PC app does not read`

The app scan-filter fields (`category`, `batch`, `group`) stay errors as in 04.

## What happens in one batch

1. **Choose the files** with **Add files…** or by dropping them on the window
   ([08 › New batch](08-pc-app-ui.md#2-new-batch)). Each file's fingerprint (size + SHA-256)
   is checked against `state.json`: a file already used in an earlier batch today is marked
   *used in batch 1* (it can still be read; its orders are then left out as already saved).
2. **Read** the files in the order shown (default: by file name, which is download order),
   page by page, with a progress bar and a Cancel button. Per page: the Order ID, the
   label-level fields, the slip lines. Pages are grouped into orders (03 rules). Every order
   gets its **download position** (1, 2, 3, … over all files of the batch); runs keep this
   order inside a run.
3. **Check**: no-slip pages stop the batch (above); `Qty Total` differences, unknown couriers
   and missing tracking IDs are warnings.
4. **Duplicate guard**: orders already saved today are taken out and listed (Order ID,
   tracking ID, batch, saved-PDF number, time; the time is the batch's `save_time`).
5. **Plan**: the `categories.toml` entries in file order, each taking its orders from what is
   left; the leftovers form the last pick, the rest. The owner may change the plan for this
   batch only: skip a pick, move it up or down, add a one-off condition at any position,
   rename the file. Every change recounts the whole plan at once (all in memory, no file
   written).
6. **Save** (one button):
   1. Number the saved PDFs from the day's next number (after the last batch's last number).
   2. Re-order each pick's orders into runs.
   3. Write everything into a temporary folder `labels/<day>/batch <b>.partial/`: the saved
      PDFs (pages copied unchanged), the packing lists, `summary.txt`, and a copy of the label
      files read, in `download/` (amend reads them back).
   4. Rename the folder to `batch <b>` and add the batch to `state.json`.

   If anything fails before step 4, the temporary folder is deleted and the day is as before
   — there is never a half-written batch.

## Day folder

```
<data folder>/
  categories.toml
  settings.json
  labels/
    2026-10-07/
      state.json
      batch 1/
        1 A Sepatu ×172.pdf
        2 B Spion & Knalpot ×35.pdf
        3 Z Lainnya ×393.pdf
        packing-list.pdf
        summary.txt
        download/
          10-07_07-00-00_Shipping label+Packing slip_1.pdf
          10-07_07-00-09_Shipping label+Packing slip_2.pdf
          10-07_07-00-17_Shipping label+Packing slip_3.pdf
      batch 2/
        4 A Sepatu ×88.pdf
        5 Z Lainnya ×41.pdf
        packing-list.pdf
        summary.txt
        download/
          …
```

- Saved-PDF numbers run through the day (batch 2 starts at 4), so with the suggested names
  "sort by name" inside a batch folder is printing order, and a number on the packing list is
  unique for the whole day.
- Batch folders are `batch 1`, `batch 2`, …. Saved-PDF names are the owner's: suggested
  `<n> <code> <name> ×<orders>`, the whole name can be changed
  ([08 › Plan](08-pc-app-ui.md#3-plan)); two saved PDFs of one batch cannot have the same name.
  Characters not allowed in Windows file names (`\ / : * ? " < > |`) become `-`; a name is cut
  at 100 characters (ending in `…`).
- Packing-list files: scope `whole` (the default) → one `packing-list.pdf` per batch; scope
  `per-pdf` → `packing-list-<n>.pdf` per saved PDF; with several layouts the layout is added
  (`packing-list-full.pdf`, `packing-list-3-pick.pdf`).
- `download/` holds the label files exactly as read (about 1.3 MB per 200 pages, ~4 MB for a
  day of 600 orders), so amend works even when the originals in Downloads are gone.
- **The day** is today's date on the PC when the app is opened; the Day screen can switch to
  another day (e.g. a download just after midnight that belongs to the evening before).

### `state.json`

One per day folder. Everything needed to continue the day, revert or amend a batch.

```json
{
  "day": "2026-10-07",
  "batches": [
    {
      "batch": 1,
      "save_time": "2026-10-07T07:40:12+07:00",
      "files": [
        {"name": "10-07_07-00-00_Shipping label+Packing slip_1.pdf",
         "size": 1310720, "sha256": "9f2c…", "pages": 200}
      ],
      "pdfs": [
        {"number": 1, "code": "A", "name": "Sepatu", "file": "1 A Sepatu ×172.pdf",
         "orders": 172, "runs": 4}
      ],
      "packing_list": {"scope": "whole", "layouts": ["pick"]},
      "warnings": ["no courier could be deduced for: 580000000000000007"]
    }
  ],
  "saved": [
    {"order_id": "580000000000000001", "tracking_id": "JY0000001234",
     "batch": 1, "pdf": 1, "run": 5}
  ]
}
```

| Field | Holds | Why |
|---|---|---|
| `batches[].files` | Name, size, SHA-256, page count of each label file read, in reading order | Recognise a file used twice ("used in batch 1"); amend reads `download/` in this order |
| `batches[].pdfs` | Each saved PDF: number, pick code/name, file name, counts | Day screen without re-reading the PDFs; revert knows which files are the batch's |
| `batches[].packing_list` | Scope and layouts chosen | Amend starts with the same choice |
| `batches[].warnings` | The read warnings of the batch, as shown on the Plan screen | Day screen *Warnings ▸* without re-reading the files |
| `saved[]` | Each saved order: IDs, batch, saved PDF, run | Duplicate guard |

The file is written whole each time (write to `state.json.tmp`, then replace), so a crash
never leaves half a file. The Python script's `state.json` (with `invocation`) is a different
format; the app does not read `work/`.

## Revert and amend (only the last batch)

Like git: the last batch can be undone (**revert**) or redone in place (**amend**). An earlier
batch cannot, because later batches' numbers and duplicate guard depend on it.

**Revert last batch** — after a confirmation that names the batch and its files:

1. Delete the folder `batch <b>` (all its saved PDFs, packing lists and summary).
2. Remove the batch and its orders from `state.json`.

The orders count as "not saved today" again, and the next batch reuses the numbers. If a file
is open in a PDF viewer, Windows refuses the delete: the app says which file to close and
changes nothing.

**Amend last batch** — for "I fixed a category, do the last batch again":

1. The app reads the batch's label files again from its `download/` copy, in the order stored
   in `state.json`: the same pages in the same order as the first time.
2. The Plan screen opens with these orders and the current `categories.toml`; the duplicate
   guard ignores the batch's own orders.
3. **Save** writes a new `batch <b>.partial/` with the same first number and the same
   `download/` copy, then replaces the old folder, and replaces the batch in `state.json`. **Cancel** leaves the old batch untouched.

Amend after printing: the printed labels may no longer match the new files. The confirmation
says so ("Batch 2 was saved at 09:12. If you already printed it, print it again after
amending.").

## Packing list

- Same layouts as the script ([03 › Packing list](03-pc-script.md#packing-list-a4)), A4, fonts
  Arial and Consolas from `C:\Windows\Fonts`.
- Chosen per batch on the Plan screen: scope `per PDF` / `whole batch` / `none`, layouts any of
  `full`, `summary`, `pick`. Default: `whole batch`, `pick`. The default can be changed in Settings.
- Header as in 03 with the batch added: `Packing list · 2026-10-07 · batch 1 · 2  B  Spion &
  Knalpot`.
- **File size.** About 1.4 MB for one page, 1.6 MB for five (the script's: 62 KB), because
  PDFium writes each whole font into the file. Accepted; the printed result is the same.

## Architecture

```
┌──────────────────────── PC app (one .exe + pdfium.dll) ────────────────────────┐
│  Window: Svelte + TypeScript (screens of 08)                                   │
│        │  invoke("read_labels", …) / events ("read-progress")                  │
│        ▼                                                                       │
│  packing-app (Tauri 2, Rust): commands, the open draft, settings               │
│        │                                                                       │
│        ├──► packing-engine (Rust, no PDF code): rules + conditions, slip and   │
│        │    label text → orders, couriers, plan, runs, numbering, state.json   │
│        │                                                                       │
│        └──► packing-pdf (Rust): page text with positions, copy pages,          │
│             write packing lists  ──►  PDFium (pdfium.dll)                      │
└────────────────────────────────────────────────────────────────────────────────┘
packing-cli (Rust, developer tool): the same engine and the shared batch steps
(`packing-pdf` `batch.rs`) from the command line, for tests and sample runs
```

### Stack

| Need | Choice | Note |
|---|---|---|
| Language | Rust (stable, edition 2024; installed: 1.98) | One language for engine and window back end |
| Window | Tauri 2 (`tauri` 2.12) | Uses the Edge WebView2 that comes with Windows 11 |
| Screens | Svelte 5 + SvelteKit (static adapter, one page app) + TypeScript + Vite (`create-tauri-app`, template `svelte-ts`) | Plain CSS, no component library. Node 26 installed. Front-end build output `pc/app/build/` (ignored by `pc/app/.gitignore`) |
| PDF: read text with x/y, copy pages unchanged, write the packing list | `pdfium-render` 0.9.4 + `pdfium.dll` (Chrome's PDF engine; prebuilt by bblanchon/pdfium-binaries, tag `chromium/7881` = the crate's `pdfium_latest`) | `PdfPageText` gives each character with its box (replaces pypdf's `visitor_text`; runs are rebuilt, see *Text runs from PDFium*); `copy_page_range_from_document` copies pages; `PdfFonts::load_true_type_from_file` + text objects write the packing list (whole fonts embedded, so the file is about 1.5 MB — accepted). Licences MIT/Apache (wrapper), BSD-3/Apache (PDFium) |
| TOML | `toml` (read) + `toml_edit` (write back keeping comments) | `categories.toml`, `couriers.toml` |
| JSON | `serde`, `serde_json` | `state.json`, test fixtures |
| Dates | `jiff` | `ship_by`, `save_time`, day folder |
| Fingerprints | `sha2` | Label files used twice |
| Tests | `cargo test`; `svelte-check` for the screens | Fixtures shared with the Python script in `testdata/` |
| Build | `pc/tools/make-portable.sh`: `npm run tauri build -- --no-bundle`, then the program folder | No installer; `Packing.exe` + `pdfium.dll` in `pc/target/portable/Packing/` ([Program folder](#program-folder-portable)) |

### Code layout

```
pc/
  Cargo.toml                 workspace: crates/* and app/src-tauri
  crates/
    engine/                  package packing-engine
      src/rules.rs           categories.toml, condition parser/evaluator/printer (04)
      src/text.rs            a page's text runs with x, y (input from packing-pdf)
      src/label.rs           Order ID search, label fields, couriers
      src/slip.rs            slip table by position
      src/orders.rs          pages → orders, contents key, display names
      src/plan.rs            plan, picks from what is left, plan edits, runs, numbering
      src/day.rs             state.json, duplicate guard, revert, amend
      src/names.rs           file and folder names
      couriers.toml          built in with include_str!
    pdf/                     package packing-pdf
      src/read.rs            open files, page text with positions, page count, progress
      src/write.rs           copy pages into a saved PDF
      src/packing_list.rs    the three layouts
      src/batch.rs           the batch steps shared by CLI and app: check files, read, duplicate guard, save
    cli/                     package packing-cli (developer tool, not for daily use)
  app/
    package.json, vite.config.js, src/routes/   Svelte screens
    src/texts.ts                                every screen text (English)
    src-tauri/                                  package packing-app: commands, settings
  tools/get-pdfium.sh        downloads the pinned pdfium.dll into pc/vendor/ (git-ignored)
  tools/make-portable.sh     builds the program folder pc/target/portable/Packing/
```

`packing-engine` has no PDF dependency: it takes text runs and returns decisions, so all
reading and sorting rules are tested without PDF files. `packing-pdf` is the only code that
touches PDFium.

### Text runs from PDFium

The slip rules work on *runs* (a piece of text with its x, y), as pypdf gave them. PDFium gives
characters, so `read.rs` rebuilds the runs (measured in the spike 2.1, `pc/spike/`):

- **Two kinds of file.** The downloaded labels (made by wkhtmltopdf) hold one character per
  text object; files made by fpdf2 (`testdata/`) hold one whole run per text object. A page
  whose text objects hold several characters is read one run per text object, characters in
  their order. Otherwise runs are rebuilt by position: drop the space and line-break
  characters PDFium adds (their boxes are empty), sort by line then x, start a new run on a new
  line, a change of font name, or a gap over 5 pt; put a space for a gap over 1 pt. The font
  name matters: `Qty Total:` (bold) and its value are as close as two words.
- **Hyphen at a line end.** PDFium reports it as character U+0002; it is read as `-`, so the
  wrap rule (no space after a trailing `-`) applies.
- In fpdf2 files runs can overlap in x (a colon drawn over the label text), so those must not
  be sorted by position.

### Commands between window and Rust

| Command | Does | Returns |
|---|---|---|
| `day_overview(day)` | Read `state.json` of the day | Batches, saved PDFs, totals |
| `check_files(paths)` | Page count and fingerprint of each chosen file | Name, size, time, page count, "used in batch n" |
| `read_labels(paths)` | Start reading on a worker thread; emits `read-progress {page, pages}` | A draft id, then orders count, duplicates, warnings, or the stop error |
| `cancel_read()` | Stop reading | — |
| `plan(draft, edits)` | Apply the plan edits (skip, move, add condition, rename) and recount | Picks with counts, runs (contents × orders), suggested names |
| `save_batch(draft, plan, packing)` | Steps of *Save* above | Written files, summary |
| `revert_last(day)` / `start_amend(day)` | Revert / amend | New overview / a draft |
| `rules_load()` / `rules_check(text or entries, draft?)` / `rules_save(…)` | Categories editor | Entries, errors with line/col, live counts on the open draft |
| `condition_to_tree(text)` / `tree_to_condition(tree)` | Boxes editor: condition text → boxes, boxes → text (the engine's parser and printer, so the window has no second parser) | Tree of groups and rows, or an error with line/col / the text |
| `open_path(path)` / `show_in_folder(path)` | Open a PDF in the default viewer / Explorer | — |
| `settings_get()` / `settings_set(…)` | Settings | — |

PDFium is not thread-safe: all PDF work runs on one worker thread that owns the PDFium
library; commands send it jobs. The open draft (orders, the opened label documents) is kept
by that thread until it is saved, cancelled, or the app closes.

### Settings

`settings.json` in the data folder:

| Setting | Default |
|---|---|
| Packing-list scope / layouts | `whole batch` / `pick` |

Where the data folder is, is not a stored setting (next section). The app is in English; every
screen text is in one file (`pc/app/src/texts.ts`) so another language can be added later.

### Program folder (portable)

There is no installer. The program is one folder, put anywhere the owner can write (e.g.
`D:\Packing\`, not `Program Files`) and started from a shortcut:

```
Packing\
  Packing.exe
  pdfium.dll
  categories.toml   ┐
  settings.json     ├ the data folder (default: the program folder itself)
  labels\           ┘
```

- Copying the folder to another PC moves the program and all its data along.
- **New version**: replace `Packing.exe` and `pdfium.dll`; the data files stay.
- **Finding the data folder at start**: the app looks for `categories.toml` in its own folder.
  Found → that folder is the data folder. Not found → the app asks, at every start until it is
  found: **Use this folder** (writes the built-in `categories.toml` next to the program) or
  **Choose a folder…** (used until the app closes; the next start asks again). See
  [08 › Start](08-pc-app-ui.md#start).
- Settings › Data folder › **Change…** switches to another folder until the app closes.
- WebView2 keeps its browser cache under `%LOCALAPPDATA%\com.wthe22.olshop-packing\`; it holds
  no data of the app and is not needed on another PC.

## Testing

| Level | What | Expected |
|---|---|---|
| Engine unit tests | Every rule above on hand-built text runs | — |
| Shared fixtures | `testdata/conditions.json` (parse, print, evaluate, error texts), `testdata/expected-labels.json`, `testdata/expected-picks.json` | The same results as the Python script. Cases that use CSV fields expect the "needs the orders CSV" error in Rust |
| PDF tests | `testdata/labels-slip.pdf` (made up, from `make_testdata.py`): read, copy, packing list | Page counts, text found in the output |
| Local sample test (skipped without `samples/`) | The three *Shipping label + Packing slip* sample files | 601 pages, 600 orders, one order on 2 pages, J&T 517 pages; the picks, runs and page counts equal the Python script's on the same files and rules |
| By eye | Saved PDFs and packing lists from the sample run | Pages identical to the download (rendered side by side), layout as the script's |
