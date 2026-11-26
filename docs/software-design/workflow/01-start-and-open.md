# Steps 1–2: start, and open a date

## 1. Start

1. **Data folder.** Read `packing-app.toml` beside the program; `data_dir` is absolute or relative to
   the program's folder. Missing: the [Start](../ui/start.md) screen asks once (**Use `Packing\packing-data`** or
   **Choose a folder**) and writes the file.
2. **Migration.** Find, reading only, what the migration would change: date folders of v0.7 (with
   `input/` or `output/`, or a backup whose conversion stopped), dates whose `session.toml` is of another
   `schema`, and live batches saved before the receiver's area was read whose label PDFs are in the date
   folder (on a date of v0.7, will be once it is flat); a damaged journal is passed over. If there is any, the
   [Start](../ui/start.md#update-the-data-folder) screen asks before any date opens: **Update** or **Open
   read-only** ([D-APP-33](../decisions/app.md)). Choosing another data folder asks the same.
   - **Update**, holding the data folder's lock, date by date, oldest first: a date of v0.7 is made flat
     first ([D-APP-32](../decisions/app.md)): its old folders move into `v0.7 backup/`, and each file is
     copied up under its new name and checked by its hash; then the orders of each batch are
     read again from the first page of their copies in its stored label PDFs and recorded in one
     `areas_filled` record ([records](../data/records.md)); then the date's views are written. A batch
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
7. **Search index.** Fold every journal of the data folder on its own thread ([reprint](../app/reprint.md#the-search-index)).

**Stops:** `parser.toml` or `config.toml` that does not parse (its name, line and column); a data folder that cannot be
written.

**Check:** is today the right date to open on?

## 2. Open a date

Today's date (the PC's date) at start; another from the [panel](../ui/panel.md). For a date with a journal,
in this order ([journal › When a date opens](../data/journal.md#when-a-date-opens)):

1. **Fold** the journal: check every seal and `seq`, apply each record by its rule. The result: the live
   batches with their orders, groups, files and printed marks; the inputs table; the reprints; the next
   batch number (highest live + 1) and the next `PdfID` (highest ever + 1).
2. **Torn tail**: a last line without `\n` or with a bad seal is left out of the fold; the next commit cuts
   it and records it as `recovered`.
3. **Inputs**: SHA-256 of every input of a live batch, compared with its record.
4. **`.partial/`**: rolled forward when the journal's last record wrote what it holds, else back.
5. **Outputs**: each output of a live batch exists with its recorded size; a missing or changed one gets
   **Restore**.
6. **Views**: written again when missing or when `session.toml`'s `journal_seq` is not the last `seq`.

A date with no journal is empty: nothing is read and no folder is made until its first commit.

Steps 4 and 6 write: they run only while this copy holds the data folder's lock, taken if no other copy
writes. While another copy writes, the date opens for viewing only and opens again once it is done. An open
date is opened again whenever its journal grew: another copy committed to it
([program › Several copies](../app/program.md#several-copies)). In read-only mode steps 4 and 6 never run.

**Stops:** damage — a bad line before the last, a `seq` gap, a record that breaks its rule, a `started`
record naming another date or schema. The date opens for viewing only, with the line named; the app
writes nothing to it and repairs nothing.

**Warnings:** an input missing or changed (named; an amend, reprint or restore that needs it stops); an
output missing or changed (**Restore** on its card).

**Keeps:** I6, I7, I8, I9, I10 — every decision below is taken from this fold.

**Check:** should a damaged date also block work on other dates? (It does not.)
