# 04 — Android App Architecture

## Stack: Kotlin + Jetpack Compose

| Need | Library |
|---|---|
| UI, adaptive phone/tablet layouts | Jetpack Compose, Material 3, Material 3 adaptive (list-detail) |
| Camera, lens switching | CameraX |
| Barcode reading (QR + Code 128, offline) | ML Kit Barcode Scanning, bundled model |
| Database | Room (SQLite) |
| CSV | Apache Commons CSV |
| Session/settings files | `java.util.zip` + kotlinx.serialization (JSON) |
| Languages incl. "System" | Android string resources + per-app language (AppCompat) |
| Build | Gradle; core logic tested as plain JVM unit tests against `samples/` |

### Complexity of the stack options for this app

| Option | Complexity | Why |
|---|---|---|
| **Kotlin + Compose** (chosen) | Lowest | One language; every need above is a first-party or long-standing Android library; the app reads camera frames directly, so the 2 s rule, two-labels-in-view and lens switching are plain code |
| Flutter | Medium | Extra framework and language on top of Android; camera details only as far as the scanner plugin exposes them |
| Capacitor + Svelte | Medium–high | Web UI plus native plugins; camera preview sits behind a transparent WebView; two runtimes to debug |
| Tauri 2 + Rust | High | Rust + web + Android toolchains; mobile scanning plugin least proven for continuous scanning |

## Logic in two places

The import, grouping and condition rules exist in Python (script) and Kotlin (app), because
both read the CSV. Both are tested against the same files: `samples/` plus an expected-result
file (batch of each order, group number, category). A change to the rules is made in both and
must pass the same expected results.

## Modules

```
core      CSV import, diff, display names, condition engine, pack-group numbering,
          status rules, session file format. Pure Kotlin, JVM unit tests.
store     Room database, session and settings export/import
scan      CameraX frames → ML Kit codes → 2 s rule → lookup → verdict → mark
ui        Compose screens (05-app-ui.md), strings in Indonesian and English
```

## Data model (SQLite)

Global:

| Table | Fields |
|---|---|
| `category` | id PK, code, name, position, condition (JSON, format of 03-pc-script) |
| `scan_filter` | id PK, name, condition (JSON), create_time |
| `setting` | key PK, value |

Per session:

| Table | Fields |
|---|---|
| `session` | id PK (UUID), name, create_time |
| `batch` | id PK, session FK, number, import_time, source_file_name, order_count, group_count |
| `order` | id PK, session FK, order_id, tracking_id, package_id, batch FK (where first seen), group_no, category_code, courier, delivery_option, channel, buyer_message, seller_note, rts_time, removed (bool), removed_batch FK. Unique (session, order_id); index on tracking_id |
| `sku` | session FK + sku_id PK, product_name, display_name, variation, seller_sku, product_category, update_time. Overwritten by every import |
| `order_line` | order FK, sku_id, quantity |
| `mark_event` | id PK (UUID), session FK, order FK, create_time, kind (`checked` / `wrong_packing` / `label_problem` / `pending` / `undo`), source (`camera` / `manual`), undoes FK (for `undo`) |
| `scan_event` | id PK, session FK, create_time, code_read, verdict, order FK (nullable), mark_event FK (nullable) |

- Status = `removed` if removed; else the kind of the latest mark not undone; else `unchecked`.
- Names live only in `sku`, so a renamed SKU shows its newest name everywhere.
- `scan_event` keeps every scan with its verdict, so any result can be explained later and a mark
  made by a scan can be undone from the scan history.

## Import

1. Read the CSV; keep `Perlu dikirim` / `Menunggu pengambilan` rows; group by Order ID.
2. New orders (not in the session) → batch N; category; pack groups numbered as in
   01 "Same numbers on PC and phone".
3. Active orders of the session absent from the CSV → removed. A removed order present again →
   active.
4. Orders whose SKU IDs, quantities or tracking ID differ → warning list; status unchanged; the
   newest values are stored.
5. Update `sku` names.
6. Show the summary (new orders, groups, removed, ignored rows, warnings) with
   **Apply import** / **Cancel**. One database transaction on Apply; nothing on Cancel.

## Scan pipeline

1. Camera frame → all barcodes in the frame.
2. Each code: if it was seen less than 2 s ago, refresh its last-seen time and stop.
3. Two different known orders in one frame → "two labels in view", nothing marked.
4. Lookup by tracking ID, then Order ID, then:
   - unknown text → *not a label code*; unknown ID → *not in this session*;
   - removed → *no longer to send*;
   - filter fails → *not for this stack* (with the failing condition and the order's value);
   - already `checked` → *already checked* (duplicate warning);
   - marked wrong packing / label problem / pending → warning with **Process** / **Cancel**;
   - otherwise → add `checked` mark; show the result with the change buttons.
5. Write `scan_event`; play the sound/vibration of the verdict.

## Session file

`<session name>.zip`: `manifest.json` (format version, app version, export_time) +
`session.json` (session, batches, orders, skus, lines, mark and scan events). No PDFs. Import:
if a session with the same id exists, the user picks replace or keep both.

## Complexity

| Part | Size | Why |
|---|---|---|
| PC script | S–M | CSV, page copying, a table PDF (prototype done) |
| App import + diff | S | Known format; same rules as the script |
| Condition engine + editor UI | M | AND/OR tree editing on a phone needs care |
| Database, sessions, zip export/import | S–M | |
| Scan screen | M–L | Speed, the 2 s rule, clear messages, sounds; decides usefulness |
| Order list, problems page, history, undo | M | |
| Layouts ×4, two languages | M | |

Overall: script ≈ a few days; app = medium, roughly 2–4 weeks of focused work for one
developer, most of it in the scan screen, the condition editor and the layouts. Order of
magnitude only.

## Build phases

0. **Spike**: camera scanning of printed sample labels on the owner's phone (QR vs 1D,
   distance, light, lens switching); scans per minute.
1. **PC script**: usable on its own from day one; produces the expected-result file.
2. **App core**: sessions, import, order list, detail, marks, undo.
3. **Scan**: filters, messages, scan history.
4. **Files and settings**: session/settings zip, categories, languages, delete protection.
5. **Polish**: tablet and landscape layouts, problems page.
