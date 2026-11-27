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
│ STORAGE                                                                                   │
│ │ Used          3.2 GB in 41 dates · 12 archived                                       │ │
│ │ Archive dates  From [2026-09-01] To [2026-09-30]  18 dates · 1.9 GB     [Archive]    │ │
│ │ Archive automatically  Dates older than [ 60 ] days               [Off][On]          │ │
│ ABOUT                                                                                     │
│ │ ◈ Packing v0.7 — Sorts the day's label PDFs into group PDFs to print, …               │ │
│ │ Version v0.7.0 · PDFium chromium/7881 · Date schema 2.1, one row each                   │ │
└───────────────────────────────────────────────────────────────────────────────────────────┘
```

| Group | Row | Control |
|---|---|---|
| Appearance | Theme | **Light**, **Dark** or **System** (Windows' own, the default), side by side; the chosen one raised ([D-UI-8](../decisions/ui.md)) |
| Data | Data folder | The folder of `packing-app.toml` + **Open** + **Change** |
| Data | *Kept, not read*, only when there is one: each key of `packing-app.toml`, `parser.toml` or `config.toml` this version does not know, with its file and line, in a file of this version or older ([D-APP-34](../decisions/versions.md)); read each time Settings opens | — |
| Storage | Used: the data folder's size, its dates and how many are archived; read each time Settings opens | — |
| Storage | Archive dates: **From** and **To** (`yyyy-mm-dd`), then how many dates of the range are not archived yet and the size of their PDFs, then **Archive** | **Archive** |
| Storage | Archive automatically: *Dates older than* a number of days | **Off** / **On**, side by side as Theme |
| About | The program's name, version and what it does; then Version (`v0.4.0`), Built (the time the program was compiled, `2026-10-30 09:15`), PDFium and Date schema (`2.1`) | — |

There are no other settings but [Storage](#storage): every other choice has one fixed default, shown where it acts
([D-APP-16](../decisions/app-features.md)). There is no log file: a date's files hold what was saved
([D-UI-2](../decisions/ui.md)). The accounts and their shops are on the [Accounts](accounts.md) screen.

**Theme** is shown at once and written into `packing-app.toml`; the OS title bar follows it.

**Change** chooses another folder and asks "Use `<folder>` for the data from now on? The data in
`<old folder>` stays where it is." It writes `packing-app.toml` (a path inside the program's folder is
written relative to it) and opens the folder as at start
([program › Start](../app/program.md#start)). A draft being prepared asks first. Nothing is moved.

## The version

The version of the program is `workspace.package.version` in the root `Cargo.toml`, and that is its **one
place**. The shown form is `v` + the whole version (`0.4.0` → `v0.4.0`). It appears in the window title,
the title bar of every screen and *About*, all fed from that one place.

- `app/src-tauri/tauri.conf.json` carries no `version` and no number in its window `title`; Tauri takes the
  version from `app/src-tauri/Cargo.toml`, which inherits the root's. `app/package.json`'s own
  `version` is npm's field: nothing shows it, and it is kept equal to the root's.
- The `v` + version text is made in one place in the app crate.

## Storage

The rows of [archive](../app/archive.md). The range starts at the oldest date and ends yesterday; **Archive** of a range is off while the range holds no date to
archive (today, a date with no batch and an archived date are passed over), and asks first, naming the dates,
the PDFs and their size. **Archive automatically** starts *Off*; **On** keeps the number (at least 1, 60 to
begin with), asks first when it would archive any date now, and writes `archive_after_days` into
`packing-app.toml`; **Off** removes it. A changed number is written when the box loses focus, asking first the
same way. In read-only mode the Storage rows show the sizes only.
