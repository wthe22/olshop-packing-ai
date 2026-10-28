# Settings

One page over the current screen, full width: four groups of rows, each row the setting at the left (its
name and a line under it) and its control at the right.

```
┌ Settings ──────────────────────────────────────────────────────────────────────────── [×] ┐
│ APPEARANCE                                                                                │
│ ┌───────────────────────────────────────────────────────────────────────────────────────┐ │
│ │ Theme                                                    [☀ Light][☾ Dark][▭ System] │ │
│ │ Light, dark, or the same as Windows.                                                  │ │
│ └───────────────────────────────────────────────────────────────────────────────────────┘ │
│ DATA                                                                                      │
│ │ Data folder  C:Packingpacking-data                             [Open] [Change…]    │ │
│ WORKSPACES                                                                                │
│ Each workspace keeps its own dates and batches. A new one is made with the + in the panel.│
│ │ Toko Contoh  (Open now)   12 dates                              [Rename…] [Delete…]   │ │
│ │ Toko Lain                 No dates yet                          [Rename…] [Delete…]   │ │
│ ABOUT                                                                                     │
│ │ ◈ Packing v0.2 — Sorts the day's label PDFs into group PDFs to print, …               │ │
│ │ Version v0.2 · PDFium chromium/7881 · Journal schema 1.0, one row each                │ │
└───────────────────────────────────────────────────────────────────────────────────────────┘
```

| Group | Row | Control |
|---|---|---|
| Appearance | Theme | **Light**, **Dark** or **System** (Windows' own, the default), side by side; the chosen one raised ([D-UI-8](../decisions/ui.md)) |
| Data | Data folder | The folder of `packing-settings.toml` + **Open** + **Change…** |
| Workspaces | One row per workspace: its name, *Open now* on the open one, its dates | **Rename…** + **Delete…** |
| About | The program's name, version and what it does; then Version (`v0.2`), PDFium and Journal schema (`1.0`) | — |

There are no other settings: every other choice has one fixed default, shown where it acts
([D-APP-16](../decisions/app-features.md)). There is no log file: the journal of each date is the record of what was
done ([D-UI-2](../decisions/ui.md)).

**Theme** is shown at once and written into `packing-settings.toml`; the OS title bar follows it.

**Change…** chooses another folder and asks "Use `<folder>` for the data from now on? The data in
`<old folder>` stays where it is." It writes `packing-settings.toml` (a path inside the program's folder is
written relative to it), releases the old folder's lock, takes the new one's, and opens it as at start
([program › Start](../app/program.md#start)). A draft being prepared asks first. Nothing is moved.

**Rename…** opens the name field in the row with **Cancel** and **Rename**; Enter renames, `Esc` cancels. A
name the rules refuse ([workspaces › Names](../app/workspaces.md#names)) is told under the field. **Delete…**
asks "Delete workspace Toko Lain?" — "Its 3 dates, with their labels and files, go to the Recycle Bin."
[Delete workspace] [Cancel]. Renaming or deleting the open workspace asks first about a draft being prepared;
what follows is in [workspaces](../app/workspaces.md#listing-opening-creating-renaming-and-deleting).

## The version

The version of the program is the workspace `version` in the root `Cargo.toml`, and that is its **one
place**. The shown form is `v` + the first two numbers (`0.2.0` → `v0.2`). It appears in the window title,
the title bar of every screen and *About*, all fed from that one place.

- `app/src-tauri/tauri.conf.json` carries no `version` and no number in its window `title`; Tauri takes the
  version from `app/src-tauri/Cargo.toml`, which inherits the workspace's. `app/package.json`'s own
  `version` is npm's field: nothing shows it, and it is kept equal to the workspace's.
- The `v` + major.minor text is made in one place in the app crate. The journal's `app` field is the full
  workspace version (`0.2.0`).
