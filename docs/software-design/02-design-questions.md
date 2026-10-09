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

---

The questions below are about the **PC app** (phase 2). Read
[07-pc-app.md](07-pc-app.md) (behaviour and architecture) and
[08-pc-app-ui.md](08-pc-app-ui.md) (screens) first. In short: a Windows program, written in
Rust with a Tauri window, that does what the Python script does, with buttons instead of a
typed command.

## D2 — Which PDF library the Rust program uses

**The problem.** The program must do three things with PDFs: (1) read the text of each label
page *with its position* — the packing-slip table is read by where each word sits on the page,
not by reading order; (2) copy label pages into the saved PDFs **unchanged** (the courier's
label must print exactly as downloaded); (3) write the A4 packing list. In Python this was
`pypdf` (read, copy) and `fpdf2` (write). Rust has no single pure-Rust library proven for all
three.

**Options.**

1. **PDFium** (`pdfium-render` + `pdfium.dll`). PDFium is the PDF engine inside Google Chrome;
   `pdfium.dll` (~5 MB) ships next to the program. It reads each character with its box, copies
   pages between documents, writes text with TrueType fonts (Arial, Consolas), and can also draw
   a page as an image (useful later for a preview). One library for all three jobs.
   - Good: it is what Chrome uses to open these labels, so reading them is very likely to work;
     least code to write.
   - Bad: one extra file to ship and to update by hand (a pinned version is downloaded by a
     script); written in C++, called from Rust.
   - Effort: **low–medium**.
