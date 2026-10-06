# 04 — Android App Architecture

Proposal. Nothing here is decided until the owner chooses (D1 for the stack).

## What the platform must do

| Need | Detail |
|---|---|
| Continuous camera scanning | QR + Code 128, offline, fast repeat, camera switching |
| Read CSV or batch file | Per D2 |
| Local database | A few thousand orders per session |
| ZIP export/import | Session and settings files |
| Adaptive layout | Phone and tablet, portrait and landscape |
| Languages | Indonesian, English, System (English fallback) |

No PDF work on the device.

## Stack options

| | A. Kotlin + Jetpack Compose | B. Capacitor + Svelte/TypeScript | C. Tauri 2 + Svelte + Rust | D. Flutter |
|---|---|---|---|---|
| Scanning | CameraX + ML Kit, own analyser: full control of the 2 s ignore rule, multiple codes per frame, lens switching | ML Kit plugin (`@capacitor-mlkit/barcode-scanning`), continuous listener, lens switch | Official barcode plugin; one result per call — continuous scanning to verify | `mobile_scanner` (ML Kit), continuous, `switchCamera()` |
| Database | Room (SQLite) | SQLite plugin | SQLite (`rusqlite`) | `drift` (SQLite) |
| Adaptive layout | Window size classes, list-detail panes | CSS | CSS | LayoutBuilder |
| Languages | Android resources (system locale + per-app language built in) | i18n library | i18n library | `intl` / ARB files |
| Main risk | Android only (fine here) | Camera view behind a WebView | Mobile support least proven | Plugin dependence for camera details |

Notes:
- With no PDF work, every option can do the job. The scan screen decides: A gives the most
  direct control of the camera stream; D is close.
- C would share Rust with the shop's other project, but no code is shared here (the script is
  Python), so that advantage is small.
- Phase 0 measures real scan speed on the owner's phone before the rest is built.

## Modules

```
core      import (CSV or batch file per D2), diff, display names, condition engine,
          status rules, session file format. Pure logic, unit-tested with samples/.
store     database, session and settings export/import
scan      camera frames → codes → 2 s ignore rule → lookup → verdict → mark
ui        screens (05-app-ui.md), strings in Indonesian and English
```

## Data model (SQLite)

Global:

| Table | Fields |
|---|---|
| `scan_filter` | id PK, name, condition (JSON, format of 03-pc-script), create_time |
| `category` | id PK, code, name, position, condition (JSON). Only if D2-a |
| `setting` | key PK, value |

Per session:

| Table | Fields |
|---|---|
| `session` | id PK (UUID), name, create_time |
| `batch` | id PK, session FK, number, import_time, source_file_name |
| `order` | id PK, session FK, order_id, tracking_id, package_id, batch FK (where first seen), courier, delivery_option, channel, buyer_message, seller_note, rts_time, removed (bool), removed_batch FK, changed_note, category_code, group_code. Unique (session, order_id); index on tracking_id |
| `sku` | session FK + sku_id PK, product_name, display_name, variation, seller_sku, product_category, update_time. Overwritten by every import |
| `order_line` | order FK, sku_id, quantity |
| `mark_event` | id PK (UUID), session FK, order FK, create_time, kind (`checked` / `wrong_packing` / `label_problem` / `pending` / `undo`), source (`camera` / `manual`), undoes FK (for `undo`) |
| `scan_event` | id PK, session FK, create_time, code_read, verdict, order FK (nullable), mark_event FK (nullable) |

- Status = `removed` if removed; else the kind of the latest mark not undone; else `unchecked`.
- Names live only in `sku`, so a renamed SKU shows its newest name everywhere.
- `scan_event` keeps every scan with its verdict, so a rejected scan can be explained later and
  a mark made by a scan can be undone from the scan list.

## Import (app side)

1. Read the file (D2). Rows of other statuses are ignored and counted.
2. For each order in the file:
   - not in session → **new**, batch N;
   - in session, different SKU IDs, quantities or tracking ID → **changed** (D8);
   - in session and removed → back to active (**returned**).
3. Every active order of the session absent from the file → **removed**.
4. Update `sku` names from the file.
5. Show the summary; save on confirm.

## Scan pipeline

1. Camera frame → all barcodes in the frame.
2. Each code: if it was seen less than 2 s ago, refresh its last-seen time and stop.
3. Two different known orders in one frame → "two labels in view", nothing marked.
4. Lookup by tracking ID, then Order ID → verdict (see 05-app-ui scan messages):
   unknown code · not in this session · removed · filter rejected · already checked ·
   ok → mark (Fast) or open card (Inspect) (D4).
5. Write `scan_event`; play the sound/vibration of the verdict; show the message.

## Session file

`<session name>.zip`: `manifest.json` (format version, app version, export_time) +
`session.json` (session, batches, orders, skus, lines, mark and scan events). No PDFs. Import:
if a session with the same id exists, the user picks replace or keep both.

## Complexity

| Part | Size | Why |
|---|---|---|
| PC script | S–M | CSV, page copying and a table PDF; rules engine shared in design |
| App import + diff | S | Known format |
| Condition engine + filter editor UI | M | AND/OR tree editing on a phone needs care |
| Database, sessions, zip export/import | S–M | |
| Scan screen | M–L | Speed, the 2 s rule, clear messages, sounds; decides usefulness |
| Order list, problems page, history, undo | M | |
| Layouts ×4, two languages | M | |

Overall: script ≈ a few days; app = medium, roughly 2–4 weeks of focused work for one
developer, most of it in the scan screen, the filter editor and the layouts. Order of
magnitude only.

## Build phases

0. **Spike**: camera scanning of printed sample labels on the owner's phone (QR vs 1D,
   distance, light, switching lens); scans per minute. Decide D1.
1. **PC script**: usable on its own from day one.
2. **App core**: sessions, import, order list, detail, marks, undo.
3. **Scan**: filters, modes, messages, scan history.
4. **Files and settings**: session/settings zip, languages, delete protection.
5. **Polish**: tablet and landscape layouts, problems page.
