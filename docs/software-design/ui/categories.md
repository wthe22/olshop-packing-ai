# Categories

Edit the categories of one **set** ([the rules file](../rules-file/README.md)), in order, without
typing the condition language: each condition is made from boxes. A text view is there for typing a
condition instead.

```
┌ Categories ───────────────────────────────────────────────────────────────── [Close] ┐
│  Set: [default        ▾]   [+ New set]   [Delete set]        file: categories.toml   │
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

- **The set** is chosen at the top of the screen: the set chooser, **+ New set** and **Delete set**.
  A set is a named collection of categories, and a batch is packed with one set, chosen on the
  [New batch](new-batch.md#the-set) screen. The file name of a set is its name: the set `default` is
  the data folder's `categories.toml` (never moved), every other set is
  `<data folder>/categories/<set name>.toml` ([the rules file](../rules-file/README.md)).
  **+ New set** asks for a name and writes `categories/<name>.toml` on **Save**; **Delete set** asks
  first and deletes that set's file, whose categories then fall back to the one catch-all entry.
  The sets and which set a batch uses follow the default of
  [Q-RULES-1](../questions/rules.md) until the owner decides.
- **Opened with a batch** (the Plan screen's **Edit categories**) the screen shows the set that batch
  uses, and the counts *In batch 3* are that set's. Opened from the top bar it shows the set used
  last — the one the next **New batch** preselects.
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
- **No picks yet**: a set with no file shows one catch-all category, no
  condition, that takes every order, and the line **No picks yet: all orders go into one PDF
  set** ([Q-UI-4](../questions/ui.md)). Adding a category above it and pressing **Save** writes
  that set's file.
- **Edit file as text** shows the chosen set's whole file as text (comments included) for larger
  edits; the same checks apply.
- **Save** is possible only when every category is valid. It writes the chosen set's file
  (`categories.toml` for the set `default`, else `categories/<set name>.toml`; comments kept), and
  the open draft's plan is rebuilt from it. Per-batch changes
  on the Plan screen (skips, added picks) are kept where they still apply.
