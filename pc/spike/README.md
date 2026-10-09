# pc/spike — Pdfium feasibility spike (throwaway)

Tests whether a pure-Rust PC app can replace the Python `pypdf` / `fpdf2` code for the
shipping-label pipeline, answering design question **D2 option 1** ("read labels, copy pages
and write the packing list with `pdfium-render` + a prebuilt `pdfium.dll`").

This directory is a scratch spike: one binary, one pure port of the reference slip logic, and a
few Python comparison scripts. Nothing here is production code. All real data stays in `out/`
(git-ignored); the outputs below are counts, sizes and times only.

## What it does

`src/main.rs` (binary `spike`), loading `../vendor/pdfium.dll` explicitly with
`Pdfium::bind_to_library`:

1. **Order ID** — two-pass search over the label text of every page (18-digit `5…` ID with a
   digit boundary check, then the `Order ID:`-prefixed form), mirroring `script/packing/labels.py`.
2. **Packing slip** — rebuilds pypdf-style `(x, y, text)` runs from Pdfium (two strategies, see
   below) and feeds them to a faithful port of `script/packing/slip.py` (`src/slip.rs`: header by
   position, Qty rows, per-column cell join with wrap, `Qty Total`, `Customer Message`). Dumps
   `out/rust-orders.json`.
3. **Copy** — copies 10 sample pages (including the one order that spans 2 pages) with
   `copy_page_range_from_document`, renders each original and copy at 150 DPI and counts
   differing pixels.
4. **Write** — `out/fonts.pdf` (1 A4 page, Arial / Arial bold / Consolas, made-up text) and
   `out/packing-list.pdf` (5 A4 pages, made-up table).

`src/slip.rs` has no PDF code and takes `Run { x, y, text }`, so it is testable on its own.

## How to run

```bash
# 1. pinned prebuilt pdfium.dll -> pc/vendor/ (git-ignored), SHA256 verified
bash pc/tools/get-pdfium.sh

# 2. Rust spike (release), prints the numbers
cd pc/spike && cargo build --release && ./target/release/spike.exe

# 3. Python reference + comparisons (repo venv)
.venv/Scripts/python.exe pc/spike/dump_python.py        # out/python-orders.json
.venv/Scripts/python.exe pc/spike/compare.py            # counts only
.venv/Scripts/python.exe pc/spike/check_fonts.py        # fonts + text readback
.venv/Scripts/python.exe pc/spike/packing_list_ref.py   # Python packing list on testdata
```

Both the Rust spike and `dump_python.py` read the real `samples/` by default and write to `out/`
(git-ignored); they print counts only. Point them at the synthetic fixture instead with
`SPIKE_LABELS` / `SPIKE_MATCH` (testdata values are made-up, so they may be printed):

```bash
cd pc/spike && cargo build --release
SPIKE_LABELS=D:/workspace/software/olshop-packing-ai/testdata SPIKE_MATCH=labels-slip ./target/release/spike.exe
cd ../.. && SPIKE_LABELS=D:/workspace/software/olshop-packing-ai/testdata SPIKE_MATCH=labels-slip .venv/Scripts/python.exe pc/spike/dump_python.py
.venv/Scripts/python.exe pc/spike/compare.py
```

## Measured numbers

| check | result |
|---|---|
| (a) Order ID on every page | **601 / 601** pass 1, 0 pass 2, 0 missed |
| (b) slip lines vs Python on `samples/` | **600 / 600 orders equal** (`lines`, `qty_total`, `customer_message`, `pages` all 0 diffs) |
| (b′) slip lines vs Python on `testdata/labels-slip.pdf` | **23 / 23 orders equal** (same four fields, 0 diffs) |
| (c) copy 10 pages (incl. the 2-page order) | 10 pages, **860,343 B**, differing pixels **0 on every page** (total 0, max 0) |
| (d) `out/fonts.pdf` | **1,365,328 B**, Arial/Arial-Bold/Consolas embedded **whole** (TrueType `/FontFile2`), text readback **9/9** |
| (d) `out/packing-list.pdf` | **1,563,732 B**, 5 A4 pages, text readback **11/11** |
| (d) Python packing list on testdata (`packing_list_ref.py`) | **61,626 B**, 1 page, 23 orders (fpdf2 embeds **subsets**) |
| (e) read all 601 pages (text + Order ID + slip) | Rust **1.60–1.62 s** vs Python **34.0–34.5 s** (≈ **21×**) |

Sample inputs: 3 PDFs, 601 pages, 600 orders (one order on 2 pages).
Synthetic input: `testdata/labels-slip.pdf`, 24 pages, 23 orders (one order on 2 pages), made up.

## Versions pinned

| item | value |
|---|---|
| pdfium-render crate | `0.9.4` (default feature `pdfium_latest` == `pdfium_7881`) |
| prebuilt PDFium | bblanchon/pdfium-binaries tag `chromium/7881`, asset `pdfium-win-x64.tgz` |
| asset size / SHA256 | 3,733,154 B / `73cc0de638ac2095e7445bf56a38200a5b7c7ca0e9f4ba144598f2457377ac08` |
| toolchain | rustc / cargo 1.98.0 |
| other crates | `regex 1`, `serde 1`, `serde_json 1`, `image 0.25` (matches pdfium-render `image_latest`) |

## Verdict: **VALIDATED**

`pdfium-render 0.9.4` + the prebuilt `pdfium.dll` (tag `chromium/7881`) reproduces the Python
pypdf/fpdf2 results for all four jobs — on the real `samples/` (600/600 orders, 601/601 Order IDs)
and on the synthetic `testdata/labels-slip.pdf` (23/23 orders) — and reads the 601 pages ~21× faster.

Reliable as-is:
- Order-ID search (601/601).
- Page copying via `copy_page_range_from_document` — pixel-identical at 150 DPI.
- Writing A4 PDFs with Arial/Arial-Bold/Consolas via `load_true_type_from_file` +
  `create_text_object`; text reads back.

Needs care (the interesting part):
- **Pdfium exposes text at two different granularities, and they need opposite handling.**
  - The **wkhtmltopdf samples** put **one glyph per text object**: `FPDFText_GetTextObject`
    returns a distinct single-char object for every character (measured: all 391,971 text objects
    across the 601 pages are single-char). pypdf's run boundary is therefore *not* in the object
    structure, and runs must be rebuilt from glyph geometry.
  - The **fpdf2 testdata** puts **one show operator per text object** (objects hold many chars), so
    there a run *is* one text object, and grouping by object reproduces pypdf's `visitor_text` runs
    exactly.
  The rebuilder inspects the page's first text objects and picks the matching strategy.
- **Geometry alone cannot reproduce pypdf on the fpdf2 pages.** There a run can be drawn *on top
  of* the previous one: `Qty Total`'s number starts at x 288.3 while `Qty Total:` starts at 252.2
  and is ~39 pt wide, and the `Customer Message` colon (x 70) sits inside `Customer Message`
  (x 10.5, ~74 pt wide). Any left-to-right / gap grouping merges them and even reorders the
  characters, giving `"Qty Total1:"` and `"Customer Mess: age…"`. Grouping by text object keeps each
  show operator whole regardless of where its glyphs land, exactly like pypdf.
- **On the per-glyph samples** Pdfium's character order is not the visual order (word pieces and
  generated spaces are interleaved), and the generated space / `\r` / `\n` characters carry
  zero-width boxes at unreliable positions, so they are dropped and spaces re-inserted from the
  gaps between real glyphs. No single gap threshold reproduces pypdf's run boundaries there:
  `Qty Total:` and its value are only 2.19 pt apart — the same as an intra-word space — so a gap
  rule alone merges them; what actually separates them is that `Qty Total:` is `Arial-BoldMT` and
  the value is `ArialMT`. The geometry path therefore splits on a **line change**, a **font
  name/size change**, or a **gap > 5.0 pt**, and inserts a space for a gap **> 1.0 pt**; with that
  the unmodified `slip.py` logic matches 600/600 on the samples.
