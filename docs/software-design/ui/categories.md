# Categories

Edit `categories.toml` (the picks, in order) without typing the condition language: each
condition is made from boxes. A text view is there for typing a condition instead.

```
┌ Categories ───────────────────────────────────────────────────────────────── [Close] ┐
│  Order matters: an order is taken by the first category whose condition matches.     │
│                                                                                      │
│  [A ] [Sepatu           ]                               In batch 3: 130   ↑ ↓ 🗑      │
│   ┌ All of these ▾ ────────────────────────────────────────────────────────────┐     │
│   │ [Product name ▾] [contains ▾] [sepatu          ]  [ ] not  [×]             │     │
│   │ [+ condition] [+ group]                                                    │     │
│   └────────────────────────────────────────────────────────────────────────────┘     │
│   [Text view]                                                                        │
│                                                                                      │
│  [B ] [Spion & Knalpot  ]                               In batch 3:  20   ↑ ↓ 🗑      │
│   ┌ All of these ▾ ────────────────────────────────────────────────────────────┐     │
│   │ [Product name ▾] [contains ▾] [sepatu          ]  [✓] not  [×]             │     │
│   │ ┌ Any of these ▾ ─────────────────────────────────────────────── [ ] not ┐ │     │
│   │ │ [Product name ▾] [contains ▾] [spion           ]  [ ] not  [×]         │ │     │
│   │ │ [Product name ▾] [contains ▾] [knalpot         ]  [ ] not  [×]         │ │     │
│   │ │ [+ condition] [+ group]                                   [× group]    │ │     │
│   │ └────────────────────────────────────────────────────────────────────────┘ │     │
│   │ [+ condition] [+ group]                                                    │     │
│   └────────────────────────────────────────────────────────────────────────────┘     │
│   [Text view]                                                                        │
│                                                                                      │
│  [Z ] [Lainnya          ]  no condition: takes the rest In batch 3: 106              │
│                                                                                      │
│  [+ Add category]                         [Edit file as text]   [Cancel]  [Save]     │
└──────────────────────────────────────────────────────────────────────────────────────┘
```

One category switched to **Text view**, with an error:

```
│  [B ] [Spion & Knalpot  ]                               In batch 3:   ✗   ↑ ↓ 🗑      │
│   ┌────────────────────────────────────────────────────────────────────────────┐     │
│   │ not name contains "sepatu"                                                 │     │
│   │ and (name contains "spion" or name contains "knalpot"                      │     │
│   └────────────────────────────────────────────────────────────────────────────┘     │
│   ✗ line 2, col 54: expected ")"                                                     │
│   [Boxes] (after the error is fixed)   Help: examples ▸                              │
```

- **A category** is a code box, a name box, its condition, the count it takes in the open batch,
  **↑ ↓** to move it and **🗑** to delete it (asks first). The last category has no condition
  (it takes the rest) and stays last. **+ Add category** adds one above it.
- **A group** (box) is either **All of these** (*and*) or **Any of these** (*or*), switched with
  its **▾**. It holds rows and other groups. **+ condition** adds a row, **+ group** a group
  inside; **×** removes a row, **× group** a group with everything in it. An empty group is an
  error ("This group is empty"). Every condition starts as one *All of these* group.
- **A row** is one comparison, field · operator · value:
  - **Field**: the fields the conditions can use ([inputs](../app/inputs.md)),
    with plain names and the field name in small text: Product name (`name`), Display name
    (`display_name`), Variation, Seller SKU, Line quantity, Total quantity, Different items
    (`distinct_items`), Courier, Tracking ID, Ship by.
  - **Operator**: depends on the field. Text: contains, equals, starts with. Number and
    date/time: =, ≠, <, ≤, >, ≥.
  - **Value**: typed, without quotes. Number fields take a whole number; Ship by takes
    `2026-10-07`, `14:00` or `2026-10-07 14:00` ([date and time values](../rules-file/conditions.md)).
    A wrong value is shown red with the message of the [condition language](../rules-file/conditions.md).
- **not** (per row and per group) reverses it: the first row of B above reads "name does not contain
  sepatu".
- **Text view** switches one category between the boxes and its condition as text (the
  [condition language](../rules-file/conditions.md)), checked while typing, errors with line and
  column; **Help: examples ▸** shows its examples. **Boxes** switches back once the text has no error. Every valid text can be
  shown as boxes: nested groups of the same kind are merged (`a and (b and c)` is one *All of
  these* group with three rows), a single comparison is a group with one row.
- **Boxes and text agree**: the boxes are made by the engine's own parser and turned back into
  text by its printer ([the commands between window and Rust](../app/program.md)). A
  category made or changed in boxes is written in the printer's standard one-line form
  (`not name contains "sepatu" and (name contains "spion" or name contains "knalpot")`); a
  category not changed keeps its text exactly as in the file, line breaks included.
- **In batch 3**: when a draft is open, the count each category takes there (in order, from
  what is left), so a change can be judged before saving. No draft → the counts are hidden.
- **No picks yet**: a folder without `categories.toml` shows one catch-all category, no
  condition, that takes every order, and the line **No picks yet: all orders go into one PDF
  set** ([Q-UI-4](../questions/ui.md)). Adding a category above it and pressing **Save** writes
  the file.
- **Edit file as text** shows the whole file as text (comments included) for larger edits; the
  same checks apply.
- **Save** is possible only when every category is valid. It writes `categories.toml` in the
  data folder (comments kept), and the open draft's plan is rebuilt from it. Per-batch changes
  on the Plan screen (skips, added picks) are kept where they still apply.
