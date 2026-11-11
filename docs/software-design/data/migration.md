# From a v0.4 data folder

v0.4 kept each workspace's dates in a folder of its own, `{data_dir}/{workspace}/{date}/`, with its shops in
`{workspace}/settings.toml`. The program does not read them ([accounts](../app/accounts.md#data-of-older-versions)).
`tools/migrate-v0.4.py`, a script apart from the program, copies them once into the layout of
[the data folder](README.md#the-data-folder) ([D-APP-27](../decisions/accounts.md)). It is never built
into `Packing.exe`.

```
py tools\migrate-v0.4.py D:\Packing          the program folder: packing-settings.toml names the data folder
```

It needs Python 3.11 or later and the `xxhash` package (`py -m pip install xxhash`) for the journal's seals.

## What it does

1. **Find** the v0.4 workspaces: each folder of the data folder that is not named as a date and holds a
   `settings.toml` or a date folder with a `journal.jsonl`.
2. **Check first, write nothing** when one of these holds, naming it: a date folder with an
   `output/.partial/` (open that date in v0.4 once), a journal that is damaged (as the
   [fold](journal.md#a-torn-write-and-damage) finds it; a torn last line is left out, as the fold does), a
   date that already has a folder at the top of the data folder, an account name that `packing-settings.toml`
   already has, or a shop listed by two workspaces.
3. **Accounts**: one per workspace, named as its folder, with the shops of its `settings.toml`, appended to
   `packing-settings.toml` as `[[accounts]]` tables; the rest of the file is left as it is.
4. **Each date** found in any workspace becomes one `{data_dir}/{date}/`:
   - **Batches**: the live batches of every workspace's journal of that date, each as its newest
     `batch_saved`, numbered 1, 2, 3, … in the order of their first save time.
   - **Inputs**: the files of `input/` copied (one file of the same SHA-256 once); `PdfID` numbered again in
     order of first use, and every `pdf` of a record changed to match.
   - **Orders**: each `account` is the workspace's name; `shop` is none, since v0.4 never kept the sender.
   - **Outputs**: each group PDF copied under its new batch number (`3-A Sepatu x88.pdf`), its `file` in the
     record changed to match, with the size and SHA-256 of its newest `batch_saved` or `output_restored`. The
     summary PDFs are not copied: they print the old number, and **Open** writes each again from its record
     ([reprint › Restore](../app/reprint.md#restore)).
   - **Marks**: a `print_sent` and a `printed_marked` for each group PDF whose mark is set.
   - **Reprints**: each `reprint_made`, in time order across the workspaces, numbered again per date; its file
     copied under the new number; each order's `batch` changed to its new number (an order whose batch was
     reverted keeps the number it had).
   - **The journal**: `started` (schema `2.0`, the date), then the batches, the marks and the reprints, each
     with the next `seq`, its own `time` and `app`, as one compact JSON line sealed with XXH3-64
     ([journal](journal.md#a-line)). The views are written by the program when the date is first opened.
5. **Report** each date with its batches, orders and accounts, and what was left out.

## What it keeps

- The v0.4 folders are only read: nothing in them is changed, moved or deleted, so they stay a backup and the
  v0.4 program still opens them.
- A batch's orders, groups, contents, left-out copies, inputs and warnings are those of its newest record.
  What the journals held before it (a reverted batch, an amend's earlier record, a mark taken back) is not
  copied.
- The printed labels keep the batch numbers they were printed with; a summary written again shows the new one.
