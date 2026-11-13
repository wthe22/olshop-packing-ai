# Start

What the app asks before the Day screen, in the order of [program › Start](../app/program.md#start).

## The data folder (first start)

When `packing-app.toml` is missing beside the program, or names no data folder:

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│  ◈ Packing v0.5                                                                        │
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
  missing: the couriers and their clues, and one sort rule `default` whose only group takes every order,
  with no account ([rules files](../rules-file/README.md)). A data folder that has them keeps them.

## No first account

The start asks no name: the Day screen of today opens. The first read of labels asks which account each shop
belongs to ([New batch › The shop prompt](new-batch.md#the-shop-prompt)), and accounts can be made ahead on
the [Accounts](accounts.md) screen.

## The rules files

A `parser.toml` or `config.toml` that does not parse stops here with its file, line and column and
**Open folder**. A
sort rule whose filter has an error opens the [Sort rules](sort-rules.md) screen on that rule before the Day
screen.
