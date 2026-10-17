# Settings

| Setting | Control | Default |
|---|---|---|
| Data folder | Folder + [Change…] + [Open] | The program's folder ([Start](start.md)) |
| Packing list default | Scope radio + layout ticks | one for the batch · pick |
| About | Version v0.1, PDFium version, [Open log file] ([Q-UI-2](../questions/ui.md)) | — |

**Change…** switches the data folder until the app closes; it does not move existing files.
The app asks "Use `<folder>` until the app closes? Existing days stay in `<old folder>`." The
new folder's groupings ([rules file](../rules-file/README.md)) and its `couriers.toml` are used; a
`couriers.toml` it does not have is written from the bundled file. The next start looks in the
program folder again.

## The version

The version of the program is the workspace `version` in the root `Cargo.toml` — `0.1.0` today — and
that is its **one place**. The shown form is `v` + the first two numbers (`0.1.0` → `v0.1`), the
owner's form. It appears in three spots, all fed from that one place: the window title, the title bar
of every screen (the same line, [the window frame](README.md#window-frame)) and *About*.

- `app/src-tauri/tauri.conf.json` carries no `version` and no number in its window `title`. Tauri then
  takes the version from `app/src-tauri/Cargo.toml`, which the app inherits from the workspace, so no
  second copy exists that could disagree. `app/package.json`'s own `version` is npm's field: nothing
  in the app shows it, and it is kept equal to the workspace `version`.
- The `v` + major.minor text is made in one place in the app crate, so the three spots can never
  differ. Changing the workspace `version` (e.g. to `0.2.0` for the next release) changes all three
  on the next build.
