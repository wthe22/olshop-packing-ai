# 2.19 The courier data in the data folder

The courier list — name, text clues, tracking forms — lives in the program, so a new courier, a
corrected form or the two-clue rule needs a new program version. It becomes a data file like
`categories.toml` (same handling): the app writes the bundled file into the data folder on first
start, *Start* and *Settings › Change…* treat it as they treat `categories.toml`, and a broken file
stops the read with its line. A courier change becomes an edit the owner can make, and the shipped
default carries the bundled list into a fresh data folder
([D-LABEL-1](../../software-design/decisions/label.md)).

- **Code:** the loader in `crates/engine/src/label.rs` reads the data folder's file (the bundled copy
  as the default), the first-start write and the Settings/Start handling next to `categories.toml`,
  the broken-file message, and the write-back of a hash taught in the logo prompt (comments kept). Design is in [inputs.md](../../software-design/app/inputs.md),
  [data-folder.md](../../software-design/app/data-folder.md),
  [program.md](../../software-design/app/program.md) and
  [start.md](../../software-design/ui/start.md) / [settings.md](../../software-design/ui/settings.md).
- **Result:** (not started)
