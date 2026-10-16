# Architecture

The PDF library is **PDFium** for all three PDF jobs: read the text with its positions, copy
pages unchanged, write the packing list. Its packing lists are about 1.4 MB per batch, because
PDFium writes the whole of each font into the file (about 2.0 MB for the `full`/`summary` layouts,
which add a CID copy of Arial for the `×` — [Packing list](../labels/packing-list.md)) — accepted;
they print the same. Only the packing-list writer could be replaced by one that embeds only the
letters used if the size becomes a problem; nothing else would change.

## Diagram

```
┌──────────────────────── PC app (one .exe + pdfium.dll) ────────────────────────┐
│  Window: Svelte + TypeScript (screens of ui)                                   │
│        │  invoke("read_labels", …) / events ("read-progress", "save-progress") │
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

## Stack

| Need | Choice | Note |
|---|---|---|
| Language | Rust (stable, edition 2024; tested with 1.98) | One language for engine and window back end |
| Window | Tauri 2 (`tauri` 2.12) | Uses the Edge WebView2 that comes with Windows 11 |
| Screens | Svelte 5 + SvelteKit (static adapter, one page app) + TypeScript + Vite (`create-tauri-app`, template `svelte-ts`) | Plain CSS, no component library. Node 26, tested. Front-end build output `app/build/` (ignored by `app/.gitignore`) |
| PDF: read text with x/y, copy pages unchanged, write the packing list | `pdfium-render` 0.9.4 + `pdfium.dll` (Chrome's PDF engine; prebuilt by bblanchon/pdfium-binaries, tag `chromium/7881` = the crate's `pdfium_latest`) | `PdfPageText` gives each character with its box (runs are rebuilt, see [Text runs from PDFium](#text-runs-from-pdfium)); `copy_page_range_from_document` copies pages; `PdfFonts::load_true_type_from_file` + text objects write the packing list (whole fonts embedded, so the file is about 1.5 MB — accepted). Licences MIT/Apache (wrapper), BSD-3/Apache (PDFium) |
| TOML | `toml` (read) + `toml_edit` (write back keeping comments) | `categories.toml`, `couriers.toml` |
| JSON | `serde`, `serde_json` | `state.json`, test fixtures |
| Dates | `jiff` | `ship_by`, `save_time`, day folder |
| Hashes | `xxhash-rust` 0.8, feature `xxh3` | XXH3-64 for every hash the program makes: the fingerprint of a label file (the whole file; "used twice") and the logo fingerprints in `couriers.toml` (the logo image's encoded bytes; 0.44 µs for a 14 KB logo, measured, release build). One crate, no dependencies of its own. Not cryptographic, which does not matter here — the files are the platform's and the owner's own, not an attacker's; a file counts as the same when size and hash both match |
| Tests | `cargo test`; `svelte-check` for the screens | Fixtures in `testdata/`, also used by the tests of the Python tools; app-only cases go into separate fixture files |
| Build | `tools/make-portable.sh`: `npm run tauri build -- --no-bundle`, then the program folder | No installer; `Packing.exe` + `pdfium.dll` in `target/portable/Packing/` ([Program folder](program.md#program-folder-portable)) |

## Code layout

```
Cargo.toml                 workspace: crates/* and app/src-tauri
crates/
  engine/                  package packing-engine
    src/rules.rs           categories.toml, condition parser/evaluator/printer (rules file)
    src/rules_edit.rs      a category written back into categories.toml, comments kept
    src/text.rs            a page's text runs with x, y (input from packing-pdf)
    src/label.rs           Order ID search, label fields, the courier data (couriers.toml)
    src/couriers_edit.rs   a taught logo hash written back into couriers.toml, comments kept
    src/slip.rs            slip table by position
    src/orders.rs          pages → orders, contents key, display names
    src/plan.rs            plan, picks from what is left, plan edits, runs, numbering
    src/day.rs             state.json, duplicate guard, revert, amend
    src/names.rs           file and folder names
    couriers.toml          bundled with include_str!
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
tools/get-pdfium.sh        downloads the pinned pdfium.dll into vendor/ (git-ignored)
tools/make-portable.sh     builds the program folder target/portable/Packing/
```

`packing-engine` has no PDF dependency: it takes text runs and returns decisions, so all
reading and sorting rules are tested without PDF files. `packing-pdf` is the only code that
touches PDFium.

## Text runs from PDFium

The slip rules work on *runs* (a piece of text with its x, y). PDFium gives characters, so
`read.rs` rebuilds the runs:

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
