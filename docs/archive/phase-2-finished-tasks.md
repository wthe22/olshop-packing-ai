- [x] **2.1 PDFium spike** (D2; throwaway code in `pc/spike/`, git-ignored output). Download
  the pinned `pdfium.dll`; a small Rust program on the three sample *Shipping label + Packing
  slip* files: Order ID on every page (601 of 601), slip lines dumped as JSON and compared with
  a dump from the Python `LabelSet.orders()` (600 of 600 equal), copy 10 pages into a new file
  and compare renders side by side, write one A4 page with Arial and Consolas, time for reading
  all 601 pages (Python: 34.5 s), size of a packing-list PDF. Record the numbers here and in
  D2; if a check fails, stop and write the finding into D2 for the owner.
  - Result (`pc/spike/README.md`): `pdfium-render` 0.9.4 + `chromium/7881`. Order ID on
    601/601 pages; slip lines equal on 600/600 orders (all sample orders have one line), and on
    23/23 of `testdata/labels-slip.pdf` (multi-line, wrapped cells, message); plain files
    562/562 orders equal. 10 copied pages: 0 pixels differ (150 DPI). Fonts page and a 5-page
    packing list written, text reads back. Reading 601 pages: 1.6 s (Python 35 s on the same
    PC). Packing list 1.4 MB (1 page) / 1.6 MB (5 pages), whole fonts embedded; Python 62 KB
    → accepted, so D2 is answered. Run rebuilding from characters → 07 *Text runs
    from PDFium*.
- [x] **2.2 Scaffold.** `pc/` Cargo workspace (`crates/engine`, `crates/pdf`, `crates/cli`,
  `app/src-tauri`), Tauri app from `create-tauri-app` template `svelte-ts` in `pc/app/`,
  `pc/tools/get-pdfium.sh` (pinned version; output `pc/vendor/`, git-ignored), one passing test
  per crate, the empty window opens with `npm run tauri dev`. Log Rust/Node versions, the
  Tauri CLI and the PDFium download in [01-setup](../../development/01-setup.md). `pc/vendor/` is git-ignored; the
  front-end output `pc/app/build/` is ignored by the template's `pc/app/.gitignore`.
- [x] **2.3 Engine: rules.** `rules.rs`: `categories.toml` loading and validation, condition
  parser, evaluator (item-level "any line"), printer, error texts with line/column (also of the
  TOML file). Only the label/slip fields (07 › *Fields the conditions can use*); CSV fields give
  the "needs the orders CSV" error. Test: every case of `testdata/conditions.json` (CSV-field
  cases expect that error).
- [x] **2.4 Engine: labels and slips.** `text.rs`, `label.rs`, `slip.rs`, `orders.rs`: Order ID
  two-step search, multi-page orders, `ship_by`, courier deduction with the built-in
  `couriers.toml`, slip table by position, `Qty Total` check, no-slip stop, contents key,
  display names. Tests on hand-built text runs.
- [x] **2.5 Engine: plan and day.** `plan.rs`, `day.rs`, `names.rs`: plan from the rules, plan
  edits (skip, move, add, rename), picks from what is left, runs, numbering through the day,
  `state.json` (07 format), duplicate guard, file fingerprints, revert, amend, file and folder
  names. Test: `testdata/expected-picks.json`.
- [x] **2.6 PDF crate.** `read.rs` (open, page count, text runs with positions, progress),
  `write.rs` (copy pages), `packing_list.rs` (the three layouts, port of `packing_list.py`,
  batch in the header; about 1.5 MB per batch, whole fonts). Worker thread that owns PDFium.
  Tests on `testdata/labels-slip.pdf`: `testdata/expected-labels.json` matches; written PDFs
  have the right page counts and text.
  - Result: 23/23 testdata orders equal `expected-labels.json` (24 pages in 0.03 s); plain file
    gives the no-slip stop; copied pages keep their text; packing lists about 1.37 MB. Not
    reproduced: the grey colour of the "(continued)" line (printed black).
