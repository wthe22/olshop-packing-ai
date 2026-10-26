# Start

What the app asks before the Day screen, in the order of [program › Start](../app/program.md#start).

## The data folder (first start)

When `packing-settings.toml` is missing beside the program, or names no data folder:

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│  ◈ Packing v0.2                                                                        │
│  Where should Packing keep its data?                                                   │
│  Each namespace's labels, group PDFs and summary PDFs are kept there, one folder per   │
│  namespace and date.                                                                   │
│  ┌ 🗀 Next to the program ─────────────────────────────────────────────────────────┐   │
│  │   D:\Packing\data                                                              │   │
│  └─────────────────────────────────────────────────────────────────────────────────┘   │
│                                       [ Choose another folder… ]  [ Use this folder ]  │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

- Both questions of a first start are a card in the middle of the window, under the program's name and
  version.
- **Use this folder** writes `data_dir = "data"` (beside the program, so the folder can be copied whole).
- **Choose another folder…** writes the chosen path, relative when it is inside the program's folder.
- A folder that cannot be written (e.g. under `Program Files`) is refused with the reason.
- `packing-workflows.toml` is written from the bundled copy when it is missing: the couriers, their clues
  and one sort rule `default` whose only group takes every order ([rules file](../rules-file/README.md)).

## Another copy is open

```
│  Packing is already open on PC-GUDANG (since 08:12).                                   │
│  Close it there, then try again.                                    [ Try again ]      │
```

## The first namespace

When the data folder holds no namespace yet:

```
│  Name the first namespace.   [ Toko Contoh            ]                  [ Create ]    │
```

The name follows [namespaces › Names](../app/namespaces.md#names); **Create** makes the namespace's folder
and opens today's date. More namespaces are added in the [panel](panel.md).

## The rules file

A `packing-workflows.toml` that does not parse stops here with its line and column and **Open folder**. A
sort rule whose filter has an error opens the [Sort rules](sort-rules.md) screen on that rule before the Day
screen.
