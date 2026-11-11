# Settings

One page over the current screen, full width: three groups of rows, each row the setting at the left (its
name and a line under it) and its control at the right.

```
┌ Settings ──────────────────────────────────────────────────────────────────────────── [×] ┐
│ APPEARANCE                                                                                │
│ ┌───────────────────────────────────────────────────────────────────────────────────────┐ │
│ │ Theme                                                    [☀ Light][☾ Dark][▭ System] │ │
│ │ Light, dark, or the same as Windows.                                                  │ │
│ └───────────────────────────────────────────────────────────────────────────────────────┘ │
│ DATA                                                                                      │
│ │ Data folder  C:Packingpacking-data                             [Open] [Change]     │ │
│ ABOUT                                                                                     │
│ │ ◈ Packing v0.5 — Sorts the day's label PDFs into group PDFs to print, …               │ │
│ │ Version v0.5.0 · PDFium chromium/7881 · Journal schema 2.0, one row each                │ │
└───────────────────────────────────────────────────────────────────────────────────────────┘
```

| Group | Row | Control |
|---|---|---|
| Appearance | Theme | **Light**, **Dark** or **System** (Windows' own, the default), side by side; the chosen one raised ([D-UI-8](../decisions/ui.md)) |
| Data | Data folder | The folder of `packing-settings.toml` + **Open** + **Change** |
| About | The program's name, version and what it does; then Version (`v0.4.0`), Built (the time the program was compiled, `2026-10-30 09:15`), PDFium and Journal schema (`2.0`) | — |

There are no other settings: every other choice has one fixed default, shown where it acts
([D-APP-16](../decisions/app-features.md)). There is no log file: the journal of each date is the record of what was
done ([D-UI-2](../decisions/ui.md)). The accounts and their shops are on the [Accounts](accounts.md) screen.

**Theme** is shown at once and written into `packing-settings.toml`; the OS title bar follows it.

**Change** chooses another folder and asks "Use `<folder>` for the data from now on? The data in
`<old folder>` stays where it is." It writes `packing-settings.toml` (a path inside the program's folder is
written relative to it) and opens the folder as at start
([program › Start](../app/program.md#start)). A draft being prepared asks first. Nothing is moved.

## The version

The version of the program is `workspace.package.version` in the root `Cargo.toml`, and that is its **one
place**. The shown form is `v` + the whole version (`0.4.0` → `v0.4.0`). It appears in the window title,
the title bar of every screen and *About*, all fed from that one place.

- `app/src-tauri/tauri.conf.json` carries no `version` and no number in its window `title`; Tauri takes the
  version from `app/src-tauri/Cargo.toml`, which inherits the root's. `app/package.json`'s own
  `version` is npm's field: nothing shows it, and it is kept equal to the root's.
- The `v` + version text is made in one place in the app crate. The journal's `app` field is the full
  version (`0.5.0`).
