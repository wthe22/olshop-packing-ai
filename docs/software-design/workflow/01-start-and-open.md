# Steps 1–2: start, and open a workspace's date

## 1. Start

1. **Data folder.** Read `packing-settings.toml` beside the program; `data_dir` is absolute or relative to
   the program's folder. Missing: the [Start](../ui/start.md) screen asks once (**Use `Packing\packing-data`** or
   **Choose a folder…**) and writes the file.
2. **Rules.** Read `packing-workflows.toml` beside the program; missing: write the bundled copy. Run its
   checks ([rules file](../rules-file/README.md#checks)).
3. **Lock.** Open `{data_dir}/.lock` and `try_lock` it; on success write `.lock-holder` (computer, process,
   start time) ([data](../data/README.md#one-writer)).
4. **v0.1 data.** If the data folder holds `labels/`, `state.json`, `couriers.toml`, `categories.toml` or
   `categories/`, the Day screen will say so; nothing of it is read or deleted.
5. **Workspace.** The one whose `journal.jsonl` was written last; none: ask for the first workspace's name and
   create its folder ([workspaces](../app/workspaces.md)).
6. **Search index.** Fold every journal of the workspace on its own thread ([reprint](../app/reprint.md#the-search-index)).

**Stops:** `packing-workflows.toml` that does not parse (its line and column); the lock held by another copy
(which one, and **Try again**); a data folder that cannot be written.

**Check:** is *the workspace whose journal was written last* the right one to open on?

## 2. Open a date

Today's date (the PC's date) at start; another from the [panel](../ui/panel.md). For a date with a journal,
in this order ([journal › When a date opens](../data/journal.md#when-a-date-opens)):

1. **Fold** the journal: check every seal and `seq`, apply each record by its rule. The result: the live
   batches with their orders, groups, files and printed marks; the inputs table; the reprints; the next
   batch number (highest live + 1) and the next `PdfID` (highest ever + 1).
2. **Torn tail**: a last line without `\n` or with a bad seal is left out of the fold; the next commit cuts
   it and records it as `recovered`.
3. **Inputs**: SHA-256 of every input of a live batch, compared with its record.
4. **`output/.partial/`**: rolled forward when the journal's last record wrote what it holds, else back.
5. **Outputs**: each output of a live batch exists with its recorded size; a missing or changed one gets
   **Restore**.
6. **Views**: written again when missing or when `session.toml`'s `journal_seq` is not the last `seq`.

A date with no journal is empty: nothing is read and no folder is made until its first commit.

**Stops:** damage — a bad line before the last, a `seq` gap, a record that breaks its rule, a `started`
record naming another workspace or date. The date opens for viewing only, with the line named; the app
writes nothing to it and repairs nothing.

**Warnings:** an input missing or changed (named; an amend, reprint or restore that needs it stops); an
output missing or changed (**Restore** on its card).

**Keeps:** I6, I7, I8, I9, I10 — every decision below is taken from this fold.

**Check:** should a damaged date also block work on other dates of the workspace? (It does not.)
