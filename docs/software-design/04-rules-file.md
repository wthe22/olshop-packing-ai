# 04 — Rules File and Condition Language

One file, `categories.toml`, holds the categories. The PC script reads it; the app reads it,
lets the owner edit it, and writes it back. The same **condition language** is used for the
app's scan filters.

## File format (TOML)

```toml
# Packing categories. Order matters: an order goes to the FIRST category whose
# condition matches. The last category has no condition and catches the rest.

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
code = "C"
name = "Motor lainnya"
when = 'product_category contains "sepeda motor"'

[[category]]
code = "Z"
name = "Lainnya"
```

| Key | Required | Meaning |
|---|---|---|
| `code` | yes | 1–3 letters or digits, unique. Shown in front of the category heading |
| `name` | yes | Heading text |
| `when` | yes, except on the last category | Condition text. A category without `when` matches every order, so it must be last |

- `'…'` is a TOML literal string (no escapes); `'''…'''` spans several lines. Use these so
  the double quotes inside the condition need no escaping.
- File encoding UTF-8. Comments (`#`) are allowed and are kept only by hand editing: the app
  rewrites the file without comments.
- Validation (script and app give the same errors): unknown key, missing `code`/`name`,
  duplicate `code`, `when` missing on a category that is not last, condition errors (below).

## Condition language

### Examples

```
name contains "sepatu"
total_quantity = 1
not name contains "sepatu" and (name contains "spion" or name contains "knalpot")
display_name equals "Spion Beat — Standard, honda" and line_quantity >= 2
courier starts_with "J&T"
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
text_op    = "contains" | "equals" | "starts_with"
number_op  = "=" | "!=" | "<" | "<=" | ">" | ">="
string     = '"' { any character except '"' and '\' | '\"' | '\\' } '"'
integer    = digit { digit }
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

### Fields

| Field | Type | Level | Value |
|---|---|---|---|
| `name` | text | item | Full `Product Name` |
| `display_name` | text | item | Display name (name before `\|` + variation) |
| `variation` | text | item | `Variation` (`Default` reads as empty) |
| `sku_id` | text | item | `SKU ID` |
| `seller_sku` | text | item | `Seller SKU` (often empty) |
| `product_category` | text | item | `Product Category` |
| `line_quantity` | number | item | `Quantity` of one line |
| `total_quantity` | number | order | Sum of the order's quantities |
| `distinct_items` | number | order | Number of lines (different SKUs) of the order |
| `courier` | text | order | `Shipping Provider Name` |
| `channel` | text | order | `Purchase Channel` (TikTok / Tokopedia) |
| `category` | text | order | Category `code`. **App scan filters only** |
| `batch` | number | order | Batch number. **App scan filters only** |
| `group` | number | order | Pack-group number inside its batch. **App scan filters only** |

Categories may use only the fields without "app scan filters only" (a category cannot depend
on itself or on numbering that is computed after categories).

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
