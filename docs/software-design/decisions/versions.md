# Decisions: data versions

How a copy of the app reads data another version wrote. The schema version itself is
[D-APP-3](app.md); the rule is written in [data › Types, versions and names](../data/README.md#types-versions-and-names).

## D-APP-34 — A copy reads its own major version and passes over what it does not know

**Decision.** A `schema` is `X.Y`. A copy reads a file of its own X whatever its Y, and passes over what it
does not know: a key of a TOML file, a field or a record type of the journal, a column of `orders.csv`. A
change an older copy of the same X handles by passing over it (or another change that is safe for it) keeps
X and may raise Y. A change that needs another way of reading, or that changes what data means (a field's
type or meaning, a record an older copy must apply to be right, a field it needs removed), raises X.

- A file of a **higher X** is not read: the copy says it is of a newer format and asks for this copy to be
  updated. A date whose journal is of a newer X opens with *This date was saved by a newer Packing (v0.9.0,
  journal schema 3.0); this copy reads 2.x. Update this copy to work on it.* and nothing is written to it.
- A file of a **lower X** is read only by the migration the owner starts ([D-APP-33](app.md)).
- A journal record of a type this copy does not know is **passed over**: only its `seq` is checked. The Day
  screen notes *This date holds records of a newer Packing (v0.9.0) that this copy does not know; they are
  kept. Update this copy to see all of it.* The date still takes commits.
- **Nothing is ever lost** but by an action whose purpose is to remove it (a delete, a revert, an amend). What
  a copy does not know stays where it is: the journal is never rewritten; `parser.toml`, `config.toml` and
  `packing-app.toml` are edited key by key, and an edited table (an account, a label group) keeps its other
  keys and the owner's comments; no view is written of a date whose journal holds a record or a field a newer
  copy wrote, none over a `day.toml` of a higher Y (the newer copy writes them again); `.partial/` is not
  settled after a record this copy passed over.
- What a copy does not know in data of **its own version or older** is not expected, so it is named, quietly:
  a key of the three TOML files in Settings › *Data* (*Kept, not read*: the file, the line, the key); a record
  type or field of the journal as a muted note on the Day screen (*Journal lines 2, 5: orders[].colour in
  batch_saved is not known to this version. It is kept and not used.*). The journal's version for this is the
  `app` of the record. Data of a newer version is expected to hold such keys and is not named.
- The journal stays `2.0` for v0.8: the records v0.8 adds (`areas_filled`, `weights_filled`) are passed over
  by a copy that knows this rule. v0.7 and older read only their exact version and call such a record damage;
  what they write, v0.8 and later read.

**Why.** The owner's rules: copies of different versions share a data folder, and data stays readable by every
copy that can read it safely; a copy that cannot says so instead of calling sound data damaged. The program
never drops data silently; what it does not expect, it names without getting in the way.

**Applies to.** [data](../data/README.md#types-versions-and-names), [journal](../data/journal.md),
[records](../data/records.md), [views](../data/views.md), [rules files](../rules-file/README.md#loading),
[messages](../ui/messages.md#opening), [Settings](../ui/settings.md).

## D-APP-35 — `day.toml`, and `orders.csv` names its version in its last column

**Decision.** The view that was `session.toml` is `day.toml`, the glossary's word for one date with its
journal folded. **Update** writes it for a date that still has `session.toml` and deletes the old file.
`orders.csv` carries its schema as its last column: the header `schema:2.1`, every cell of it empty. A reader
finds the columns by their headers and passes over that one; a new column goes before it. `day.toml` and
`orders.csv` share one version, `2.1`.

**Why.** The owner's choice: *session* is a word of v0.1; the CSV tells its version alone and still opens in
Excel as before, the version out of the way at the end.

**Applies to.** [views](../data/views.md), [data](../data/README.md).
