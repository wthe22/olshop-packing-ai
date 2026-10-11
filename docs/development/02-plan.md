# 02 — Build Plan

Phases in order. Each task ends with a commit. Tick the box (`[x]`) in the same commit that
finishes the task. A phase is done when all its tasks are ticked **and** the owner review is
noted as passed.

Status: **Phase 2 (PC app): tasks 2.1–2.15 and 2.17 done — the packing-list display defects and the
*Open folder* defect of the owner's review round 1 (2026-10-11, his own day's data) are fixed;
2.16 (the courier) is next, and 2.18 (the check of the saved PDFs, designed at the
owner's request on 2026-10-11) waits behind it.**
The owner decided that the PC gets a desktop app written in Rust (a
command line is not good for daily use), so phase 1 stops with its code done (270 tests) and
stays as the reference implementation and fallback; its two open tasks move into phase 2. Design:
[07-pc-app](../software-design/07-pc-app.md), [08-pc-app-ui](../software-design/08-pc-app-ui.md).
Open in [02-design-questions.md](../software-design/02-design-questions.md): D1 (Android, open
until phase 3), D17 (whether 2.18's check runs on every batch), D18 (the J&T Cargo logo, the
one the logo clue cannot read) and D19 (where the logos the owner teaches the app live). **D15 is answered**
(2026-10-11, the owner: two clues that must agree — the tracking-ID form and the service tag; the
rule is written into 03 › *Courier deduction*, so D15 is removed) and **D16 is answered** (2026-10-11): the courier data lives in the data folder, so a courier change is a file
edit the owner can make, not a new program version — task 2.19. D2 (the
PDF library) is answered: PDFium does all three PDF
jobs, and the bigger packing-list files (about 1.4 MB, 2.0 MB with `full`/`summary`) are accepted.

**Next step.** 2.16 (the courier: the form-and-tag rule, no owner answer needed), then
2.19 (the courier data into the data folder, D16's answer — no owner answer needed, so it can be
built at any time) and 2.18 (the saved-PDF check; its four choices are answered and D17 has a
"built for now" default, so it waits on nothing). Then the round-2 review at the end of Phase 2 — open the same day's packing
list (the header must stand clear, `×` must print), press *Open folder* (the batch folder must open,
not Documents), check the Day screen for a *Courier unknown* warning, and save one batch to watch
the check run (the progress line; a clean day must report nothing). The
answers belong in
[02-design-questions.md](../software-design/02-design-questions.md) — each question has a
"Built for now" default, nothing waits on them — or as a new session here; the phase-3 Android
work waits for this phase's review to pass.

---

## Phase 1 — PC script (stopped: reference implementation)

The Python script, built as the daily tool and then replaced by the Rust PC app (phase 2)
before it was used daily. Its code is finished and tested; it stays for good, frozen, as the reference the
Rust program is compared with and as a fallback when the app cannot start
([07-pc-app](../software-design/07-pc-app.md)).
Tasks 1.15 and 1.16 are not done here: the sample run is part of task 2.7, the user guide is
written for the app in task 2.14. There is no separate owner review of phase 1.

Read: [01-requirements](../software-design/01-requirements.md) (Concepts, PC script rules,
P1–P6), [03-pc-script](../software-design/03-pc-script.md),
[04-rules-file](../software-design/04-rules-file.md),
[business-process/02-data-sources](../business-process/02-data-sources.md).
Reference: `script/prototype/packing_list_preview.py` (working layout code for `full` and
`summary`; reads `samples/` directly; not production code).

- [x] **1.1 Scaffold.** `script/pyproject.toml` (package `packing`, deps `pypdf`, `fpdf2`,
  extra `dev` = `pytest`), empty modules as in 03-pc-script "Code layout", `.venv` (01-setup),
  one passing placeholder test. Log the venv step in 01-setup.
- [x] **1.2 Test data (CSV).** `testdata/make_testdata.py` writes `orders-1.csv` and
  `orders-2.csv` with the real 65-column header (business-process 02), trailing tabs on ID/time
  values, BOM, made-up IDs. Content covers: multi-line order; same packing list in different
  line order; `Default` variation; name with `|`; row with another status (ignored); order
  removed in export 2; order new in export 2; Tokopedia channel; quantity > 1. Plus
  `testdata/categories.toml`, a hand-checked `testdata/expected.json` (05-app-architecture
  "Logic in two places") and `testdata/conditions.json`.
- [x] **1.3 Orders (CSV).** `orders.py`: read CSV, strip values, filter status, group by Order
  ID, display names, signatures. Tests on testdata.
- [x] **1.4 Orders: paid and created time.** Add `paid_time` and `created_time` to the order
  record (`rts_time` exists); the conditions need them
  ([04-rules-file](../software-design/04-rules-file.md)). Tests.
- [x] **1.5 Rules (base).** `rules.py`: TOML loading + validation, condition parser (grammar in
  04-rules-file), evaluator with item-level "any line" semantics, error messages with line and
  column. `testdata/conditions.json` with parse/eval/error cases. Tests.
- [x] **1.6 Rules: pick-flow fields.** Date/time comparisons (`ship_by`, `paid_time`,
  `rts_time`, `created_time`) with the value forms in 04-rules-file; the new text fields
  `tracking_id` and `courier`; the CSV-only check (`sku_id`, `product_category`, `channel` and
  the CSV times error without a CSV); category errors reported with the line and column of
  `categories.toml` itself. Extend `conditions.json` and the tests.
- [x] **1.7 Labels (base).** `labels.py`: Order ID per page (two-step search), multi-page
  orders, page copying. Tests with label PDFs generated by the test itself (fpdf2, made-up IDs,
  one 2-page order, one spaced-character page).
- [x] **1.8 Label fields and slip.** `labels.py`: read the label-level fields (`In transit by`
  → `ship_by`, the courier text). New `slip.py`: read the slip table by text position (columns
  from the header words, a row per `Qty`, wrapped cells joined, no space after `-`, multi-page
  slips, `Qty Total` cross-check, `Customer Message`). Tests with generated label+slip PDFs.
- [x] **1.9 Courier deduction.** From the tracking-ID pattern, else the label text (courier
  name, web address, sort-code style); unknown → empty + warning listing the orders. Tests.
- [x] **1.10 Picks and day state (`picks.py`).** Merge the CSV over the slip with a warning per
  difference; the contents key; the picks in rules order taking from what is left; the final
  "rest" pick; re-ordering into runs; saved-PDF and run numbering through the day;
  `work/<day>/state.json`; the duplicate guard. Tests. (Replaces `batches.py`.)
- [x] **1.11 Packing list (base).** `packing_list.py`: layouts `full`, `summary`, `pick` as in
  03-pc-script. Test: PDFs are produced, page count sane, text contains numbers and tracking IDs
  (via pypdf).
- [x] **1.12 Packing list for picks.** Headings = saved PDF number + pick code/name; groups =
  runs (`3-05`); scope `per-pdf` / `whole` / `none`; error or skip with a message when there is
  no item data. Tests.
- [x] **1.13 Command line.** `prepare` with optional `--csv`, `--rules`, `--interactive`,
  `--layout`, `--packing-list`, `--day`, `--work`; screen summary and `summary.txt`; errors and
  warnings as in 03-pc-script.
- [x] **1.14 Test data (labels).** `testdata/make_testdata.py` also writes label PDFs (plain
  and with a slip) with made-up IDs, one 2-page order, wrapped slip cells; plus a hand-checked
  `testdata/expected-picks.json`.
- [ ] **1.15 Run on the samples** (local, not committed). **Moved to 2.7**; the notes below are
  the Python numbers 2.7 compares with. The three `Shipping label + Packing
  slip` files (601 pages, 600 orders, one order on 2 pages, every page has an Order ID; 517
  pages J&T) with and without the CSV — the sample CSVs are from another day and hold none of
  these orders, so with the CSV every order is warned "not in the CSV" and the slip values are
  used. Then the old plain-label files without slips: expect the "no item data" warning. Record
  the numbers here.
  - Done so far (reader level, `LabelSet.orders()`): slip files 600 orders in 34.5 s, all with
    item lines, `Qty Total` = sum on all 600, `ship_by` on all 600, J&T 517, courier unknown
    83, tracking ID not found 1. Plain files: 562 orders in 22 s, no slip, no `ship_by`
    (plain labels do not print it), J&T 511, IDX 1, unknown 50, tracking ID not found 1.
  - First CLI run (slip files, no CSV, a rules file with a pick on `product_category`) stopped
    correctly with `error: pick "C" uses field "product_category", which needs the orders CSV`.
    The CSV-only check runs before the labels are read (instant error).
  - Still to run: the slip files with the repository `categories.toml`; the slip files with
    `--csv samples/orders-06-1.csv` (expect the "CSV holds none of the label orders" warning); the plain files; a second
    invocation (duplicates skipped); `--redo`; `--interactive`; each layout and scope. Check
    the saved PDFs and packing lists by eye, page counts = label pages. Print counts only,
    never real IDs or names.
- [ ] **1.16 User guide.** **Moved to 2.14** (written for the app, not the script). `docs/user-guide/README.md` (index) and `docs/user-guide/01-pc-script.md`
  for the owner (not a developer), under ~250 lines: what it does; setup (link 01-setup);
  daily use step by step (which export to download, the copy-paste command, what is written to
  `work/<day>/`, printing); several downloads a day; fixing a mistake (`--redo`); interactive
  menu with an example session; the rules file with 3-4 example conditions (link 04); all
  options; every warning/error with what to do. Examples are real output of runs on `testdata/`
  (made-up data). Add the folder to the table in `docs/README.md` ("how to use a finished tool
  belongs in `user-guide/`"), to the Docs line in `AGENTS.md`, and point the "Run the script"
  row of `docs/development/README.md` to it.

**Owner review:** none (replaced by the phase-2 review).

---

## Phase 2 — PC app (Rust + Tauri)

A Windows program that replaces the script for daily use: label downloads in, picks adjusted on
screen, saved PDFs and packing lists out, revert/amend of the last batch. Written in Rust;
screens in Svelte + TypeScript in a Tauri 2 window. A portable program folder, no installer.
The PDF library is PDFium for all three PDF jobs — read the text with its positions, copy pages
unchanged, write the packing list (spike 2.1; the bigger packing-list files are accepted).

Read: [07-pc-app](../software-design/07-pc-app.md), [08-pc-app-ui](../software-design/08-pc-app-ui.md),
[03-pc-script](../software-design/03-pc-script.md) (the reading and sorting rules that carry
over), [04-rules-file](../software-design/04-rules-file.md),
[business-process/02-data-sources](../business-process/02-data-sources.md).
Reference: the Python package `script/packing/` — same rules, tested; port it module by module
(`rules.py` → `rules.rs`, `slip.py` → `slip.rs`, …).

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
  Tauri CLI and the PDFium download in 01-setup. `pc/vendor/` is git-ignored; the
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
  01-setup.
  - Result: `pc/tools/make-portable.sh` builds the release program (`--no-bundle`; the binary is
    `pc/target/release/packing-app.exe`, renamed to `Packing.exe`) and assembles
    `pc/target/portable/Packing/` (`Packing.exe` 7,792,128 B, `pdfium.dll` 7,211,520 B);
    `pc/tools/make-portable.bat` does the same from Explorer or `cmd` — it finds Git for Windows'
    `bash.exe` itself and runs the script, waiting for a key so a double-click's output stays
    readable (`--no-pause` for scripts). Tested
    from a copy outside the repo (`%TEMP%\portable-test\Packing`) started from a `.lnk`: first
    start asked for the data folder, *Use this folder* wrote the built-in `categories.toml`, one
    batch on `testdata/labels-slip.pdf` → 24 pages, 23 orders, 3 saved PDFs (A ×7, B ×8, Z ×8) +
    `packing-list.pdf` (1.4 MB) in `labels/<day>/batch 1/`. Sizes and steps in 01-setup.
- [x] **2.14 User guide** (replaces 1.16). `docs/user-guide/README.md` (index) and
  `docs/user-guide/01-pc-app.md` for the owner (not a developer), under ~250 lines: setting up the
  program folder, first start and a new version; a normal batch step by step with screenshots taken on `testdata/` labels
  (made-up data); several batches a day; revert and amend; editing categories with 3–4 example
  conditions (link 04); every message with what to do. Add the folder to `docs/README.md`
  ("how to use a finished tool belongs in `user-guide/`") and to the Docs line in `AGENTS.md`.
  - Result: `docs/user-guide/01-pc-app.md` (≈210 lines) + index `README.md` + 8 screenshots in
    `docs/user-guide/images/` (12–55 KB each, taken on `testdata/labels-slip.pdf`, cropped to the
    window). Linked from `docs/README.md`, `AGENTS.md` and the "Use the PC app" row of
    `docs/development/README.md`.

- [x] **2.15 Packing list: the two display defects of review round 1.** (a) *The body started at
  the top margin, so on every page the first row or bar was printed over the header.* `Builder::new`
  started at `MARGIN` and `need()` restarted at `MARGIN`, while the header draws the title at
  `MARGIN` and the counts at `MARGIN + 7` and never advanced the cursor; the Python header leaves
  the cursor at 23 mm (`script/packing/packing_list.py:153`). The body now starts at `BODY_TOP`
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
- [ ] **2.16 Courier: the SiCepat labels** (review round 1; D15 was answered by the owner on
  2026-10-11, D16 moved into 2.19). 23 of the owner's 98 orders get no courier (J&T Express 74,
  SiCepat 23, IDX 1; the day warns 15 + 8): SiCepat labels print their name **only as a logo
  image** and their tracking ID is 12 digits starting `00`, a form `couriers.toml` marks as
  shared (`tracking_unique = false`; J&T Cargo is also 12 digits), so `deduce_courier`
  (`crates/engine/src/label.rs:217`) refuses and warns. The 2026-10-07 sample is the same shape:
  82 of 601 pages. **The owner's rule (2026-10-11): two clues, and the courier is named only when
  they agree** — the **logo** (the page's logo image, its encoded bytes hashed) and the
  **tracking-ID form** (`00` → SiCepat, `57` → J&T Cargo, both now unambiguous). 03 › *Courier
  deduction* holds the rule; the logo hashes, the ask-the-owner flow and J&T Cargo's unreadable
  logo are D18. Code: `couriers.toml` gains a logo-hash list per courier and a J&T Cargo entry,
  and `deduce_courier` hashes the page's logo image (`FPDFImageObj_GetImageDataRaw`) and requires
  the two clues to agree; a hash that is not in the table is asked of the owner once
  (`courier-logos.toml`, [08 › the logo prompt](../software-design/08-pc-app-ui.md#the-logo-prompt):
  *I don't know* is remembered per hash, *Skip all* only silences the rest of that batch and
  writes nothing — no day flag, no `state.json` entry).
  Tests: every measured sample page keeps the courier it has now, a logo with a foreign form warns,
  an unseen logo asks, the two slip-continuation pages stay unknown. Then re-check the owner's day
  (the warning must be gone for the SiCepat orders) and rebuild the portable folder.
  - Open sub-point: the owner reports having seen a J&T Express tracking ID with a **`JT`** prefix
    (2026-10-11). The samples hold none — all 2,086 J&T Express rows of the 2026-10-06 exports and
    all 5,570 letter-prefixed tokens on the 1,269 sample pages are `JY` + 10 digits — so the form
    stays `JY\d{10}` until a label is in hand; then it is a one-line data edit
    (`JY\d{10}|JT\d{10}`, the pattern is anchored as a full match), and because the logo must
    agree, a wrong entry there can only turn a name into a warning, never a warning into a wrong
    name.
  - Measured for D15 (2026-10-11, counts and shapes only; method: the Python reader page by page
    and the exports read by provider): the 601 sample pages → J&T Express 517, no courier 84
    (82 with a `00` + 10-digit tracking ID, 1 slip continuation, 1 `TikTok Shop` label with no
    tracking ID); `sicepat` on 0 pages, in the plain and in the whitespace-stripped text; the
    2026-10-06 exports name SiCepat 173 orders (`00…`) and J&T Cargo 1 (`57…`). The two 12-digit
    forms do not collide. **Both J&T Cargo labels were found on 2026-10-11** (`0708-orders.pdf`
    p13, its slip continuing on p14; `10-07_00-03-17_Shipping label_3.pdf` p114): both print the
    `57…` form, the second one's order is the row the export itself names J&T Cargo, and both
    print `FastTrack` as text. Over the samples as they now stand (9 files, 1,269 pages — the
    owner renamed `10-orders.pdf` to `order w jt cargo.pdf` and added `order w idx.pdf`, 40 pages
    of 2026-10-11, while this was measured): J&T Express 1,114 · SiCepat 148 · J&T Cargo 2 ·
    IDX 2 · `TikTok Shop` 1 · slip continuations 2. Every page is one order except that one Cargo
    order. The two clues (the owner's rule): the service tag read from the header strip gives
    J&T Express `EZ` 453 · `ECO` 253 · `NDD` 408, SiCepat `NDD` 41 · `REG` 73 · `HALU` 34,
    J&T Cargo `FastTrack` 2, IDX `REG` 2, TikTok Shop `GKS` 1 — form and tag agree on all 1,266
    label pages that have a tracking form, and the one `TikTok Shop` label has a tag but no form
    (so it stays unknown); the 2 slip continuations have neither. The courier logo is an embedded
    image whose *encoded bytes* (1.5–15 KB, so hashing costs ~0.01 ms a page) hash to one stable
    value per courier for J&T Express, SiCepat, IDX and TikTok Shop across all 9 files; J&T
    Cargo's logo is a *masked* image re-rastered between exports (680×156 / 683×157), so it has no
    stable hash (D18). The barcode and QR are vector paths and every page also carries a
    page-wide frame image.
    The spill: the label takes the top 70% of the page on every style (the slip starts at 126.5 pt
    on all four), so only the slip's own content spills — the sample's one spilled order had a
    single item line and a buyer message.
  - Result: (not started; next, nothing waits for the owner)
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
- [ ] **2.18 Check the saved PDFs before the batch is saved** (the owner's request, 2026-10-11;
  designed here, not built yet). Wrong code could copy a label page twice, drop one, put an order
  in two saved PDFs or in none, or write the pages in the wrong order — and today that is only
  found at the printer. After the saved PDFs and the packing lists are written and before
  `batch <b>.partial/` is renamed, the app reads its own output back (the same page reader as
  *What happens in one batch* step 2, so no new parsing) and compares it with the plan: every
  order exactly once, in one saved PDF, page for page and in order, and the packing list's
  printed counts equal to the plan's.
  - **Two severities, never one list.** *Errors*: a page missing, a page twice in one PDF, an
    order in two saved PDFs, an order in no PDF, a page holding another order's label, a written
    page count that differs from the plan, packing-list counts that differ. *Warnings*, lower
    severity: the pages are all there exactly once but the **order** inside a saved PDF differs
    from the run order. The dialog, the Day screen and `state.json` keep the two lists apart.
  - **Save anyway, and it stays visible.** The dialog names the findings in plain words and offers
    *Save anyway* / *Cancel — nothing is saved*. Cancel deletes the temporary folder, leaves
    `state.json` untouched and keeps the draft (the Plan screen with the read labels) — what the
    all-or-nothing save already gives on an error. Saving anyway writes both lists into the
    batch's `state.json` entry (`errors`, `warnings`, defaulted so an older day folder still
    loads), prints them in `summary.txt`, and the Day screen shows them on the batch card
    (*Saved with 2 errors ▸*) until the batch is reverted or amended — never a message that
    scrolls away.
  - **Shown while it runs.** The check costs 1–2 s on the owner's big day (600 pages, 5 saved
    PDFs), so *Save batch* becomes a blocking-thread command emitting a progress event
    (`save-progress {phase, page, pages}`, phases *Writing* / *Checking*) like the read's
    `read-progress`, with the phase and a bar on the Plan screen — no window that looks frozen.
    This also removes the freeze during the writing itself, which is the same synchronous command
    today.
  - Code: `engine/src/verify.rs` (the arithmetic, no PDF), `pdf/src/verify.rs` (page counts, the
    read-back comparison, the packing-list counts), called from `pdf/src/batch.rs` between the
    writes and the rename; `PlanScreen.svelte` + `texts.ts` for the dialog and the two lists.
    Design into 07 › *What happens in one batch* (a *Check before saving* step) and 08 › *3. Plan*
    / *4. Messages* in the same commit.
  - Tests: a doubled page, a dropped page, an order in two picks (and in none) and a swapped page
    order must each be reported, each in its own list (the swapped one as a warning only); the
    tests fail when the check is switched off, as the 2.15 display tests do. Behind it: the
    smaller seams too — a page count changed in a written file, a packing-list count changed.
  - Open: D17 (every batch or only a large one).
  - Result: (not started)
- [ ] **2.19 The courier data in the data folder** (D16, answered by the owner 2026-10-11). The
  courier list — name, text clues, tracking forms — is built into the program today, so a new
  courier, a corrected form or D15's answer needs a new program version. It becomes a data file
  like `categories.toml` (same handling): the app writes the built-in file into the data folder on
  first start, *Start* and *Settings › Change…* treat it as they treat `categories.toml`, and a
  broken file stops the read with its line. After this, a courier change is an edit the owner can
  make, and the shipped default still carries the built-in list into a fresh data folder.
  - Code: the loader in `crates/engine/src/label.rs` reads the data folder's file (built-in copy
    as the default), the first-start write and the Settings/Start handling next to
    `categories.toml`, and the broken-file message. Design is written into 07 › *Inputs*, *Day
    folder*, *Program folder* and 08 › *Start* / *6. Settings*; D16 is removed from
    02-design-questions.md because its answer is written here.
  - Result: (not started)

**Owner review (round 1, 2026-10-11, on the owner's own day's data).** The owner ran a real day
through the portable folder (2 batches, 4 label files, 98 orders, no terminal) and reported two
things; checking them found a third.

| # | Reported / found | What it is | Cause |
|---|---|---|---|
| 1 | "packing list isn't displayed correctly, some text overlap" | Every page of every layout prints the first body row or bar over the header (title 10–17 mm, counts 17–22 mm): the body starts at the top margin (10 mm) instead of below the header (23 mm) | The Rust header is a set of fixed ops that never advances the cursor; `Builder` starts and restarts at `MARGIN` |
| 2 | found while checking 1 (the owner has not seen it: the pick layout has no `×`) | `×` prints as `ÿ`, in the drawn glyph **and** in the text layer, in the run lines of `full`/`summary`; `·` and `—` are right | PDFium writes U+00D7 wrongly for a non-CID TrueType font; the app loads the fonts with `cid = false` |
| 3 | "some couriers are not recognized by the current parser" | 23 of 98 orders save with an empty courier | SiCepat labels carry the name **only as a logo image**, and their `00` + 10-digit tracking form is marked as shared with J&T Cargo, so the deduction refuses |
| 4 | "open folder doesnt open the correct folder. it opens the user's userprofile documents instead" | *Open folder* on a batch and on the data folder opened *Documents*, later *Home* — never the folder | `explorer.exe /select,<path>`: Explorer parses `/select,` itself and does not understand the quotes Rust puts around an argument with a space in it, so it read no target and fell back to Documents (*Home* when the target is empty) |

Evidence: the owner's `batch 1`/`batch 2` `packing-list.pdf` (both 1 page, layout `pick`)
rendered and read back, against the Python reference (`pc/spike/out/python-packing-list.pdf`);
`testdata/labels-slip.pdf` through `packing-cli prepare --layout full,summary,pick` — all three
layouts show the overlap; the 4 label files of 2026-10-11 and the 3 of `samples/` read for the
courier clues, counts only; the pages with no courier text rendered (they are `SICEPAT EKSPRES`,
the name only in the logo); PDFium probed with a non-CID and a CID font; the two Explorer forms run
from a probe in the owner's own session (the fixed form opened *batch 1 - File Explorer* on the
batch folder, the `/select,` form opened no such window and fell back to Documents/Home). The owner's data folder
was only read — nothing in it was written to or changed; every probe wrote outside it.

