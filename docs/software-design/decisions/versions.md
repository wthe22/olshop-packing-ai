# Decisions: data versions

How a copy of the app reads data another version wrote. The schema version itself is
[D-APP-3](app.md); the rule is written in [data › Types, versions and names](../data/README.md#types-versions-and-names).

## D-APP-34 — A copy reads its own major version and passes over what it does not know

**Decision.** A `schema` is `X.Y`. A copy reads a file of its own X whatever its Y, and passes over what it
does not know: a key of a TOML file, a column of `orders.csv`. A
change an older copy of the same X handles by passing over it (or another change that is safe for it) keeps
X and may raise Y. A change that needs another way of reading, or that changes what data means (a field's
type or meaning, a field it needs removed), raises X.

- A file of a **higher X** is not read: the copy says it is of a newer format and asks for this copy to be
  updated. A date whose files are of a newer X opens with *This date was saved by a newer Packing (v0.9.0,
  schema 3.0); this copy reads 2.x. Update this copy to work on it.* and nothing is written to it.
- A file of a **lower X** is read only by the migration the owner starts ([D-APP-33](app.md)).
- A key or column a newer copy wrote in a date's files is **passed over** when it reads. A copy writes no date
  whose files hold one, nor any file of a newer Y: the date opens for viewing only, and the Day screen notes
  *This date holds data of a newer Packing (v0.9.0) that this copy does not know. Update this copy to work on it.*
- **Nothing is ever lost** but by an action whose purpose is to remove it (a delete, a revert, an amend). What
  a copy does not know stays where it is: `parser.toml`, `config.toml` and
  `packing-app.toml` are edited key by key, and an edited table (an account, a label group) keeps its other
  keys and the owner's comments.
- What a copy does not know in data of **its own version or older** is not expected, so it is named, quietly:
  a key of the three TOML files in Settings › *Data* (*Kept, not read*: the file, the line, the key); a key or
  column of a date's files as a muted note on the Day screen (*day.toml line 12: label_batches[].colour is not
  known to this version; this date opens for viewing only.*). Data of a newer version is expected to hold such keys and is not named.
- A date with `journal.jsonl` is of v0.7 or older: only the migration the owner starts reads it (D-APP-33).

**Why.** The owner's rules: copies of different versions share a data folder, and data stays readable by every
copy that can read it safely; a copy that cannot says so instead of calling sound data damaged. The program
never drops data silently; what it does not expect, it names without getting in the way.

**Applies to.** [data](../data/README.md#types-versions-and-names), [files](../data/files.md), [save](../data/save.md), [rules files](../rules-file/README.md#loading),
[messages](../ui/messages.md#opening), [Settings](../ui/settings.md).

## D-APP-35 — `day.toml`, and `orders.csv` names its version in its last column

**Decision.** A date's batches are in `day.toml`, the glossary's word for one date, not `session.toml`.
**Update** writes it for a date that still has `session.toml` and deletes the old file.
`orders.csv` carries its schema as its last column: the header `schema:2.1`, every cell of it empty. A reader
finds the columns by their headers and passes over that one; a new column goes before it. Each file has its own version and changes it only when its own shape changes: `day.toml` is `2.1`,
`orders.csv` is `2.1` (no `account`, `courier_clue` or `seller_skus` column), and `order-contents.toml` is `1.1` (an item has its `sku`).

**Why.** The owner's choice: *session* is a word of v0.1; the CSV tells its version alone and still opens in
Excel as before, the version out of the way at the end.

**Applies to.** [files](../data/files.md), [data](../data/README.md).

## D-APP-36 — Update lists what it moves and keeps a copy of what it changes

**Decision.** Before **Update** changes a date, it starts `update v{version}/` in the date folder
(`update v0.8.0-alpha.11/`): `update.toml`, the list of what it does, and copies of files.

- A file Update only **moves or renames** (a label PDF, output or summary of v0.7, `output/.partial/`) is not
  copied: `update.toml` names where it was and where it goes, written before the move. A label PDF two
  batches name is copied to its second name, and the copy is listed. A file in the way that holds other bytes
  is moved into `replaced/` of the update folder, and listed.
- A file whose **content** Update changes or removes (`day.toml`, `session.toml`, `orders.csv`, `order-contents.toml`) is copied into the update folder first and
  checked by its hash; a file it writes where there was none is listed as written. A date with `journal.jsonl`
  has its three files written from it and the journal moves into the update folder ([D-APP-33](app.md)). A second Update by the same version keeps the first
  copies.
- `update.toml` holds the SHA-256 of `day.toml` as Update last left it, and `flattened = true` once a date of
  v0.7 is flat. What v0.7 left that Update does not move stays where it is.
- The owner's `tools/update-backup.py` reverts the Update of one version on every date (the only one there,
  or the one named when there are several): it moves each file
  back, deletes each copy and file it wrote, puts back each file it changed, and renames the folder
  `update v0.8.0-alpha.11 reverted`. It leaves a date whose `day.toml` has changed since (a save after the Update) and
  names it. The app deletes no update folder; the owner runs `update-backup.py delete` once the dates look right, which
  removes `update v…` and `update v… reverted` of that version on every date.

**Why.** The owner's rule: nothing is lost but on purpose. A moved file is not lost, and a list undoes the
move without a second copy of every PDF; a converted file comes back only from its copy.

**Applies to.** [D-APP-32, D-APP-33](app.md), [01 Start](../workflow/01-start-and-open.md),
[Start](../ui/start.md#update-the-data-folder), [development](../../development/README.md).