2. **Pure Rust**: `lopdf` (copy pages) + `pdf-extract` (text with positions) + `krilla`
   (write the packing list).
   - Good: no extra DLL; everything is Rust source.
   - Bad: three libraries; copying pages between files with `lopdf` is done by hand (moving
     each page's fonts and images along); `pdf-extract`'s text quality on these labels
     (made by `wkhtmltopdf`; the spaced-character IDX labels) is unknown.
   - Effort: **medium–high**, with risk.
3. **PDFium for reading and copying, `krilla` for the packing list.** Smaller packing-list
   files: `krilla` embeds only the letters used, PDFium may embed whole fonts (Arial is about
   1 MB).
   - Effort: **medium**.

**How it is decided.** The first task of phase 2 is a **spike** (a short throwaway test) with
option 1 on the sample downloads: an Order ID found on all 601 pages, slip lines equal to the
Python script's on all 600 orders, copied pages that look identical, a packing-list page with
both fonts, the time to read 600 pages, and the packing-list file size. If the spike fails, the
result is written here and option 2 or 3 is tried.

**Built for now: option 1**, confirmed or rejected by the spike.

## D3 — The Python script after the rewrite

**The problem.** The Python script of phase 1 works (270 tests) but is replaced by the Rust
program for daily use. Keeping two programs that do the same thing means every rule change is
made twice; deleting it too early loses the easiest way to check that the Rust program gives the
same result.

**Options.**

1. **Keep it as the reference until the phase-2 review, then delete it.** During phase 2 the
   Rust program is compared with it on the same files (same picks, same runs, same page counts).
   After you accept the PC app, `script/` is deleted (git keeps the history). The shared test
   files in `testdata/` stay, and so does their generator `make_testdata.py` (Python stays
   installed).
2. **Keep it for good**, frozen (no new features), as a fallback when the app cannot start.
3. **Delete it now** and test the Rust program only against the hand-checked test files.

The two unfinished phase-1 tasks are handled the same way under every option: the sample run
(1.15) becomes part of the Rust sample test (2.7), and the user guide (1.16) is written for the
app instead (2.14). See [02-plan.md](../development/02-plan.md).

**Built for now: option 1.**

## D4 — Where the app keeps its files (the data folder)

**The problem.** The script wrote into `work/` inside the code repository, which only makes
sense for a developer. An installed program needs its own place for `categories.toml` and the
`labels/<day>/` folders. That place should be easy to find in Explorer (you open the saved PDFs
from there too) and should be in your backup.

**Example.** With option 1, batch 1 of 7 October is in
`C:\Users\<you>\Documents\Packing\labels\2026-10-07\batch 1\`.

**Options.**

1. **`Documents\Packing`**, asked once at first start, changeable in Settings. **Built for
   now.**
2. **A folder next to the program** (portable: copy the folder to another PC and everything
   moves along). Bad: Windows does not let programs write into `Program Files`, so this only
   works when the program is not installed there (see D11 option 2).
3. **Inside the repository** (`work/`), as now — only works on the development PC.

## D5 — Batch folder and file names

**Decided already:** `labels/<day>/<batch>/<my group names>.pdf` — a top-level `labels`
folder, one folder per day, one subfolder per batch, the saved PDFs named by you (a suggested
name may be offered), numbering through the day. Still open are three details.

**(a) The batch folder name.**

1. `batch 1`, `batch 2`, … **Built for now.**
2. `1`, `2`, … (short, but next to files that also start with a number it is easy to confuse).
3. `batch 1 07-40` (with the save time: easier to match with a printed packing list; longer).

**(b) The suggested PDF name and what you can change.** The suggestion is the script's name,
e.g. `6 A Sepatu ×130` (saved-PDF number 6, pick A *Sepatu*, 130 orders). You can type over it
on the Plan screen.

1. **The number stays in front, you change the rest** (`6 ` + your text). Files then always
   sort in printing order and the number matches the packing list. **Built for now.**
2. You can change the whole name, number included.
3. No suggestion: you type a name for every PDF (slower every batch).

**(c) Packing-list file names.**

1. `packing-list-6.pdf` (one per saved PDF) or `packing-list.pdf` (one for the batch), inside
   the batch folder, as in the script. **Built for now.**
2. Named after the saved PDF: `6 A Sepatu ×130 - packing list.pdf`, so a saved PDF and its
   packing list sort next to each other.

## D6 — Amend: where the app gets the label pages back from

**The problem.** "Amend the last batch" (decided: like git) means: redo the last batch with
changed categories, in place. To redo it, the app needs the batch's label pages again. The
original downloads may have been moved or deleted by then.

**Example.** Batch 2 (129 orders) was saved at 09:12. At 09:30 you notice that the spion orders
went to *Lainnya* because pick B was spelled wrong. You fix pick B and press **Amend**.

**Options.**

1. **Read the batch's own saved PDFs.** Together they hold every page of the batch. The order of
   the original download (needed to keep the page order inside a run) is stored per order in
   `state.json` when the batch is saved. Works even when the downloads are gone. **Built for
   now.**
2. **Keep a copy of the downloaded files** in the batch folder (`batch 2/download/`). Simple and
   exact, but doubles the disk use (about 1.3 MB per 200 pages, ~4 MB for a day of 600 orders).
3. **Use the original files** where they were; if they are gone, ask you to choose them again.

## D7 — Printing

**The problem.** Today you print each group's labels separately (the printer stops at the end
of each group). The app has to hand the saved PDFs to the printer somehow.

**Options.**

1. **Open in the PDF viewer**: each saved PDF and packing list has an **Open** button; it opens
   in the PC's default PDF viewer, and you print from there as today. **Built for now.**
2. **Print button in the app** that sends a file straight to a printer chosen in Settings
   (label printer, A4 printer), without opening a viewer. Saves a few clicks per PDF, but
   needs a printing helper (e.g. the free SumatraPDF, installed separately) or drawing every
   page for the Windows printer — more work and more that can go wrong.
3. **Print the whole batch** with one click, PDF after PDF (needs option 2's printing; the
   printer would not stop between groups unless the printer itself is set to).

## D8 — Language of the app

**The problem.** The warehouse team uses the printed packing list, not the app; the app is used
by you. The Android app was planned in Indonesian and English from the start.

**Options.**

1. **English only** for now; every text kept in one file so Indonesian can be added later.
   **Built for now.**
2. **English and Indonesian** from the start, switch in Settings (more text to write and check
   now).
3. **Indonesian only.**

## D9 — Finding today's label downloads

**The problem.** Each batch is 1–3 PDF files in the Downloads folder. Choosing them in a file
dialog every time is slow, and choosing a file of an earlier batch is an easy mistake.

**Options.**

1. **List of today's label downloads** on the New-batch screen: PDFs in the Downloads folder
   changed on the shown day whose name contains `Shipping label` (the seller centre's file
   name looks like `10-07_11-02-10_Shipping label+Packing slip_1.pdf`). Files not used yet are
   ticked; files used earlier today are marked *used in batch 2*. Plus **Add files…** and
   drag-and-drop for anything else. **Built for now.**
2. **Only Add files… and drag-and-drop** (simpler; the "used in batch 2" check still runs on
   the chosen files).

## D10 — Pick C (`product_category`) in `categories.toml`

**The problem.** Decided: `product_category` is never used (it is the platform's category, not
the shop's), and the orders CSV is not read any more. Your `categories.toml` still has pick C
*Motor lainnya* with `when = 'product_category contains "sepeda motor"'`. The app refuses the
file until pick C changes.

**Options.**

1. **Delete pick C.** Its orders go to *Z Lainnya* (the rest). **Built for now**: the app shows
   the error and you delete the entry in Categories; the app does not change the file by itself.
2. **Rewrite pick C on the product name**, e.g. `name contains "motor"` — you give the words.
3. **Rewrite pick C as "not A and not B"** — which is what the rest already is, so it would only
   rename *Lainnya*.

## D11 — How the app is installed and started

**The problem.** The app is built from the source on this PC. You need a normal way to start it
(Start menu or desktop icon) and a way to get a new version.

**Options.**

1. **Installer** (`Packing Setup.exe`, made by the Tauri bundler): installs for your Windows user
   only (no administrator rights needed), adds a Start-menu entry; a new version is installed
   over the old one. Not code-signed, so Windows SmartScreen warns once ("More info → Run
   anyway"). **Built for now.**
2. **Portable**: one folder with `Packing.exe` and `pdfium.dll`, started from a shortcut; a new
   version replaces the folder.
3. **No installation**: run it from the repository with `npm run tauri dev` (developer use only).

## D12 — How categories are edited in the app

**The problem.** A category's condition is text in the condition language
(`not name contains "sepatu" and (name contains "spion" or …)`). The Android design has a
"boxes" editor where each row is chosen from lists (field · operator · value), so nothing is
typed except the value ([06-app-ui.md](06-app-ui.md#condition-editor-scan-filters-and-categories)).

**Options.**

1. **Condition as text**, checked while typing (error with line and column), with the count
   each category takes in the open batch, the list of field names and examples; plus **Edit as
   text** for the whole file. **Built for now** ([08 › Categories](08-pc-app-ui.md#5-categories)).
2. **Boxes editor** like the Android design, with the text view as an alternative. Easier for
   rare edits, more work now (and built again in Kotlin for the app).
3. **No editor in the app**: edit `categories.toml` in Notepad; the app only checks it and
   shows errors.