- [x] **2.7 Developer CLI and sample run** (local, not committed; replaces 1.15).
  `packing-cli prepare --labels … [--day] [--data]` writes a batch like the app. On the sample
  slip files with the repository `categories.toml`: 601 pages, 600 orders, one order
  on 2 pages, J&T 517 pages, courier unknown 83, tracking ID not found 1 (the Python numbers of
  1.15); the picks, runs and page counts equal the Python script's on the same files and rules;
  a second run saves nothing (all duplicates); revert then run again gives the same files;
  plain sample files stop with the no-slip message. Check saved PDFs and packing lists by eye.
  Record the numbers here (counts only, no real IDs).
  - Result (release build, rules = repository `categories.toml`, scope `whole`, layout
    `pick`): 3 slip files, 601 pages, 600 orders in 2.1 s (Python 41 s); courier unknown 83,
    tracking ID not found 1. Picks A 564 orders / 4 runs, B 36 orders / 17 runs; every order in
    the same saved PDF and run as the Python script, same file names, same page counts, same
    Order ID on every page in the same order. Second run: "All 600 orders were already saved
    today", nothing written. Revert, then run again: same files. Plain files: stop, "562 of 562
    pages are plain shipping labels". Saved PDFs 4.7 MB + 0.95 MB (Python 4.75 MB + 1.0 MB)
    after `write.rs` was changed to one import per saved PDF (page-by-page import gave 51 MB).
    Packing list 1 page (Python 1). By eye: left for the owner review.
- [x] **2.8 App: frame, Day screen, Settings, start** (08 › 1, 6, *Start*): finding the data
  folder (program folder, else ask each start), day switch, batches from `state.json`, Open /
  Open folder, all screen texts in `src/texts.ts`. Batch warnings on the Day screen and the
  Settings *About* row are done in 2.10.
- [x] **2.9 App: New batch** (08 › 2): Add files…, drag-and-drop, page counts, "used in batch
  n", reading with progress and Cancel, stop messages.
- [x] **2.10 App: Plan and Save** (08 › 3, 4): picks with counts and runs, skip/move/add,
  file names (whole name editable, same name twice refused), packing-list options (default one
  per batch), warnings, the `download/` copy, Save → Day screen.
- [x] **2.11 App: Revert and Amend** (07 › *Revert and amend*): confirmations, file-in-use
  message, amend from the batch's `download/` copy.
- [x] **2.12 App: Categories** (08 › 5): boxes editor (groups, rows, not), text view per
  category, live check and counts, Edit file as text, save keeping comments and unchanged
  condition texts. Test: every parse case of `testdata/conditions.json` goes text → boxes →
  text and gives the printer's text (66/66). Categories and Settings open over the current
  screen and keep an open draft; the boxes editor is also used in *Add a pick for this batch*.
- [x] **2.13 Portable build** (07 › *Program folder*): `pc/tools/make-portable.sh`
  (`npm run tauri build -- --no-bundle`, then `Packing.exe` + `pdfium.dll` into
  `pc/target/portable/Packing/`); copy the folder outside the repository, start it from a
  shortcut, the first start asks for the data folder, one batch on `testdata/` labels. Log in
  [01-setup](../../development/01-setup.md).
  - Result: `pc/tools/make-portable.sh` builds the release program (`--no-bundle`; the binary is
    `pc/target/release/packing-app.exe`, renamed to `Packing.exe`) and assembles
    `pc/target/portable/Packing/` (`Packing.exe` 7,792,128 B, `pdfium.dll` 7,211,520 B);
    `pc/tools/make-portable.bat` does the same from Explorer or `cmd` — it finds Git for Windows'
    `bash.exe` itself and runs the script, waiting for a key so a double-click's output stays
    readable (`--no-pause` for scripts). Tested
    from a copy outside the repo (`%TEMP%\portable-test\Packing`) started from a `.lnk`: first
    start asked for the data folder, *Use this folder* wrote the built-in `categories.toml`, one
    batch on `testdata/labels-slip.pdf` → 24 pages, 23 orders, 3 saved PDFs (A ×7, B ×8, Z ×8) +
    `packing-list.pdf` (1.4 MB) in `labels/<day>/batch 1/`. Sizes and steps in [01-setup](../../development/01-setup.md).
