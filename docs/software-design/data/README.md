# Data

Where the app keeps what it knows. The rules that fill the files are in [../workflow/](../workflow/README.md).

| File | Content |
|---|---|
| [files.md](files.md) | `day.toml`, `orders.csv`, `order-contents.toml`: the truth of a date and their fields |
| [save.md](save.md) | How a change commits, and what is checked when a date opens |

## The files

```
Packing.exe
pdfium.dll
packing-app.toml          where the data folder is, the theme, archiving on its own
packing-data\             the data folder (its name is the owner's choice)
  parser.toml             how labels are read: couriers, courier clues
  config.toml             the accounts and the sort rules
  2026-10-07\ …           one folder per date, every file of the date directly in it:
    day.toml              the truth, with orders.csv and order-contents.toml
    orders.csv
    order-contents.toml
    2 Input 03 61459e8a8203.pdf   an input of batch 2
    2-A Sepatu x12.pdf    a group PDF of batch 2
    2 Summary.pdf         the summary PDF of batch 2
    Reprint 1 x4.pdf      a reprint
    Day Summary.pdf       a summary made on request
    .partial\             what a change writes before its commit
```

- `packing-app.toml` (beside the program): the app's preferences. Written at the first start
  ([Start](../ui/start.md)), by Settings › **Change**, **Theme** and **Archive automatically**; each write keeps
  the other values.
- `parser.toml` and `config.toml` (in the data folder): the owner's rules
  ([rules files](../rules-file/README.md)). Written from the copies bundled in the program when missing;
  from then on they are the owner's. They travel with the data folder, so a copy on another PC that uses
  the same data folder shares them ([D-APP-30](../decisions/app-features.md)).

```
Root {
  schema: String // X.Y version, starts at 1.0
  data_dir: Path // absolute, or relative to the program's folder
  theme: Theme   // system (missing: system) | light | dark ([D-UI-8](../decisions/ui.md))
  archive_after_days: Option<u16> // archive dates older than this ([archive](../app/archive.md)); missing: off
}
```
A date folder holds the work of one date for every account ([accounts](../app/accounts.md)); batch numbers
run through a date. A date folder has no subfolder but `.partial/` ([D-APP-32](../decisions/app.md)); a file
`day.toml` does not name is the owner's.

## What is truth, and what is derived

| File | Kind | Written | Read for |
|---|---|---|---|
| `day.toml`, `orders.csv`, `order-contents.toml` | **Truth** | Whole, at every commit ([save](save.md)) | Every decision, the screens, the owner and other programs |
| `{batch} Input {pdf} {short hash}.pdf` | Inputs | At save, before the commit | Reading, amend, reprint, restore, the verifier |
| The group PDFs, `{batch} Summary.pdf`, `Reprint {n} x{count}.pdf` | Outputs | Written into `.partial/`, verified, moved in after the commit | Printing |
| The summaries made on request | Outputs | **PDF** on the Summary screen | Reading and printing |
| `parser.toml` | The owner's parser rules | The owner, the logo prompt | Reading labels; a saved batch keeps what it used |
| `config.toml` | The accounts and the sort rules | The owner, the Sort rules screen, the Accounts screen, the shop prompt | Planning, and an order's account when it is shown ([accounts](../app/accounts.md#the-account-of-an-order)); a saved batch keeps its own copy of the rule it used |
| `packing-app.toml` (beside the program) | Preferences | First start, Settings | Finding the data folder; the theme |

`parser.toml` and `config.toml` are inputs to a read, like the label files: nothing saved is decided by
them after the save.

Every saved fact has one home in the three files; the search index is built from them and only read. An
output is reproducible from the inputs and the three files ([reprint](../app/reprint.md)).

## One writer

Several copies of the app may use the data folder at once ([program › Several copies](../app/program.md#several-copies)).
The data folder holds `.lock`, locked with `File::try_lock` only while a copy writes: a commit with the files
it moves, opening a date that settles `.partial/`, a summary made on request, a write of `parser.toml` or
`config.toml`. The copy that holds it writes its computer name, process and the time into `.lock-holder`
(Windows refuses to read a locked file, so the holder is written beside it). Another copy's write waits for
the lock, a minute at most, then names the holder ([Messages](../ui/messages.md)).

Reading needs no lock. A copy finds another's commit by the last-line hash of `day.toml` and reads the date
again; a write reads it again under the lock before it decides. A date opened while another copy holds the
lock is opened for viewing only — `.partial/` is that copy's — and opened again once the lock is free.

## Types, versions and names

- `Date` is `2026-10-07`, `Time` is `14:26`, `Datetime` is ISO 8601 with the UTC offset
  (`2026-10-07T14:26:00+07:00`). A `*_time` field is a `Datetime`.
- `PdfID: u8` (an input of the date), `BatchID: u8` (1, 2, 3, … within the date),
  `OrderContentID: u16` and `RuleID: u8`. `CourierCode` is `Courier.code` of `parser.toml`.
- Every TOML file and `orders.csv` (its last column's header, `schema:2.1`) carry `schema` as an `X.Y`
  version, starting at `1.0`. The schema version is not the app version: `day.toml` and `orders.csv` `2.1`
  (a date for all accounts), `packing-app.toml`, `parser.toml`, `config.toml` and `order-contents.toml`
  `1.1` ([D-APP-3](../decisions/app.md)).
- **A copy reads every version of its own X** and passes over what it does not know: a key, a field, a
  column. A change an older copy of the same X can pass over keeps X; one that needs another way of reading
  or changes what data means raises X. A file of a higher X is not read: the copy says it is of a newer
  format. A file of a lower X is read only by the migration the owner starts.
- **Nothing is lost** but by an action meant to remove it: a copy writes no date whose files hold a key or a
  column it does not know, nor any file of a newer Y; such a date opens for viewing only, and the Day screen
  says why ([D-APP-34](../decisions/versions.md)).
- **Default file names** use only letters, digits, space, `-` and `_`. Any other character becomes a space,
  repeated spaces become one, the ends are trimmed, and a name is cut at 100 characters. The owner may
  still change an output's name before saving; such a name must be a valid Windows file name and unique in
  the batch (upper and lower case count as the same).
- Page numbers count from 1 within their PDF.
