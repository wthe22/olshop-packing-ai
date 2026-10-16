# Program

## Commands between window and Rust

| Command | Does | Returns |
|---|---|---|
| `day_overview(day)` | Read `state.json` of the day | Batches, saved PDFs, totals |
| `check_files(paths)` | Page count and fingerprint of each chosen file | Name, size, time, page count, "used in batch n" |
| `read_labels(paths, redo)` | Start reading on a worker thread; `redo` is the batch being redone when the read is the amendment's ([New batch](../ui/new-batch.md#the-amends-selection-page)), nothing for a new batch; emits `read-progress {page, pages}` | Orders count, duplicates, warnings, the plan, or the stop error |
| `cancel_read()` | Stop reading | — |
| `plan(draft, edits)` | Apply the plan edits (skip, move, add condition, rename) and recount | Picks with counts, runs (contents × orders), suggested names |
| `save_batch(draft, plan, packing, save_anyway)` | Steps of [Save](batch.md) and the [check before saving](saved-pdf-check.md); emits `save-progress {stage, page, pages}` | Written files and summary, or the check's findings (nothing written until *Save anyway*) |
| `revert_last(day)` / `start_amend(day)` | Revert / read the last batch's own label files again for an amend ([New batch](../ui/new-batch.md#the-amends-selection-page)) | New overview / the batch being redone, its file list (the selection page's starting list) and the read, so the Plan screen opens at once ([Plan](../ui/plan.md)); emits `read-progress {page, pages}` |
| `rules_load()` / `rules_check(text or entries, draft?)` / `rules_save(…)` | Categories editor | Entries, errors with line/col, live counts on the open draft |
| `condition_to_tree(text)` / `tree_to_condition(tree)` | Boxes editor: condition text → boxes, boxes → text (the engine's parser and printer, so the window has no second parser) | Tree of groups and rows, or an error with line/col / the text |
| `search_orders(conditions, scope)` | Search the saved orders matching the condition boxes; `scope` is `day` (*This day*, the day in the title bar) or `all` (*All days*) | One row per order (Order ID, tracking ID, courier, items, batch, saved PDF name and path) and the order and batch counts |
| `open_path(path)` / `show_in_folder(path)` | Open a PDF in the default viewer / a folder in Explorer — one call, `explorer.exe "<path>"` with the path as the only argument | — |
| `settings_get()` / `settings_set(…)` | Settings | — |

Both shell commands hand Explorer the path as its **only** argument. `explorer.exe /select,<path>` is
not usable: Explorer parses `/select,` itself and does not understand the quotes Rust puts around an
argument that holds a space, so it reads no target and opens *Documents* (or *Home* when the target is
empty) — and the batch folder path always holds a space (`…\labels\2026-10-11\batch 1`). A bare path
opens a folder as a folder and a file in its default program, which is all the two buttons need.

PDFium is not thread-safe: all PDF work runs on one worker thread that owns the PDFium
library; commands send it jobs. The open draft (orders, the opened label documents) is kept
by that thread until it is saved, cancelled, or the app closes.

## Settings

`settings.json` in the data folder:

| Setting | Default |
|---|---|
| Packing-list scope / layouts | `whole batch` / `pick` |

Where the data folder is, is not a stored setting (next section). The app is in English; every
screen text is in one file (`app/src/texts.ts`) so another language can be added later.

## Program folder (portable)

There is no installer. The program is one folder, put anywhere the owner can write (e.g.
`D:\Packing\`, not `Program Files`) and started from a shortcut:

```
Packing\
  Packing.exe
  pdfium.dll
  categories.toml   ┐
  couriers.toml     ├ the data folder (default: the program folder itself)
  settings.json     │
  labels\           ┘
```

- Copying the folder to another PC moves the program and all its data along.
- **New version**: replace `Packing.exe` and `pdfium.dll`; the data files stay.
- **Finding the data folder at start**: the app looks for `couriers.toml` in its own folder.
  Found → that folder is the data folder. Not found → the app asks, at every start until it is
  found: **Use this folder** (writes the bundled `couriers.toml` next to the program; no
  `categories.toml` is written, so the picks start as one catch-all entry) or
  **Choose a folder…** (used until the app closes; the next start asks again). See
  [Start](../ui/start.md).
- Settings › Data folder › **Change…** switches to another folder until the app closes.
- WebView2 keeps its browser cache under `%LOCALAPPDATA%\com.wthe22.olshop-packing\`; it holds
  no data of the app and is not needed on another PC.