- [x] **2.14 User guide** (replaces 1.16). `docs/desktop/user-guide/README.md` (index) and
  `docs/desktop/user-guide/01-pc-app.md` for the owner (not a developer), under ~250 lines: setting up the
  program folder, first start and a new version; a normal batch step by step with screenshots taken on `testdata/` labels
  (made-up data); several batches a day; revert and amend; editing categories with 3–4 example
  conditions (link 04); every message with what to do. Add the folder to `docs/README.md`
  ("how to use a finished tool belongs in `user-guide/`") and to the Docs line in `AGENTS.md`.
  - Result: `docs/desktop/user-guide/01-pc-app.md` (≈210 lines) + index `README.md` + 8 screenshots in
    `docs/desktop/user-guide/images/` (12–55 KB each, taken on `testdata/labels-slip.pdf`, cropped to the
    window). Linked from `docs/README.md`, `AGENTS.md` and the "Use the PC app" row of
    `docs/desktop/development/README.md`.

- [x] **2.15 Packing list: the two display defects of review round 1.** (a) *The body started at
  the top margin, so on every page the first row or bar was printed over the header.* `Builder::new`
  started at `MARGIN` and `need()` restarted at `MARGIN`, while the header draws the title at
  `MARGIN` and the counts at `MARGIN + 7` and never advanced the cursor; the Python header leaves
  the cursor at 23 mm (`tools/analysis/packing/packing_list.py:153`). The body now starts at `BODY_TOP`
  (23 mm) in all three layouts, on the first page and after every page break, the `(continued)`
  line included. (b) *`×` printed as `ÿ`.* PDFium's non-CID font path writes `× ÷ é ® ™ ° ± ñ Ä ½ €`
  as `ÿ` (`cid = false`); its CID path writes them correctly but turns a plain space into U+00A0
  and `-` into U+00AD in the text layer. Fix: the three fonts stay non-CID, a CID copy of the same
  font is loaded only when a page needs one, and the affected characters are drawn with it — each
  run placed at PDFium's own character origin, because a text object's width is its ink bounds and
  drops the trailing spaces in front of the next run. `full`/`summary` grow to about 2.0 MB (the
  second copy of Arial); `pick` stays about 1.4 MB.
  - Result: done. All three layouts on `testdata/` and on the owner's 2026-10-11 batch 1 (54
    orders · 77 units · 9 runs — the same counts as his file) and batch 2: the header stands clear
    of the body and the item lines print `×1`. Read back with `pypdf`, the written files have no
    U+00A0, U+00AD, U+A78F or `ÿ` in the text layer. New tests:
    `the_body_is_never_printed_over_the_header` and
    `characters_pdfium_encodes_wrongly_are_drawn_and_still_read_back` (both fail when their fix is
    reverted) and the module test `only_the_characters_pdfium_encodes_wrongly_use_the_cid_font`;
    all 124 tests of the workspace pass. Design written into 03 › *Packing list (A4)* and 07 ›
    *PDF library*.

- [x] **2.17 Open folder: it opened Documents (review round 1).** `show_in_folder` ran
  `explorer.exe /select,<path>`. Explorer parses `/select,` itself and does not understand the
  quotes Rust puts around an argument that holds a space, so it never read the target and fell back
  to *Documents* (an empty target falls back to *Home*). The batch folder always has a space in its
  path (`…\labels\2026-10-11\batch 1`). *Open* (a file) and *Open folder* now share one call,
  `open_with_explorer`, which hands Explorer the path as its **only** argument: a folder opens as a
  folder, a file opens in its default program — what 08 › *1. Day* already said (*Open folder* opens
  the batch folder in Explorer).
  - Result: done. Checked on this PC, in the owner's own interactive session:
    `explorer.exe "/select,<the batch folder>"` opened no window of that folder (Explorer fell back
    to Documents/Home — the owner saw exactly that), while `explorer.exe "<the batch folder>"`
    opened *batch 1 - File Explorer* on the batch folder. New test
    `explorer_gets_the_path_as_its_only_argument` pins the argument list (one argument, no
    `/select,`); the 23 tests of the app crate pass.
