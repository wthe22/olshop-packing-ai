# UI: the owner's feedback

The owner's findings on the window and the screens. Open items only; see [README](README.md) for the form.

## R-UI-1 — The version in the title and in About

**Now:** the window title is *Packing*. The *About* row of Settings shows *Version 0.1.0*, the workspace
version in `Cargo.toml`.

**Wanted:** the program is about to be distributed, so the version shows in both places. This release is
**v0.1**: the window title reads *Packing v0.1*, and *About* reads *Version v0.1*. The number is kept in one
place only. The exact form of the number (`v0.1` or `0.1.0`) is for the next session to settle in
[software-design/](../software-design/README.md), with the owner's *v0.1* as the default.
