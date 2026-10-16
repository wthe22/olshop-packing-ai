# Settings

| Setting | Control | Default |
|---|---|---|
| Data folder | Folder + [Change…] + [Open] | The program's folder ([Start](start.md)) |
| Packing list default | Scope radio + layout ticks | one for the batch · pick |
| About | Version, PDFium version, [Open log file] ([Q-UI-2](../questions/ui.md)) | — |

**Change…** switches the data folder until the app closes; it does not move existing files.
The app asks "Use `<folder>` until the app closes? Existing days stay in `<old folder>`." The
new folder's `categories.toml` (if any) and `couriers.toml` are used; a `couriers.toml` it does
not have is written from the bundled file. The next start looks in the program folder again.
