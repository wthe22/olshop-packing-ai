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

The question below is about the **PC app** (phase 2): [07-pc-app.md](07-pc-app.md)
(behaviour and architecture), [08-pc-app-ui.md](08-pc-app-ui.md) (screens).

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
