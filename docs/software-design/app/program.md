# Program

## Commands between window and Rust

| Command | Does | Returns |
|---|---|---|
| `dates_list()` | The panel: the dates of the data folder from the search index ([accounts](accounts.md#listing-and-opening-dates)) | Per date its orders, whether it is damaged, its folder; whether today has a group PDF not printed |
| `day_open(date)` | Open a date: the fold and the open-time checks ([journal](../data/journal.md#when-a-date-opens)) | Its batches, groups, sent and printed marks, reprints, findings; or the damage message |
| `check_files(paths)` | SHA-256 and page count of each chosen file; *used in batch n*; one file chosen twice is listed once | The file list |
| `read_labels(paths, sort_rule, redo)` | Read on the worker thread with the chosen sort rule; `redo` is the batch an amend redoes. Emits `read-progress {page, pages}` | Orders, warnings, unseen logos, the senders and files to ask an account for, repeats, the plan; or the stop |
| `sort_rules()` | New batch's **Sort rule**: the rules that are on, and the one to preselect ([New batch](../ui/new-batch.md#the-sort-rule)) | The names, the preselected one, the rule used last |
| `cancel_read()` | Stop reading | — |
| `name_logo(hash, answer)` | The logo prompt's answer: a courier, *Not a courier*, *I don't know* ([New batch](../ui/new-batch.md#the-logo-prompt)) | The recount |
| `answer_shop(answer)` | The shop prompt's answer: a sender's account or a new account, written into `packing-settings.toml`; or the account of a file's labels with no sender, for this read ([New batch](../ui/new-batch.md#the-shop-prompt)) | What is still to ask, or the recount |
| `resolve_repeats(choices)` | Apply the choice per group or per order ([duplicates](duplicates.md)) | One copy per order, the copies left out, the recount |
| `plan(edits)` | Skip, move, add a one-off group, rename a file; recount | Groups with counts, order contents, file names |
| `save_batch(save_anyway)` | Write, verify, commit, move ([04 Save](../workflow/04-save.md)). Emits `save-progress {stage, page, pages}` | The saved batch, or the verifier's findings (nothing saved until *Save anyway*) |
| `revert_last(day, batch)` / `start_amend(day, batch)` | Revert the newest batch / read its inputs again for an amend ([05 After the save](../workflow/05-after-save.md)); a `batch` that is no longer the newest is refused | The date / the amend's file list and read |
| `set_printed(day, batch, group, printed)` | The **Printed** tick: appends `printed_marked` | — |
| `print_output(day, file)` | **Print** a group PDF or a reprint: the output made ready, one not there written again first, like `open_output`; the preview is the front end's | Opened, written again, or not kept with the verifier's findings |
| `output_pdf(day, file)` | The bytes of an output that is there, for the preview | The PDF's bytes |
| `set_sent(day, batch, group, sent)` | Appends `print_sent`: `true` once a group PDF's preview is open, `false` from the **Sent** mark's ✕ | — |
| `restore_output(day, file)` | Write a changed output again ([reprint](reprint.md#restore)) | The date, or the verifier's findings |
| `open_output(day, file)` | **Open** a group PDF or summary PDF; one that is not there is written again first, like `restore_output` | Opened, written again, or not kept with the verifier's findings |
| `day_summary(day, account)` | Write the date's [day summary](../labels/day-summary.md), of all accounts (`account` none) or of one, on its own thread, then the window opens it | Its path |
| `account_summary(day, batch, account)` | Write a batch's [summary of one account](../labels/summary-pdf.md#one-account), check it, then the window opens it | Its path, or the verifier's findings |
| `accounts_list()` | The [Accounts](../ui/accounts.md) screen: each account with its shops, and per date its orders from the search index | The accounts |
| `account_new(name)` / `account_rename(name, new_name)` / `account_delete(name)` / `account_set_shops(name, shops)` | **+ New account**, **Rename**, **Delete account**, **Save** of its shops: `packing-settings.toml` read again and written under the lock ([accounts](accounts.md#names)) | The accounts as written, or why it is refused |
| `rules_list()` / `rules_load(rule)` / `rules_check(rule, groups)` / `rules_save(rule, groups)` | The Sort rules screen ([rules file](../rules-file/README.md)); a save rebuilds the open draft's plan | Rules on or off, the file and the couriers; groups; errors with line and column; live counts on the open draft |
| `rules_check_text(text)` / `rules_save_text(text)` | *Edit file as text* | The first check that fails, with its line and column |
| `rule_new(name, copy_from)` / `rule_rename(rule, name)` / `rule_enable(rule, enabled)` / `rule_delete(rule)` | **+ New rule**, **Rename**, the rule's switch (the draft's plan stays as it is), **Delete rule** | — |
| `filter_to_tree(text)` / `tree_to_filter(tree)` | The boxes editor: the engine's parser and printer, so the window has no second parser | Tree, or an error with line and column / the text |
| `search_orders(filter, range)` | Search the index ([reprint](reprint.md#search)) on the dates `from`–`to`, both included; no range searches every date | One row per order with its group file and its trace in pieces (a hash or form as code) |
| `reprint(orders)` | One PDF of the chosen orders' pages, verified, recorded ([reprint](reprint.md#reprint)) | The file, or the verifier's findings |
| `open_path(path)` / `show_in_folder(path)` | `explorer.exe "<path>"`, the path the only argument; a path that is not there is refused, since Explorer would open its home page | — |
| `settings_get()` / `settings_set(data_dir)` | Settings ([settings](../ui/settings.md)) | — |
| `about()` | The app version, the build time and the PDFium version | — |
| `data_changes()` | The window's look for other copies' writes, each time it gets the focus ([Several copies](#several-copies)) | Whether the open date and the panel changed |

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
3. **The data folder can be written**: its `.lock` file opens ([data](../data/README.md#one-writer)).
   Another copy of the app using the folder is no stop.
4. **The accounts** of `packing-settings.toml` ([accounts](accounts.md#the-accounts-file)); none yet is no
   stop.
5. **Today's date**, opened as in `day_open`; the search index of every date is built beside it.

## Several copies

Several copies of the app — on one PC, or on several PCs that share the data folder — may run at once
([D-APP-9](../decisions/app.md)).

- **A write** holds the data folder's lock ([data](../data/README.md#one-writer)) and waits while another
  copy holds it. It folds the date again first when its journal grew since it was read, so every decision
  is taken from the journal as it is on disk.
- **A save** whose orders another copy saved or reverted after the labels were read stops: *read the
  labels again*. Another copy's batch with other orders only moves this one to the next number.
- **Revert** and **Amend** name the batch the Day screen showed; one that is no longer the newest is refused.
- **The window looks** each time it gets the focus (`data_changes`): the Day screen reads its date again when the
  journal grew, or when it was opened while another copy wrote; the panel reads its list again when a
  date came or went or a journal grew, and the search index takes each such date in again.
- `packing-workflows.toml` and `packing-settings.toml` (with the accounts) are beside each program, not in
  the data folder: copies started from one program folder share them, and the last save of a sort rule
  wins. A write of the accounts reads the file again first, so another copy's change to another account is
  kept; a copy on another PC keeps its own accounts.
