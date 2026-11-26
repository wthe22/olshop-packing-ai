# Start

What the app asks before the Day screen, in the order of [program › Start](../app/program.md#start).

## The data folder (first start)

When `packing-app.toml` is missing beside the program, or names no data folder:

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│  ◈ Packing v0.7                                                                        │
│  Where should Packing keep its data?                                                   │
│  The labels, group PDFs and summary PDFs are kept there, one folder per date.          │
│  ┌ 🗀 Next to the program ─────────────────────────────────────────────────────────┐   │
│  │   D:\Packing\packing-data                                                      │   │
│  └─────────────────────────────────────────────────────────────────────────────────┘   │
│                                       [ Choose another folder ]  [ Use this folder ]   │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

- Both questions of a first start are a card in the middle of the window, under the program's name and
  version.
- **Use this folder** writes `data_dir = "packing-data"` (beside the program, so the folder can be copied whole).
- **Choose another folder** writes the chosen path, relative when it is inside the program's folder.
- A folder that cannot be written (e.g. under `Program Files`) is refused with the reason.
- `parser.toml` and `config.toml` are written into the folder from the bundled copies when they are
  missing: the couriers and their clues in `parser.toml`, and a `config.toml` with no account and no sort rule (a
  batch then uses the built-in `default`, one group that takes every order) ([rules files](../rules-file/README.md)). A data folder that has them keeps them.

## Update the data folder?

When the data folder holds data this version keeps more of, before any date opens
([D-APP-33](../decisions/app.md)): a date folder of v0.7, a date whose views are of an older `schema`, or a
batch saved before the receiver's area or the weight was read whose label PDFs are in its date folder. Choosing another data folder (the start's
**Choose a folder**, Settings › **Change**) asks the same about that folder.

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│  ◈ Packing v0.8.4-alpha                                                                │
│  Update the data folder?                                                               │
│  Update moves the files of 1 date out of the input and output folders of Packing v0.7  │
│  into their date folders, keeping the old folders in v0.7 backup. It reads the         │
│  receivers' addresses and the parcels' weights of 3 batches from their label PDFs and  │
│  writes the files that list each date again. No batch, label or PDF is changed or      │
│  deleted. Read-only shows the data as it is and changes nothing.                       │
│  ┌ DATE ─────────────── WHAT CHANGES ──────────────────────────────────────────────┐   │
│  │ Tue, 24 Nov 2026     Files of v0.7 moved in · Addresses and weights of 2        │   │
│  │                      batches, 61 orders · the date's lists                      │   │
│  │ Wed, 25 Nov 2026     the date's lists                                           │   │
│  └─────────────────────────────────────────────────────────────────────────────────┘   │
│                                                     [ Open read-only ]  [ Update ]     │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

- **Update** runs the dates in order, oldest first; the card then shows its progress under the table: a line
  per date done (*2026-11-24: files moved, addresses and weights of 2 batches read, lists written*) and the
  step running (*Moving files: 2026-11-24, file 3 of 12*, then *Reading labels: 2026-11-24, batch 1, page 3 of
  120*). Each batch is one append to its date's journal; then the
  date's views are written. While another copy of the app writes, Update waits for it. When all are done,
  *Done. The data folder is up to date.* and **Continue** opens the Day screen.
- A batch whose label PDF has changed is named in red on its date's line (*Batch 2 not read: …*) and skipped;
  the rest go on, and the next start asks about it again. A batch whose label PDF is missing is not offered:
  it cannot be read, and its addresses and weights stay empty.
- An Update that stops (the data folder cannot be written) says why, and both buttons show again.
- **Open read-only** opens the Day screen in read-only mode. It is the only way into it.

## Read-only

The title bar shows a red *Read-only* mark beside the version, and the Day screen shows the line *Read-only:
this copy changes nothing in the data folder.* under its title. Nothing in the data folder is written: no lock file, no `.partial/` settled,
no view, no `parser.toml` or `config.toml` written from the bundled copies. What would write is not offered:
**+ New batch**, **Amend**, **Revert**, **Restore**, **Print**, the *Sent* and **Printed** marks, the summaries
made on request, **Reprint** and the ticks that choose its orders, the Accounts screen's changes, and the Sort
rules screen's changes (its groups take no input; its switch shows the rule's state). The **Printed** tick shows
its state and cannot be clicked. **Open**, Search, the panel and the Day screen of any date work as usual; a
write asked for anyway is refused with the same line. Starting the program again asks again. The theme and
Settings › **Change** still work: they write `packing-app.toml` beside the program, not the data folder
([D-APP-33](../decisions/app.md)); a folder chosen there opens as at start.

## No first account

The start asks no name: the Day screen of today opens. The first read of labels asks which account each shop
belongs to ([New batch › The shop prompt](new-batch.md#the-shop-prompt)), and accounts can be made ahead on
the [Accounts](accounts.md) screen.

## The rules files

A `parser.toml` or `config.toml` that does not parse stops here with its file, line and column and
**Open folder**. A
sort rule whose filter has an error opens the [Sort rules](sort-rules.md) screen on that rule before the Day
screen.