- **Font size is not usable as a signal** on the samples (every char reports the same
  `unscaled_font_size`, 38.0); only the font *name* differs.
- **Pdfium reports a hyphen at a line end as U+0002** (its "soft hyphen" marker), not `-`. Both
  paths map it back to `-`, so `slip.py`'s wrap rule (no space after a trailing `-`) applies:
  `Yamaha-` + `Mio` -> `Yamaha-Mio` and `SPION-SCOL-DH-` + `CY` -> `SPION-SCOL-DH-CY`.

Recommendations for the real crate:
- Keep the run rebuilder as its own module with the two strategies above (text-object grouping
  when objects are operator-level, glyph geometry when they are per-glyph), and pin it with tests
  against pypdf's `visitor_text` output on both the samples and `testdata/labels-slip.pdf`.
- APIs used: `Pdfium::bind_to_library`, `PdfDocument`/`PdfPages::{get,len}`,
  `PdfPage::{text, objects}`, `PdfPageObjects::iter`, `PdfPageObject::as_text_object`,
  `PdfPageTextObject::chars`, `PdfPageText::chars`,
  `PdfPageTextChar::{unicode_string, origin, loose_bounds, tight_bounds, font_name,
  unscaled_font_size}`, `PdfPages::copy_page_range_from_document`,
  `Pdfium::create_new_pdf`, `PdfPages::create_page_at_end(PdfPagePaperSize::a4())`,
  `PdfFonts::load_true_type_from_file`, `PdfPageObjects::{create_text_object,
  create_path_object_line}`, and `PdfRenderConfig + PdfPage::render_with_config`.
- **File size:** pdfium embeds the whole Arial/Arial-Bold/Consolas on a 1-page document
  (~1.3 MB); fpdf2 subsetted the same fonts to 60 KB on the testdata list. If output size
  matters, add font subsetting (or accept the larger files) — pdfium-render does not expose
  subsetting directly.
- Pin `pdfium-render = "0.9.4"` and the matching `chromium/7881` DLL.

## Files

- `Cargo.toml` — standalone crate (`[workspace]`), deps pinned.
- `src/main.rs` — the spike binary.
- `src/slip.rs` — pure port of `slip.py` / Order-ID search.
- `dump_python.py` — `packing.labels.LabelSet.open(...).orders()` → `out/python-orders.json`.
- `compare.py` — differs the two order dumps, prints counts only.
- `check_fonts.py` — font embedding + text readback of `out/fonts.pdf` / `out/packing-list.pdf`.
- `packing_list_ref.py` — Python packing list from `testdata/`, for the size comparison.
- `../tools/get-pdfium.sh` — pinned, SHA256-verified `pdfium.dll` download.
- `out/` — all outputs (git-ignored): `rust-orders.json`, `python-orders.json`, `copy10.pdf`,
  `page-src-*.png` / `page-dst-*.png`, `fonts.pdf`, `packing-list.pdf`, `python-packing-list.pdf`.