**Owner review (round 2).** After 2.15–2.18: open the same day's packing list (the header
must stand clear of the body; a `summary` or `full` list must print `×`, not `ÿ`), press *Open
folder* on a batch card (the batch folder must open, not Documents) and on the Settings screen (the
data folder), check the Day screen shows no *Courier unknown* warning, save one batch (the check
must show its progress and, on a clean day, report nothing — 2.18), and print one packing list.

---

## Phase 3 — Android camera spike

Answers one question before the app is built: does phone-camera scanning of the printed labels
work fast and reliably enough?

Read: [05-app-architecture](../software-design/05-app-architecture.md) (stack, scan pipeline).

- [ ] **3.1** Install Android Studio and SDK; log in 01-setup.
- [ ] **3.2** Create `android/` (Kotlin, Compose, `minSdk 26`, application id
  `com.wthe22.packingchecker`), modules `app` + `core` (pure Kotlin).
- [ ] **3.3** One screen: CameraX preview + ML Kit (bundled), QR + Code 128; shows each decoded
  value with time; 2 s rule; lens switch button; beep on new code.
- [ ] **3.4** Install on the owner's phone (model and Android version → 01-setup log).

**Owner review:** scan a stack of real printed labels. Record in this file: scans per minute,
QR vs barcode which reads first, distance, light, problems. Decide whether anything in the scan
design changes.

