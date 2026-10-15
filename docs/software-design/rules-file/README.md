# The rules file

One file, `categories.toml`, holds the entries. The PC app reads it, uses each entry as one
**pick** (one saved label PDF, [picks.md](../labels/picks.md)) and lets the owner edit it. The
**condition language** of its entries is described in [conditions.md](conditions.md).

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
