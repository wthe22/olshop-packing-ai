# The journal

`{namespace}/{date}/journal.jsonl` is the only truth of a date ([D-APP-7](../decisions/app.md)). Everything
else in the date folder is either an input it names, an output it vouches for, or a view written from it.
Crate `packing-store` owns it.

## A line

```
3f9a0c71b2d4e5f6 {"seq":2,"time":"2026-10-07T07:40:12+07:00","app":"0.2.0","type":"batch_saved",…}
```

- One record per line: the **seal** (XXH3-64 of the JSON bytes, 16 lowercase hex characters), one space,
  the record as compact JSON on one line, then `\n`. The seal covers exactly the bytes between the space
  and the `\n`.
- Every record carries `seq` (1, 2, 3, … with no gap), `time`, `app` (the app version that wrote it) and
  `type`. The record types and their fields are in [records.md](records.md).
- Seq 1 is `started`: the schema (`1.0`), the namespace and the date. They must equal the folder's names.

## A commit

A commit is **one append of the record's line, then `sync_all`**. Nothing is saved before it returns;
everything is saved after it. The first commit of a date writes `started` and its own record in the same
append. Outputs move into `output/` only after the commit ([04 Save](../workflow/04-save.md)).

## The fold

Opening a date reads the journal top to bottom and applies the records in order — the **fold**. It gives
the live batches, the inputs of the date, the printed marks, the reprints and the next numbers. Every
decision (the already-saved guard, the next batch number, what may be reverted) is taken from a fold of
the journal as it is on disk, never from a view or the search index.

| Record | Rule the fold checks |
|---|---|
| `started` | Seq 1, and only there; namespace and date equal the folder |
| `batch_saved`, `amends: false` | `batch` = the highest live batch + 1 (1 on a date with none) |
| `batch_saved`, `amends: true` | `batch` = the highest live batch |
| `batch_reverted` | `batch` = the highest live batch |
| `printed_marked` | The batch is live and has that label group |
| `output_restored` | The batch is live and the file is one of its outputs |
| `reprint_made` | `number` = the date's reprints so far + 1 |
| `recovered` | — |
| every record | A `PdfID` always names the same `sha256`, and a `sha256` one `PdfID` |

A batch is **live** from its `batch_saved` until a `batch_reverted` of it; an amend's record replaces the
batch's earlier one. The inputs table holds every input any record named, live or not, so a `PdfID` is
never given twice within a date.

## A torn write, and damage

| The journal holds | It is | What happens |
|---|---|---|
| A **last** line without `\n`, or whose seal does not match | A **torn write**: the app stopped during an append | The fold ignores it. The next commit first cuts the file at the end of the last good line and appends a `recovered` record holding the dropped bytes, then its own record |
| A bad line **before** the last, a gap in `seq`, or a record that breaks a rule of the table | **Damage** | The date stops with a plain message naming the line ([Messages](../ui/messages.md)): it can be viewed, nothing is written to it. It is never a crash, and the app never repairs it |

## When a date opens

In this order, before the date is shown:

1. **Fold** the journal (above).
2. **Inputs**: every input of a live batch must exist in `input/` and have the `sha256` its record names.
   One that is missing or changed is reported ([Messages](../ui/messages.md)) and never used: an amend,
   reprint or restore that needs it stops with its name.
3. **`output/.partial/`**, if present, is settled by the journal's last record:
   - **forward** when that record is a `batch_saved`, `output_restored` or `reprint_made`, every file it
     writes is in `.partial/` or in `output/` with the `sha256` it names, and `.partial/` holds no other
     file — or a `batch_reverted`, `.partial/replaced/` holds that batch's files and `.partial/` nothing
     else: the new files are moved into `output/` and `replaced/` is deleted. (A file the record does not
     name is a later operation's that did not commit);
   - **back** otherwise: the files of `replaced/` are moved back into `output/`, the rest is deleted.
4. **Outputs**: every output of a live batch must exist in `output/` with the size the newest record that
   wrote it names (`batch_saved` or `output_restored`). A missing one is shown on its batch card with
   **Restore** ([reprint](../app/reprint.md#restore)).
5. **Views**: rebuilt when one is missing or `session.toml`'s `journal_seq` differs from the journal's
   last `seq` ([views](views.md)).

## Size

About 150 bytes a saved order plus its item lines once per distinct content: a date of three batches and
600 orders is about 150 KB. The search index folds every journal of a namespace when it opens
([reprint](../app/reprint.md#the-search-index)).
