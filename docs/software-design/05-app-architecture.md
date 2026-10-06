# 05 — Android App Architecture

Implements A1–A7 of [01-requirements.md](01-requirements.md). Screens are in
[06-app-ui.md](06-app-ui.md).

## Stack: Kotlin + Jetpack Compose

| Need | Library (latest stable at setup; record versions in development/01-setup.md) |
|---|---|
| Language, build | Kotlin, Gradle (Kotlin DSL), Android Gradle Plugin |
| UI, phone/tablet layouts | Jetpack Compose, Material 3, Material 3 adaptive (window size classes, list-detail) |
| Camera, lens switching | CameraX (`camera-camera2`, `camera-lifecycle`, `camera-view`) |
| Barcode reading (QR + Code 128, offline) | ML Kit Barcode Scanning, **bundled** model (`com.google.mlkit:barcode-scanning`), works without Google Play download |
| Database | Room (SQLite) with KSP |
| CSV | Apache Commons CSV |
| TOML read | tomlj (`org.tomlj:tomlj`, TOML 1.0). The writer is a small own function: the file shape is fixed (04-rules-file) |
| Session/settings files | `java.util.zip` + kotlinx.serialization (JSON) |
| Languages incl. "System" | String resources `values/` (English, the fallback) and `values-in/` (Indonesian; Android uses the code `in`), per-app language via AppCompat `setApplicationLocales` |
| Tests | JUnit on the JVM for `core`; Compose UI tests only where cheap |

Targets: `minSdk 26` (Android 8), latest `targetSdk`. Application id proposal
`com.wthe22.packingchecker`, app name "Packing Checker" (changeable until the first install).

Why Kotlin: lowest complexity for an Android-only app. Every need above is a first-party or
long-standing Android library, and the app reads camera frames directly, so the 2 s rule,
"two labels in view" and lens switching are plain code. (Flutter: extra framework, camera
details only as far as its plugin allows. Capacitor: camera behind a WebView, two runtimes.
Tauri 2: three toolchains, least proven for continuous mobile scanning.)

## Logic in two places

The CSV rules, batch/group numbering and the condition language exist in Python (script) and
Kotlin (app). Both test suites run the same committed fixtures in `testdata/` (made-up data):

- `testdata/orders-1.csv`, `orders-2.csv` — two exports of one day, same columns as the real
  export;
- `testdata/categories.toml`;
- `testdata/expected.json` — for each order: batch, category code, group number; per batch:
  order count, group count, removed count;
- `testdata/conditions.json` — condition text → expected parse result / error / value on a
  sample order.

A rule change is made in both languages and must pass the same fixtures.

## Modules (Gradle modules or packages)

```
core   pure Kotlin, no Android: CSV import, display names, signatures, condition language,
       categories.toml read/write, batch diff, group numbering, status from events,
       session/settings file format. JVM unit tests with testdata/ and (local) samples/.
data   Room database, repositories, export/import of zip files
scan   CameraX analyser → ML Kit → de-duplication (2 s rule) → lookup → verdict
ui     Compose screens, navigation, strings (EN + ID)
```

## Database (Room / SQLite)

Global (survive session deletion):

| Table | Fields |
|---|---|
| `category` | id PK, position (int, order in file), code, name, when_text (nullable for the last) |
| `scan_filter` | id PK, name, when_text, create_time |
| `setting` | key PK, value (text). Keys: `language` (`system`/`en`/`in`), `delete_protect_until` (`17:00` or empty), `sound` (bool), `vibration` (bool), `active_filter_id` |

Per session:

| Table | Fields |
|---|---|
| `session` | id PK (UUID text), name, create_time |
| `batch` | id PK, session_id FK, number, import_time, source_file_name, order_count, group_count, removed_count, ignored_row_count |
| `order` | id PK, session_id FK, order_id, tracking_id, package_id, batch_id FK (batch where first seen), category_code, group_no, courier, delivery_option, channel, buyer_message, seller_note, rts_time, removed (bool), removed_batch_id FK (nullable). Unique (session_id, order_id); index (session_id, tracking_id) |
| `sku` | (session_id, sku_id) PK, product_name, display_name, variation, seller_sku, product_category, update_time. Overwritten by each import |
| `order_line` | order FK, sku_id, quantity. PK (order, sku_id) |
| `mark_event` | id PK (UUID text), session_id FK, order FK, create_time, kind (`checked` / `wrong_packing` / `label_problem` / `pending` / `undo`), source (`camera` / `manual`), undoes_id FK (for `undo`, the mark it cancels) |
| `scan_event` | id PK, session_id FK, create_time, code_read, verdict (see scan pipeline), order FK (nullable), mark_event_id FK (nullable) |