---

## Phase 4 — App core (no camera)

Read: 01-requirements A1, A3, A4, A6; 05-app-architecture (database, import);
06-app-ui (sessions, orders, import summary).

- [ ] **4.1** `core`: CSV import, display names, signatures, condition language, TOML
  read/write, batch diff, group numbering, status from events. JVM tests on `testdata/` must
  match `expected.json` and `conditions.json`; local test on `samples/` must match
  `samples/expected-06.json`.
- [ ] **4.2** Room database (tables of 05-app-architecture), repositories.
- [ ] **4.3** Sessions screen: create, rename, open, delete with confirmation.
- [ ] **4.4** Import CSV with summary and Apply / Cancel.
- [ ] **4.5** Orders list + detail, filters, search, grouped view.
- [ ] **4.6** Manual marks and undo; history screen.
- [ ] **4.7** Strings in English and Indonesian from the start (`values/`, `values-in/`).

**Owner review:** import a real day (two CSVs), compare numbers with the script's packing list,
mark and undo some orders by hand.

---

## Phase 5 — Scanning

Read: 01-requirements A2; 05-app-architecture (scan pipeline); 06-app-ui (scan, messages,
condition editor).

- [ ] **5.1** Scan screen from the spike: pipeline, verdicts, result card, change buttons,
  recent marks with undo.
