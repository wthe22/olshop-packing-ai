# Filter language

A label group's `filter`, a one-off group of the Plan screen and a search are written in this language.

## Examples

```
name contains "sepatu"
total_quantity = 1
tracking_id starts_with "JY"
courier equals "jnt"
courier is_empty
not tracking_id is_empty
ship_by < "2026-10-08"
not name contains "sepatu" and (name contains "spion" or name contains "knalpot")
display_name equals "Spion Beat — Standard, honda" and line_quantity >= 2
```

## Grammar

```
filter     = or_expr
or_expr    = and_expr { "or" and_expr }
and_expr   = not_expr { "and" not_expr }
not_expr   = "not" not_expr | primary
primary    = "(" or_expr ")" | comparison
comparison = text_field text_op string
           | text_field "is_empty"
           | number_field number_op integer
           | date_field number_op date
text_op    = "contains" | "equals" | "starts_with"
number_op  = "=" | "!=" | "<" | "<=" | ">" | ">="
string     = '"' { any character except '"' and '\' | '\"' | '\\' } '"'
integer    = digit { digit }
date       = '"' digit digit digit digit "-" digit digit "-" digit digit '"'
```

- Keywords, field names and operators are case-insensitive. Spaces and line breaks are free.
- `not` binds tighter than `and`, `and` tighter than `or`: `not a and b or c` = `((not a) and b) or c`.
- Text comparison ignores upper and lower case (`honda` = `HONDA`) and trims spaces of the field value. It
  does not ignore accents.
- `is_empty` is true when the field has no value; spaces count as none. On an item field it is true when
  one line has none (`seller_sku is_empty`). It is how a filter takes the orders whose courier was not
  named (`courier is_empty`) or that have no tracking ID ([D-RULES-3](../decisions/rules.md)); a
  `courier equals` value must be a courier's code.
- A date is written year-month-day ([D-APP-21](../decisions/app-features.md)).
- Errors give line and column and say what is expected, e.g.
  `line 2, col 18: expected a text operator (contains, equals, starts_with, is_empty) after "name"`. Inside
  `packing-workflows.toml` the line and column are those of the file, not of the filter text.

## Fields

| Field | Type | Level | Source | Value |
|---|---|---|---|---|
| `name` | text | item | slip | Full product name |
| `display_name` | text | item | — | Product name up to the first `\|`, trimmed; then ` — ` and the variation, which is left out when empty |
| `variation` | text | item | slip | The variation (empty for `Default`) |
| `seller_sku` | text | item | slip | The Seller SKU; often empty |
| `line_quantity` | number | item | slip | Quantity of one line |
| `total_quantity` | number | order | slip | The printed `Qty Total`; none on the Shopee-style slip, so a comparison on it is false there |
| `distinct_items` | number | order | — | Number of lines of the order |
| `courier` | text | order | label | The courier's **code** in `packing-workflows.toml` (`jnt`); empty when no courier is named ([courier](../labels/courier.md)) |
| `order_id` | text | order | label | The Order ID |
| `tracking_id` | text | order | label | The tracking ID (also the barcode and QR value) |
| `ship_by` | date | order | label | `ship_by_date` ([reading](../labels/reading.md)) |

A field not in this table is an error with line and column.

## Item-level fields

An order has several lines. A comparison on an **item** field is true when **at least one line** matches:

- `name contains "spion"` → some line is a spion;
- `not name contains "sepatu"` → no line is a sepatu;
- `name contains "spion" and line_quantity = 2` → some line is a spion **and** some line (maybe another
  one) has quantity 2. Each comparison looks at the lines on its own.

## How the app edits filters

The app shows a filter as rows of `field · operator · value` inside AND / OR boxes, picked from lists, so
the owner rarely types ([Sort rules](../ui/sort-rules.md)); a courier is picked by name and stored by code.
The text above is what is stored. Printing a filter always uses lowercase keywords, one space around
operators and parentheses only where needed, so the same filter always gives the same text.
