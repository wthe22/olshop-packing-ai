# Data

Where the app keeps what it knows, and which file is the truth. The rules that fill the files are in
[../workflow/](../workflow/README.md).

| File | Content |
|---|---|
| [journal.md](journal.md) | The journal: line format, commit, torn write, the fold and what is checked when a date opens |
| [records.md](records.md) | The journal's records and their fields |
| [views.md](views.md) | `session.toml`, `orders.csv`, `order-contents.toml`: what the journal says, written for reading |

## Beside the program

```
Packing.exe
pdfium.dll
packing-settings.toml      where the data folder is, the theme
packing-workflows.toml     couriers, courier clues, sort rules
```

- `packing-settings.toml`: `schema`, `data_dir` (an absolute path or one relative to the program's
  folder) and `theme`. Written at the first start ([Start](../ui/start.md)), by Settings › **Change…** and
  by Settings › **Theme**; each keeps the other's value.
- `packing-workflows.toml`: the owner's rules ([rules file](../rules-file/README.md)). Written from the copy
  bundled in the program when it is missing; from then on it is the owner's.

```
Root {
  schema: String // X.Y version, starts at 1.0
  data_dir: Path // absolute, or relative to the program's folder
  theme: Theme   // system (missing: system) | light | dark ([D-UI-8](../decisions/ui.md))
}
```

## The data folder

```
{data_dir}/
  .lock  .lock-holder                     one copy of the app at a time
  {workspace}/                            one per workspace
    {date}/                               2026-10-07
      journal.jsonl                       the truth
      session.toml  orders.csv  order-contents.toml       views of the journal
      day summary.pdf                     a view of the journal, written on request
      input/{sha256}.pdf                  each input PDF once, named by its content
      output/{batch}-{code} {name} x{count}.pdf           one per label group
      output/{batch} summary.pdf          one per batch
      output/reprint {n} x{count}.pdf     one per reprint
      output/.partial/                    outputs of the operation in progress
```

A **workspace** keeps its batches apart from every other ([workspaces](../app/workspaces.md)); a date folder
holds its work of one date. Batch numbers run through a workspace and date.

## What is truth, and what is derived

| File | Kind | Written | Read for |
|---|---|---|---|
| `journal.jsonl` | **Truth** | One append per commit ([journal](journal.md)) | Every decision |
| `session.toml`, `orders.csv`, `order-contents.toml` | Views | After every commit; rebuilt when missing or stale ([views](views.md)) | The owner and other programs. Never read for a decision |
| `input/` | Inputs | At save, before the commit | Reading, amend, reprint, restore, the verifier |
| `output/` | Outputs | At save: written aside, verified, moved in after the commit | Printing |
| `packing-workflows.toml` | The owner's rules | The owner, the Sort rules screen, the logo prompt | Reading and planning; a saved batch keeps its own copy of the rule it used |
| `packing-settings.toml` | Setting | First start, Settings | Finding the data folder |

Every saved fact has one home, the journal; a view and the search index are rebuilt from it and are only
read. An output is reproducible from `input/` and the journal ([reprint](../app/reprint.md)).

## One writer

The data folder holds `.lock`, locked with `File::try_lock` while the app runs. The app that holds it writes
its computer name, process and start time into `.lock-holder` (Windows refuses to read a locked file, so
the holder is written beside it). A second copy that finds `.lock` locked reads `.lock-holder` and says
which copy has the folder ([Messages](../ui/messages.md)); it reads and writes nothing else.

## Types, versions and names

- `Date` is `2026-10-07`, `Time` is `14:26`, `Datetime` is ISO 8601 with the UTC offset
  (`2026-10-07T14:26:00+07:00`). A `*_time` field is a `Datetime`.
- `PdfID: u8` (an input of the date), `BatchID: u8` (1, 2, 3, … within the workspace and date),
  `OrderContentID: u16` and `RuleID: u8` (numbered in the views). `CourierCode` is `Courier.code` of
  `packing-workflows.toml`.
- Every TOML file and the journal's first record carry `schema` as an `X.Y` version, starting at `1.0`.
  The schema version is not the app version (`v0.2` writes `1.0`); every journal record also carries the
  app version that wrote it. Data of an older schema is not read and not migrated
  ([D-APP-3](../decisions/app.md)).
- **Default file names** use only letters, digits, space, `-` and `_`. Any other character becomes a space,
  repeated spaces become one, the ends are trimmed, and a name is cut at 100 characters. The owner may
  still change an output's name before saving; such a name must be a valid Windows file name and unique in
  the batch (upper and lower case count as the same).
- Page numbers count from 1 within their PDF.