- **Status** of an order = `removed` if removed; else the `kind` of its newest mark that has no
  `undo` pointing at it; else `unchecked`.
- Undoing a mark = inserting an `undo` event. Undoing an `undo` is not offered.
- `category_code` and `group_no` are fixed at import of the order's batch; changing categories
  later does not renumber old batches (the paper is already printed).
- Names live only in `sku`, so a renamed SKU shows its newest name everywhere.

## Import (A1)

1. Parse the CSV in `core` with the shared rules (01-requirements). Parse errors (missing
   column, unreadable file) → a clear message, nothing changes.
2. Compute, without writing: new orders (batch N) with category and group numbers; orders now
   removed; removed orders that came back (become active again); changed orders (different
   SKU IDs, quantities or tracking ID → warning); ignored row count.
3. Show the summary screen. **Apply import** writes everything in one transaction.
   **Cancel** discards it.

## Scan pipeline (A2)

1. CameraX `ImageAnalysis` (keep-only-latest) → ML Kit, formats QR + Code 128 (+ Code 39 /
   EAN as fallback) → list of code values in the frame.
2. A label holds the same tracking ID in its QR code and its barcode: identical values in one
   frame count once.
3. **2 s rule**: a map `code → last_seen_time`. A value seen less than 2 s ago only refreshes
   its time and is dropped. So a code counts again only after 2 s out of view.
4. Several **different** known orders in one frame → verdict `two_labels`, nothing marked.
5. Look up the value as tracking ID, then as Order ID, in the open session. Verdict:

| Verdict | Condition | Effect |
|---|---|---|
| `not_label_code` | Value fits no tracking-ID or Order-ID pattern | none |
| `unknown_order` | Valid-looking ID not in this session | none |
| `removed` | Order is removed | none |
| `filter_rejected` | Active filter is false for the order | none |
| `already_checked` | Status `checked` | none (duplicate warning) |
| `has_problem` | Status wrong packing / label problem / pending | dialog Process / Cancel; scanning paused while open. Process → as `checked` |
| `checked` | Otherwise | insert `checked` mark (source camera); result card with change buttons |

6. Insert `scan_event`; play the verdict's sound and vibration; show the message.
7. The change buttons insert a new mark (`wrong_packing`, `label_problem`, `pending`) for the
   order on the card; they apply until the next scan replaces the card.

Tracking-ID patterns (for `not_label_code`): `JY` + 10 digits; 12 digits; `TK` + 11 letters
or digits; Order ID 18 digits starting with 5. Unknown patterns that *are* in the session still
match (the lookup goes first; the pattern only picks the message).

## Session file (A6)

`<session name>.zip` containing:

- `manifest.json`: `{"format": "packing-session", "format_version": 1, "app_version": "…",
  "export_time": "…"}`
- `session.json`: the session with its batches, orders, order lines, skus, mark events and
  scan events, as plain arrays with the database field names.

No PDFs. Import checks `format` and `format_version`. If a session with the same id exists,
the user picks **Replace** or **Keep both** (the import gets a new id and "(copy)" in its name).

## Settings file (A7)

`packing-settings.zip` containing `manifest.json` (`"format": "packing-settings"`),
`categories.toml`, and `settings.json` (scan filters and the `setting` table).
`categories.toml` can also be imported/exported on its own for editing on the PC.

## Complexity

| Part | Size | Why |
|---|---|---|
| PC script | S–M | CSV, page copying, table PDF (prototype exists), condition parser |
| App `core` (import, diff, conditions, TOML) | M | Second implementation of the script's rules, tested on the same fixtures |
| Condition editor UI | M | AND/OR boxes on a phone screen |
| Database, sessions, zip export/import | S–M | |
| Scan screen | M–L | Speed, 2 s rule, messages, sounds; decides whether the app is useful |
| Order list, problems page, history, undo | M | |
| Four layouts, two languages | M | |

Overall: script ≈ a few days; app ≈ 2–4 weeks of focused work for one developer, most of it in
the scan screen, the condition editor and the layouts. Order of magnitude only.
