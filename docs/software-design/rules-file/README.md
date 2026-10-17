# The rules file

The categories of one **grouping**: a named collection of categories that a batch is packed with. One
file per grouping, in the data folder:

| Grouping | File |
|---|---|
| `default` | `<data folder>/categories.toml` — the file the app already writes; it is not moved |
| any other name | `<data folder>/categories/<grouping name>.toml` — the file name is the grouping name |

The grouping a batch uses is chosen when the batch is prepared
([New batch](../ui/new-batch.md#the-grouping)); the groupings are edited in [Categories](../ui/categories.md).
Every entry of a grouping is one **pick** (one saved label PDF, [picks.md](../labels/picks.md)) and the
entries are applied in file order. The **condition language** of the entries is described in
[conditions.md](conditions.md).

Which grouping a batch uses, and where the groupings are kept, are open with the owner
([Q-RULES-1](../questions/rules.md)); the table above and [Groupings](#groupings) are the default the app
follows until the owner decides.

## File format (TOML)

```toml
# Picks. Order matters: an order is taken by the FIRST entry whose condition matches.
# An entry is one pick = one saved label PDF (picks.md). The last entry may have no
# condition and takes the rest.

[[category]]
code = "A"
name = "Sepatu"
when = 'name contains "sepatu"'

[[category]]
code = "B"
name = "Spion & Knalpot"
when = '''
not name contains "sepatu"
and (name contains "spion" or name contains "knalpot")
'''

[[category]]
code = "Z"
name = "Lainnya"
```

| Key | Required | Meaning |
|---|---|---|
| `code` | yes | 1–3 letters or digits, unique. Shown in front of the pick heading and in the saved-PDF file name |
| `name` | yes | Heading text |
| `when` | yes, except on the last entry | Condition text. An entry without `when` matches every order, so it must be last |

- `'…'` is a TOML literal string (no escapes); `'''…'''` spans several lines. Use these so the
  double quotes inside the condition need no escaping.
- File encoding UTF-8. Comments (`#`) are allowed; the app keeps them when it saves.
- Validation: unknown key, missing
  `code`/`name`, duplicate `code`, `when` missing on an entry that is not last, condition errors
  ([conditions.md](conditions.md)).
- In the PC app, a pick whose condition uses a field with no value (a CSV-only field) is an error
  ([inputs.md](../app/inputs.md)).

## Groupings

- **The grouping list** of a data folder: `default` always, plus every `<grouping name>.toml` in its
  `categories/` folder. The names are the file names without `.toml`; characters that are not
  allowed in Windows file names (`\ / : * ? " < > |`) become `-`. A file
  `categories/default.toml` is refused with both files named: the name `default` belongs to the
  data folder's own `categories.toml`, so it is neither created nor deleted nor renamed.
- **A grouping with no file** (a fresh data folder, or a grouping the owner has not saved yet) has one
  catch-all entry — code `Z`, name `Lainnya`, no condition — which takes every order, so a batch
  packed with it saves every order in one PDF set ([inputs](../app/inputs.md)).
- Every grouping is one file, so the owner may copy, rename or edit a grouping by hand. The grouping used last —
  the one a [New batch](../ui/new-batch.md#the-grouping) was last read with —
  is remembered and preselected for the next batch; each batch records its own
  ([day folder](../app/data-folder.md)).
