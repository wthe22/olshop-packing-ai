# Start

What the app asks before the Day screen, in the order of [program › Start](../app/program.md#start).

## The data folder (first start)

When `packing-settings.toml` is missing beside the program, or names no data folder:

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│  Where should Packing keep its data?                                                   │
│  Each owner's labels and outputs are kept there, one folder per owner and date.        │
│                                                                                        │
│  [ Use D:\Packing\data ]                 [ Choose a folder… ]                          │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

- **Use `…\data`** writes `data_dir = "data"` (beside the program, so the folder can be copied whole).
- **Choose a folder…** writes the chosen path, relative when it is inside the program's folder.
- A folder that cannot be written (e.g. under `Program Files`) is refused with the reason.
- `packing-workflows.toml` is written from the bundled copy when it is missing: the couriers, their clues
  and one sort rule `default` whose only group takes every order ([rules file](../rules-file/README.md)).

## Another copy is open

```
│  Packing is already open on PC-GUDANG (since 08:12).                                   │
│  Close it there, then try again.                                    [ Try again ]      │
```

## The first owner

When the data folder holds no owner yet:

```
│  Whose labels are these?   Owner [ Toko Contoh            ]             [ Create ]     │
```

The name follows [namespaces › Names](../app/namespaces.md#names); **Create** makes the owner's folder and
opens today's date. More owners are added in the [panel](panel.md).

## The rules file

A `packing-workflows.toml` that does not parse stops here with its line and column and **Open folder**. A
sort rule whose filter has an error opens the [Sort rules](sort-rules.md) screen on that rule before the Day
screen.
