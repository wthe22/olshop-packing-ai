# 2.20 One hash for everything: XXH3-64 for the label-file fingerprint

The label-file fingerprint ("used in batch n", amend) is size + SHA-256 (`sha2`); it becomes size +
**XXH3-64 of the whole file** (16 hex characters, streamed), so `xxhash-rust` — already in for the
logo hash — is the only hash crate and `sha2` leaves the workspace.
`state.json` entries carry `xxh3` instead of `sha256`
([data-folder.md](../../software-design/app/data-folder.md)); a day folder whose entries carry
`sha256` still loads — `sha256` is read and never matched, since the "used twice" hint only matters
inside one day.

- **Code:** `engine/src/day.rs`, `pdf/src/batch.rs`, `app/src-tauri/src/commands/day.rs`, the `sha2`
  lines of the `Cargo.toml` files, and `engine/tests/day_state.rs`. Not changed: the SHA-256 that
  `tools/get-pdfium.sh` checks against the PDFium release — the vendor publishes it, it is a download
  check and not a hash the program makes. Design is in
  [inputs.md](../../software-design/app/inputs.md),
  [data-folder.md](../../software-design/app/data-folder.md) and
  [architecture.md](../../software-design/app/architecture.md).
- **Tests:** the same file is recognised twice; a file with one changed byte is not; a day file with
  `sha256` entries loads.
- **Result:** (not started)
