# Data

Where the app keeps what it knows, and which file is the truth. The rules that fill the files are in
[../workflow/](../workflow/README.md).

| File | Content |
|---|---|
| [journal.md](journal.md) | The journal: line format, commit, torn write, the fold and what is checked when a date opens |
| [records.md](records.md) | The journal's records and their fields |
| [views.md](views.md) | `session.toml`, `orders.csv`, `order-contents.toml`: what the journal says, written for reading |

## The files

```
Packing.exe
pdfium.dll
packing-app.toml          where the data folder is, the theme
packing-data\             the data folder (its name is the owner's choice)
  parser.toml             how labels are read: couriers, courier clues
  config.toml             the accounts and the sort rules
  2026-10-07\ …           one folder per date
```

- `packing-app.toml` (beside the program): the app's preferences. Written at the first start
  ([Start](../ui/start.md)), by Settings › **Change** and **Theme**; each write keeps the other value.
- `parser.toml` and `config.toml` (in the data folder): the owner's rules
  ([rules files](../rules-file/README.md)). Written from the copies bundled in the program when missing;
  from then on they are the owner's. They travel with the data folder, so a copy on another PC that uses
  the same data folder shares them ([D-APP-30](../decisions/app-features.md)).

```
Root {
  schema: String // X.Y version, starts at 1.0
  data_dir: Path // absolute, or relative to the program's folder
  theme: Theme   // system (missing: system) | light | dark ([D-UI-8](../decisions/ui.md))
}
```
A date folder holds the work of one date for every account ([accounts](../app/accounts.md)); batch numbers
run through a date. `output/` holds only the files the journal records; a summary made on request sits
beside the journal.

## What is truth, and what is derived

| File | Kind | Written | Read for |
|---|---|---|---|
| `journal.jsonl` | **Truth** | One append per commit ([journal](journal.md)) | Every decision |
| `session.toml`, `orders.csv`, `order-contents.toml`, the summaries made on request | Views | After every commit; rebuilt when missing or stale ([views](views.md)) | The owner and other programs. Never read for a decision |
| `input/` | Inputs | At save, before the commit | Reading, amend, reprint, restore, the verifier |
| `output/` | Outputs | At save: written aside, verified, moved in after the commit | Printing |
| `parser.toml` | The owner's parser rules | The owner, the logo prompt | Reading labels; a saved batch keeps what it used |
| `config.toml` | The accounts and the sort rules | The owner, the Sort rules screen, the Accounts screen, the shop prompt | Planning, and an order's account when labels are read ([accounts](../app/accounts.md#the-account-of-an-order)); a saved batch keeps its own copy of the rule it used |
| `packing-app.toml` (beside the program) | Preferences | First start, Settings | Finding the data folder; the theme |

`parser.toml` and `config.toml` are inputs to a read, like the label files: nothing in the journal is
decided by them after the save.

Every saved fact has one home, the journal; a view and the search index are rebuilt from it and are only
read. An output is reproducible from `input/` and the journal ([reprint](../app/reprint.md)).

## One writer

Several copies of the app may use the data folder at once ([program › Several copies](../app/program.md#several-copies)).
The data folder holds `.lock`, locked with `File::try_lock` only while a copy writes: a commit with the file
moves around it, opening a date that settles `.partial/` or writes the views, a summary made on request, a
write of `parser.toml` or `config.toml`. The copy that holds it writes its computer name, process and the time into
`.lock-holder` (Windows refuses to read a locked file, so the holder is written beside it). Another copy's
write waits for the lock, a minute at most, then names the holder ([Messages](../ui/messages.md)).

Reading needs no lock. A copy finds another's commit by the journal's length on disk (the journal only
grows) and folds it again; a write does so under the lock before it decides, and the append refuses a
journal that grew meanwhile. A date opened while another copy holds the lock is opened for viewing only —
`.partial/` and the views are that copy's — and opened again once the lock is free.

## Types, versions and names

- `Date` is `2026-10-07`, `Time` is `14:26`, `Datetime` is ISO 8601 with the UTC offset
  (`2026-10-07T14:26:00+07:00`). A `*_time` field is a `Datetime`.
- `PdfID: u8` (an input of the date), `BatchID: u8` (1, 2, 3, … within the date),
  `OrderContentID: u16` and `RuleID: u8` (numbered in the views). `CourierCode` is `Courier.code` of
  `parser.toml`.
- Every TOML file and the journal's first record carry `schema` as an `X.Y` version, starting at `1.0`.
  The schema version is not the app version: the journal and `session.toml` are `2.0` (a date for all
  accounts), `packing-app.toml`, `parser.toml`, `config.toml` and `order-contents.toml` `1.0`. Every journal record also carries the
  app version that wrote it. Data of an older schema is not read and not migrated
  ([D-APP-3](../decisions/app.md)).
- **Default file names** use only letters, digits, space, `-` and `_`. Any other character becomes a space,
  repeated spaces become one, the ends are trimmed, and a name is cut at 100 characters. The owner may
  still change an output's name before saving; such a name must be a valid Windows file name and unique in
  the batch (upper and lower case count as the same).
- Page numbers count from 1 within their PDF.
