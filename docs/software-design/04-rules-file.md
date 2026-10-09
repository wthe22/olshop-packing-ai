# 04 — Rules File and Condition Language

One file, `categories.toml`, holds the entries. The PC script reads it and uses each entry as one
**pick** (one saved label PDF, [03-pc-script.md](03-pc-script.md)); the app reads it, lets the
owner edit it, and writes it back. The same **condition language** is used for the app's scan
filters.

## File format (TOML)

```toml
# Picks. Order matters: an order is taken by the FIRST entry whose condition matches.
# An entry is one pick = one saved label PDF (03-pc-script). The last entry may have no
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
- File encoding UTF-8. Comments (`#`) are allowed and are kept only by hand editing: the app
  rewrites the file without comments.
- Validation (script and app give the same errors): unknown key, missing `code`/`name`,
  duplicate `code`, `when` missing on an entry that is not last, condition errors (below).
- In the script, a pick whose condition uses a field with no value stops the run
  ([03-pc-script.md](03-pc-script.md)).

## Condition language

### Examples

```
name contains "sepatu"
total_quantity = 1
tracking_id starts_with "JY"
courier starts_with "J&T"
rts_time = "2026-10-06"
paid_time < "14:00"
ship_by < "2026-10-07 17:00"
not name contains "sepatu" and (name contains "spion" or name contains "knalpot")
display_name equals "Spion Beat — Standard, honda" and line_quantity >= 2
```

### Grammar

```
condition  = or_expr
or_expr    = and_expr { "or" and_expr }
and_expr   = not_expr { "and" not_expr }
not_expr   = "not" not_expr | primary
primary    = "(" or_expr ")" | comparison
comparison = text_field text_op string
           | number_field number_op integer
           | time_field number_op time
text_op    = "contains" | "equals" | "starts_with"
number_op  = "=" | "!=" | "<" | "<=" | ">" | ">="
string     = '"' { any character except '"' and '\' | '\"' | '\\' } '"'
integer    = digit { digit }
time       = '"' ( date | clock | date " " clock ) '"'
date       = digit digit digit digit "-" digit digit "-" digit digit
clock      = digit digit ":" digit digit
```

- Keywords, field names and operators are case-insensitive. Spaces and line breaks are free.
- `not` binds tighter than `and`, `and` tighter than `or`:
  `not a and b or c` = `((not a) and b) or c`. Use parentheses when in doubt.
- Text comparison ignores upper/lower case (`honda` = `HONDA`) and trims spaces of the field
  value. It does not ignore accents or other differences.
- Errors give line and column and say what was expected, e.g.
  `line 2, col 18: expected a text operator (contains, equals, starts_with) after "name"`.
  Inside `categories.toml` the line and column are those of the file itself, not of the
  condition text.

### Date and time values

A date/time field is compared with `=`, `!=`, `<`, `<=`, `>`, `>=` and a quoted value. The
value's form decides what is compared:

| Value form | Compares | Example |
|---|---|---|
| `"YYYY-MM-DD"` | The date part (that day) | `rts_time = "2026-10-06"`, `ship_by < "2026-10-07"` |
| `"HH:MM"` | The time of day, any date; seconds ignored | `paid_time < "14:00"` |
| `"YYYY-MM-DD HH:MM"` | That exact moment | `paid_time < "2026-10-06 14:00"` |

A malformed value stops the run with a clear error, e.g.
`line 3, col 22: expected a date "YYYY-MM-DD", a time "HH:MM" or a date and time "YYYY-MM-DD HH:MM"`.
The date is written year-month-day; the time is `HH:MM` without seconds.

### Fields

| Field | Type | Level | Source | Value |
|---|---|---|---|---|
| `name` | text | item | slip, CSV | Full product name (the slip prints it in full; CSV `Product Name`) |
| `display_name` | text | item | — | Display name (name before `\|` + variation) |
| `variation` | text | item | slip, CSV | Variation; the slip's `SKU` column (`Default` reads as empty) |
| `sku_id` | text | item | CSV | `SKU ID`. CSV-only |
| `seller_sku` | text | item | slip, CSV | `Seller SKU` (often empty) |
| `product_category` | text | item | CSV | `Product Category`. CSV-only |
| `line_quantity` | number | item | slip, CSV | Quantity of one line |
| `total_quantity` | number | order | — | Sum of the order's quantities |
| `distinct_items` | number | order | — | Number of lines (different products) of the order |
| `courier` | text | order | label or CSV | `Shipping Provider Name`, or deduced from the label ([03-pc-script.md](03-pc-script.md)) |
| `channel` | text | order | CSV | `Purchase Channel` (TikTok / Tokopedia). CSV-only |
| `tracking_id` | text | order | label, CSV | The tracking ID printed on the label (also the barcode and QR value) |
| `ship_by` | date/time | order | label | `In transit by: dd/mm/yyyy hh:mm` |
| `paid_time` | date/time | order | CSV | `Paid Time`. CSV-only |
| `rts_time` | date/time | order | CSV | `RTS Time`. CSV-only |
| `created_time` | date/time | order | CSV | `Created Time`. CSV-only |
| `category` | text | order | — | Category `code`. **App scan filters only** |
| `batch` | number | order | — | **App scan filters only**; meaning revised with the app |
| `group` | number | order | — | **App scan filters only**; meaning revised with the app |

- `sku_id`, `product_category`, `channel`, `paid_time`, `rts_time` and `created_time` need a
  CSV; a pick using one without a CSV stops the run ([03-pc-script.md](03-pc-script.md)). The
  PC app reads no CSV, so in its rules these fields are always an error
  ([07-pc-app.md](07-pc-app.md#fields-the-conditions-can-use)).
- `product_category` is the platform's category, not the shop's: do not use it in
  `categories.toml`.
- Item fields need item data (a slip or a CSV); a pick using one with neither stops the run.
- Categories may use only the fields without "app scan filters only" (a category cannot depend
  on itself or on numbering that is computed after the categories).

### Item-level fields

An order has several lines. A comparison on an **item** field is true when **at least one line**
matches. So:

- `name contains "spion"` → some line is a spion;
- `not name contains "sepatu"` → no line is a sepatu;
- `name contains "spion" and line_quantity = 2` → some line is a spion **and** some line (maybe
  another one) has quantity 2. Each comparison looks at the lines on its own.

## How the app edits conditions

The app shows a condition as rows of `field · operator · value` inside AND / OR boxes, picked
from lists, so the owner rarely types. It converts that view to and from the text above;
the text is what is stored and exported. A text view is available for long conditions.
Printing a condition always uses lowercase keywords, one space around operators, and
parentheses only where needed, so the same condition always gives the same text.