- [ ] **5.2** All scan messages, sounds, vibration; Process/Cancel dialog.
- [ ] **5.3** Condition editor (boxes + text view, live match count); saved scan filters.
- [ ] **5.4** Type-last-digits entry.

**Owner review:** check a real batch with the app, including filter use, a wrong packing, a
label problem, a pending order, an accidental scan undone.

---

## Phase 6 — Files and settings

Read: 01-requirements A6, A7; 05-app-architecture (session file, settings file); 06-app-ui
(More, settings).

- [ ] **6.1** Session export/import `.zip` (Replace / Keep both).
- [ ] **6.2** Categories screen; import/export `categories.toml` (file from the script must
  load unchanged; file written by the app must work in the script).
- [ ] **6.3** Settings export/import `.zip`; language System/English/Indonesia; delete
  protection; sound/vibration switches.

**Owner review:** move a session to a second device and back; edit categories in the app,
use the exported file in the script.

---

## Phase 7 — Layouts and polish

Read: 06-app-ui (all layouts, problems page).

- [ ] **7.1** Phone landscape, tablet portrait and landscape (list-detail) for every screen.
- [ ] **7.2** Problems page.
- [ ] **7.3** Release build signed with a key kept outside the repository (log where in
  01-setup).

**Owner review:** a full working day with the app.
