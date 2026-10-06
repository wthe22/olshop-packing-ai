# 02 — Design Questions

Open questions only. Each one is undecided until the owner chooses.

### D1 — Rules file format
One file holds the categories (ordered) and is edited in the app or on the PC. The condition
inside each category is a tree of AND / OR / NOT. All examples below say the same thing.

- a. **TOML + condition as text.** Easiest to read and edit by hand. Needs a small expression
  parser in the script and in the app (`not` before `and` before `or`, parentheses). Works with
  any TOML 1.0 reader (Python `tomllib`, Java/Kotlin TOML libraries).
  ```toml
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
- b. **TOML + condition as nested tables.** No parser needed. TOML 1.0 keeps each inline table on
  one line, so nested rules become long lines:
  ```toml
  [[category]]
  code = "B"
  name = "Spion & Knalpot"
  when = { all = [ { not = { field = "name", op = "contains", value = "sepatu" } }, { any = [ { field = "name", op = "contains", value = "spion" }, { field = "name", op = "contains", value = "knalpot" } ] } ] }
  ```
- c. **JSON tree** (example in 03-pc-script). No parser needed; built into Python and Kotlin;
  verbose, and one missing comma breaks the file.
- d. **YAML tree.** Nesting reads well; needs a library on both sides (PyYAML, kaml);
  an indentation mistake can change the meaning without an error.

### D2 — Packing list layout
Measured with the prototype on the samples:

| Layout | Batch 1 (956 orders) | Batch 2 (481 orders) |
|---|---|---|
| a. Summary: one row per pack group | 2 pages | 1 page |
| b. Summary + tracking IDs under each group, 6 columns, left→right then down, in label order | 6 pages | 3 pages |
| c. Both available, chosen when printing | — | — |
