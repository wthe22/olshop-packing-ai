# Settings

| Setting | Control |
|---|---|
| Data folder | The folder of `packing-settings.toml` + [Change…] + [Open] |
| About | The program's name, version and what it does; then Version (`v0.2`), PDFium and Journal schema (`1.0`) |

The theme is the title bar's menu ([window frame](README.md#window-frame)). There are no other settings:
every other choice has one fixed default, shown where it acts
([D-APP-16](../decisions/app-features.md)). There is no log file: the journal of each date is the record of what was
done ([D-UI-2](../decisions/ui.md)).

**Change…** chooses another folder and asks "Use `<folder>` for the data from now on? The data in
`<old folder>` stays where it is." It writes `packing-settings.toml` (a path inside the program's folder is
written relative to it), releases the old folder's lock, takes the new one's, and opens it as at start
([program › Start](../app/program.md#start)). A draft being prepared asks first. Nothing is moved.

## The version

The version of the program is the workspace `version` in the root `Cargo.toml`, and that is its **one
place**. The shown form is `v` + the first two numbers (`0.2.0` → `v0.2`). It appears in the window title,
the title bar of every screen and *About*, all fed from that one place.

- `app/src-tauri/tauri.conf.json` carries no `version` and no number in its window `title`; Tauri takes the
  version from `app/src-tauri/Cargo.toml`, which inherits the workspace's. `app/package.json`'s own
  `version` is npm's field: nothing shows it, and it is kept equal to the workspace's.
- The `v` + major.minor text is made in one place in the app crate. The journal's `app` field is the full
  workspace version (`0.2.0`).
