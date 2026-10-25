# Architecture

## Diagram

```
┌───────────────────────── PC app (Packing.exe + pdfium.dll) ─────────────────────────┐
│  Window: Svelte + TypeScript (the screens of ui/)                                   │
│        │  invoke("read_labels", …) / events ("read-progress", "save-progress")      │
│        ▼                                                                            │
│  packing-app (Tauri 2): commands, the open draft, the worker thread                  │
│        ├──► packing-engine  rules, filters, pages → copies → orders, repeats, plan,  │
│        │                    order contents, the record of a batch. No PDF, no file   │
│        ├──► packing-pdf     PDFium: page text and images, copy pages, summary PDF   │
│        ├──► packing-store   lock, journal, fold, input/ and output/, views, index   │
│        └──► packing-verify  lopdf: page digests and the checks of a record          │
└─────────────────────────────────────────────────────────────────────────────────────┘
packing-cli (developer tool): the same crates from the command line, for tests and sample runs
```

## Who may use whom

What checks an output is never the code that made it: `packing-verify` shares no code and no PDF library
with the crates that read and write ([verification](verification.md)).

| Crate | Uses | Never uses |
|---|---|---|
| `packing-engine` | `regex`, `jiff`, `serde`, `toml` | PDFium, files, the other crates |
| `packing-pdf` | `packing-engine` (its types), `pdfium-render` | `packing-store`, `packing-verify` |
| `packing-store` | `packing-engine` (its types), `serde_json`, `sha2`, `xxhash-rust` | PDFium, `packing-pdf`, `packing-verify` |
| `packing-verify` | `lopdf`, `serde_json`, `sha2`, `xxhash-rust` | `packing-engine`, `packing-pdf`, `packing-store`, PDFium |
| `packing-app`, `packing-cli` | all of the above | — |

`packing-verify` reads the journal record as JSON into types of its own, so a field the writer gets wrong
is read as written, not as meant.

## Stack

| Need | Choice | Note |
|---|---|---|
| Language | Rust (stable, edition 2024) | One language for every crate and the window's back end |
| Window | Tauri 2 | Uses the Edge WebView2 that comes with Windows 11 |
| Screens | Svelte 5 + SvelteKit (static adapter) + TypeScript + Vite | Plain CSS, no component library |
| Read, copy, write PDFs | `pdfium-render` 0.9.4 + `pdfium.dll` (bblanchon/pdfium-binaries, `chromium/7881`) | Text with positions, image bytes, page copy, the summary PDF ([D-APP-1](../decisions/app.md)) |
| Verify PDFs | `lopdf` | The verifier's only PDF library: page objects, content streams, the summary's text |
| TOML | `toml` (read) + `toml_edit` (write back keeping comments) | `packing-workflows.toml`, `packing-settings.toml`, the views |
| JSON | `serde`, `serde_json` | The journal |
| Dates | `jiff` | Dates of the labels, `*_time` |
| Hashes | `xxhash-rust` (XXH3-64), `sha2` (SHA-256) | XXH3-64: logo and shape hashes, page fingerprints and digests, the journal's seals. SHA-256: input and output files, so a file's name is its content |
| Tests | `cargo test`; `svelte-check` and `node --test` for the screens | Fixtures written by the Rust fixture writer ([testing](../../development/testing.md)) |
| Build | `tools/make-portable.sh` | No installer; the program folder ([program](program.md#program-folder)) |

## Threads

PDFium is not thread-safe: all PDFium work runs on one worker thread that owns the library, and the
commands send it jobs. The open draft (orders, the opened input documents) stays on that thread until it
is saved or cancelled. The verifier and the search index run on their own threads; the journal is written
only by the command that holds the draft, one commit at a time.

## Text runs from PDFium

The slip rules work on *runs* (a piece of text with its x, y). PDFium gives characters and text objects,
so `packing-pdf` rebuilds the runs:

- **One character per text object** (the downloaded labels, made by wkhtmltopdf, and the test fixtures):
  drop the space and line-break characters PDFium adds (their boxes are empty), sort by line then x, start
  a new run on a new line, a change of font name, or a gap over 5 pt; put a space for a gap over 1 pt. The
  font name matters: `Qty Total:` (bold) and its value are as close as two words.
- **Several characters per text object** (the Shopee-style label's edge texts make its page one of
  these): one run per text object, characters in their order; two objects on one line whose gap is not a
  space (the next starts left of, or within 1 pt of, the previous one's right edge) are one run, so a word
  the label splits at a font change (`V` + `ariasi`) is read whole.
- **Hyphen at a line end**: PDFium reports it as U+0002; it is read as `-`, so the wrap rule (no space
  after a trailing `-`) applies.
