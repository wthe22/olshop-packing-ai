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
([D-APP-33](../decisions/app.md)): a date folder of v0.7, a date whose files are of an older `schema`, a date with `journal.jsonl`, or a
batch saved before the receiver's area, the weight or the shop was read whose label PDFs are in its date folder. Choosing another data folder (the start's
**Choose a folder**, Settings › **Change**) asks the same about that folder.

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│  ◈ Packing v0.8.0-alpha.12                                                             │
│  Update the data folder?                                                               │
│  29 dates were saved by an older Packing. Updating takes a few minutes and keeps a     │
│  copy of every file it changes.                                                        │
│                                                     [ Open read-only ]  [ Update ]     │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

- The card says only how many dates and that a copy is kept ([D-UI-20](../decisions/ui-screens.md)); what
  each date needs is not listed.
- **Update** runs the dates in order, oldest first ([01 Start](../workflow/01-start-and-open.md#1-start)). The
  buttons give way to one line and a bar for the whole run: *Updating 2026-11-24, 3 of 29* and, under it in
  grey, the step (*Reading labels: page 3 of 120*). While another copy of the app writes, Update waits for it.
  When all are done the Day screen opens; when a batch was skipped, first one line says so with **Continue**.
- A batch whose label PDF has changed is skipped (*1 batch not read: 2026-11-24 batch 2, its label PDF has
  changed.*); the rest go on, and the next start asks about it again. A batch whose label PDF is missing is not offered:
  it cannot be read, and its addresses and weights stay empty.
- An Update that stops (the data folder cannot be written) says why, and both buttons show again.
- **Open read-only** opens the Day screen in read-only mode. It is the only way into it.

## Read-only

The title bar shows a red *Read-only* mark beside the version, and the Day screen shows the line *Read-only:
this copy changes nothing in the data folder.* under its title. Nothing in the data folder is written: no lock file, no `.partial/` settled,
no date file, no `parser.toml` or `config.toml` written from the bundled copies. What would write is not offered:
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
