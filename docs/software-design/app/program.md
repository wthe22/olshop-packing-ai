# Program

## Commands between window and Rust

| Command | Does | Returns |
|---|---|---|
| `workspaces_list()` | The workspaces of the data folder, and the open one's dates from the search index ([workspaces](workspaces.md)) | Names; per date its orders, whether it is damaged, its folder |
| `workspace_new(name)` | **+ New workspace**: make the workspace's folder and open it on today's date | The name, or why it is refused |
| `workspace_open(name)` | Open another workspace on today's date; its search index is built beside it | — |
| `day_open(workspace, date)` | Open a date: the fold and the open-time checks ([journal](../data/journal.md#when-a-date-opens)) | Its batches, groups, printed marks, reprints, findings; or the damage message |
| `check_files(paths)` | SHA-256 and page count of each chosen file; *used in batch n*; one file chosen twice is listed once | The file list |
| `read_labels(paths, sort_rule, redo)` | Read on the worker thread with the chosen sort rule; `redo` is the batch an amend redoes. Emits `read-progress {page, pages}` | Orders, warnings, unseen logos, repeats, the plan; or the stop |
| `cancel_read()` | Stop reading | — |
| `name_logo(hash, answer)` | The logo prompt's answer: a courier, *Not a courier*, *I don't know* ([New batch](../ui/new-batch.md#the-logo-prompt)) | The recount |
| `resolve_repeats(choices)` | Apply the choice per group or per order ([duplicates](duplicates.md)) | One copy per order, the copies left out, the recount |
| `plan(edits)` | Skip, move, add a one-off group, rename a file; recount | Groups with counts, order contents, file names |
| `save_batch(save_anyway)` | Write, verify, commit, move ([04 Save](../workflow/04-save.md)). Emits `save-progress {stage, page, pages}` | The saved batch, or the verifier's findings (nothing saved until *Save anyway*) |
| `revert_last(day)` / `start_amend(day)` | Revert the newest batch / read its inputs again for an amend ([05 After the save](../workflow/05-after-save.md)) | The date / the amend's file list and read |
| `set_printed(day, batch, group, printed)` | The **Printed** tick: appends `printed_marked` | — |
| `restore_output(day, file)` | Write a changed output again ([reprint](reprint.md#restore)) | The date, or the verifier's findings |
| `open_output(day, file)` | **Open** a group PDF or summary PDF; one that is not there is written again first, like `restore_output` | Opened, written again, or not kept with the verifier's findings |
| `day_summary(day)` | Write the date's [day summary](../labels/day-summary.md) on its own thread, then the window opens it | Its path |
| `rules_list()` / `rules_load(rule)` / `rules_check(rule, groups)` / `rules_save(rule, groups)` | The Sort rules screen ([rules file](../rules-file/README.md)); a save rebuilds the open draft's plan | Rules, the file and the couriers; groups; errors with line and column; live counts on the open draft |
| `rules_check_text(text)` / `rules_save_text(text)` | *Edit file as text* | The first check that fails, with its line and column |
| `rule_new(name, copy_from)` / `rule_rename(rule, name)` / `rule_delete(rule)` | **+ New rule**, **Rename…**, **Delete rule** | — |
| `filter_to_tree(text)` / `tree_to_filter(tree)` | The boxes editor: the engine's parser and printer, so the window has no second parser | Tree, or an error with line and column / the text |
| `search_orders(filter, range)` | Search the index ([reprint](reprint.md#search)) on the dates `from`–`to`, both included; no range searches every date | One row per order with its group file and its trace in pieces (a hash or form as code) |
| `reprint(orders)` | One PDF of the chosen orders' pages, verified, recorded ([reprint](reprint.md#reprint)) | The file, or the verifier's findings |
| `open_path(path)` / `show_in_folder(path)` | `explorer.exe "<path>"`, the path the only argument; a path that is not there is refused, since Explorer would open its home page | — |
| `settings_get()` / `settings_set(data_dir)` | Settings ([settings](../ui/settings.md)) | — |
| `about()` | The app version and the PDFium version | — |

`explorer.exe /select,<path>` is not used: Explorer parses `/select,` itself and does not understand the
quotes Rust puts around an argument with a space, so it opens *Documents*. A bare path opens a folder as a
folder and a file in its default program, which is all the two buttons need.

## Program folder

There is no installer. The program is one folder, put anywhere the owner can write (e.g. `D:\Packing\`,
not `Program Files`) and started from a shortcut:

```
Packing\
  Packing.exe
  pdfium.dll
  packing-settings.toml     data_dir = "packing-data"
  packing-workflows.toml
  packing-data\             the data folder (the default)
```

- Copying the folder moves the program and, with a relative `data_dir`, all its data along.
- **New version**: replace `Packing.exe` and `pdfium.dll`; the two TOML files and the data stay. A version
  that writes another schema is a new major data version: older data is not read ([D-APP-3](../decisions/app.md)).
- WebView2 keeps its browser cache under `%LOCALAPPDATA%\com.wthe22.olshop-packing\`; it holds no data of
  the app.
- The app is in English; every screen text is in one file (`app/src/texts.ts`).

## Start

1. **The data folder.** Read `packing-settings.toml` beside the program. Missing, or its `data_dir`
   missing: the [Start](../ui/start.md) screen asks once and writes the file.
2. **The rules.** Read `packing-workflows.toml` beside the program; missing: write the bundled copy. A file
   that does not parse stops the app with its line and column; a filter error opens the Sort rules screen
   on that rule.
3. **The lock** on the data folder ([data](../data/README.md#one-writer)). Held by another copy: say which
   one, and offer **Try again**.
4. **The workspace** whose journal was written last; none yet: ask for the first workspace's name
   ([workspaces](workspaces.md)).
5. **Today's date** of that workspace, opened as in `day_open`; the search index is built beside it.
