# Steps 1–2: start, and open a date

## 1. Start

1. **Data folder.** Read `packing-app.toml` beside the program; `data_dir` is absolute or relative to
   the program's folder. Missing: the [Start](../ui/start.md) screen asks once (**Use `Packing\packing-data`** or
   **Choose a folder**) and writes the file.
2. **Migration.** Find, reading only, what the migration would change: date folders of v0.7 (with
   `input/` or `output/`, or a backup whose conversion stopped, and no Update that listed it flat), dates whose files are of an older
   `schema` or still `session.toml`, dates with `journal.jsonl`, and live batches saved before the receiver's area, the weight or the shop was read whose label PDFs are in the date
   folder (on a date of v0.7, will be once it is flat); a damaged `journal.jsonl` is passed over. If there is any, the
   [Start](../ui/start.md#update-the-data-folder) screen asks before any date opens: **Update** or **Open
   read-only** ([D-APP-33](../decisions/app.md)). Choosing another data folder asks the same.
   - **Update**, holding the data folder's lock, date by date, oldest first: the files it changes are
     copied into `update v{version}/` of the date ([D-APP-36](../decisions/versions.md)); a date of v0.7 is
     made flat first ([D-APP-32](../decisions/app.md)): each file moves up under its new name, checked by its
     hash, each move listed in `update.toml` before it is made; a date with `journal.jsonl` has its three files written from it, and the journal moves into the update
     folder; then the area, weight and shop of the orders without them are read again from the first page of
     their copies in the batch's stored label PDFs, straight into `orders.csv`. A batch
     whose label PDF has changed is skipped and named; the next start asks again.
   - **Open read-only**: the steps below run without writing. No lock is taken, a missing rules file is read
     from its bundled copy, and every date opens for viewing only.
3. **Writable.** Open `{data_dir}/.lock`: a folder that cannot be written stops here. Another copy of the
   app using the folder is no stop; the lock is taken only for a write ([data](../data/README.md#one-writer)).
4. **Rules.** Read `parser.toml` and `config.toml` of the data folder; a missing one is written from the
   bundled copy. Run their checks ([rules files](../rules-file/README.md#loading)).
5. **Older data.** If the data folder holds data of v0.1 or v0.4, the Day screen will say so; nothing of it
   is read, changed or deleted ([accounts](../app/accounts.md#data-of-older-versions)).
6. **Accounts.** Read `accounts` of `config.toml` ([accounts](../app/accounts.md#the-accounts-file));
   none is no stop: the first read asks.
7. **Search index.** Read the `orders.csv` and `order-contents.toml` of every date on its own thread ([reprint](../app/reprint.md#the-search-index)).

**Stops:** `parser.toml` or `config.toml` that does not parse (its name, line and column); a data folder that cannot be
written.

**Check:** is today the right date to open on?

## 2. Open a date

Today's date (the PC's date) at start; another from the [panel](../ui/panel.md). In this order
([save › When a date opens](../data/save.md#when-a-date-opens)):

1. **`.partial/`** settled: back when it holds `day.toml`, else forward.
2. **Hashes** of `orders.csv` and `order-contents.toml` compared with `day.toml`; a changed or missing one
   is named ([D-APP-40](../decisions/date-files.md)).
3. **Read** `day.toml`, `orders.csv` and `order-contents.toml`. The result: the live batches with their
   orders, groups, files, Sent times and printed ticks; the inputs table; the reprints; the next batch number
   (highest live + 1) and the next `PdfID` (highest held + 1).
4. **Inputs**: SHA-256 of every input of a live batch, compared with `hash_sha256`.
5. **Outputs**: each output of a live batch exists; a missing one gets **Restore**.

A date folder with none of the three files is empty: nothing is read and no file is made until its first
commit.

Step 1 writes: it runs only while this copy holds the data folder's lock, taken if no other copy writes.
While another copy writes, the date opens for viewing only and opens again once it is done. An open date is
opened again whenever its `day.toml` changed: another copy committed to it
([program › Several copies](../app/program.md#several-copies)). In read-only mode step 1 never runs.

**Stops:** damage — a file that does not read, a `date` that is not the folder's, an order, group or content
that one file names and another lacks. The date opens for viewing only, with the file and line named; the app
writes nothing to it and repairs nothing.

**Warnings:** an input missing or changed (named; an amend, reprint or restore that needs it stops); an
output missing (**Restore** on its card).

**Keeps:** I6, I7, I8, I9, I10 — every decision below is taken from these files.

**Check:** should a damaged date also block work on other dates? (It does not.)
