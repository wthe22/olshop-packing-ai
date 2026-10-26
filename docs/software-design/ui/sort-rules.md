# Sort rules

Edit the sort rules of `packing-workflows.toml` ([rules file](../rules-file/README.md)) without typing the
filter language: each filter is made from boxes. A text view is there for typing a filter instead.

```
┌ Sort rules ───────────────────────────────────────────────────────────────── [Close] ┐
│  Rule: [default ▾]  [+ New rule]  [Rename…]  [Delete rule]                           │
│  Order matters: an order goes to the first group whose filter matches.               │
│                                                                                      │
│  [A ] [Sepatu           ]                               In batch 3: 130   ↑ ↓ 🗑      │
│   ┌ All of these ▾ ────────────────────────────────────────────────────────────┐     │
│   │ [Product name ▾] [contains ▾] [sepatu          ]  [ ] not  [×]             │     │
│   │ [+ condition] [+ group]                                                    │     │
│   └────────────────────────────────────────────────────────────────────────────┘     │
│   [Text view]                                                                        │
│  [B ] [Spion & Knalpot  ]                               In batch 3:  20   ↑ ↓ 🗑      │
│   ┌ All of these ▾ ────────────────────────────────────────────────────────────┐     │
│   │ [Product name ▾] [contains ▾] [sepatu          ]  [✓] not  [×]             │     │
│   │ ┌ Any of these ▾ ─────────────────────────────────────────────── [ ] not ┐ │     │
│   │ │ [Product name ▾] [contains ▾] [spion           ]  [ ] not  [×]         │ │     │
│   │ │ [Product name ▾] [contains ▾] [knalpot         ]  [ ] not  [×]         │ │     │
│   │ └────────────────────────────────────────────────────────────────────────┘ │     │
│   └────────────────────────────────────────────────────────────────────────────┘     │
│  [Z ] [Lainnya          ]  no filter: takes the rest    In batch 3: 106              │
│  [+ Add group]                             [Edit file as text]   [Cancel]  [Save]    │
└──────────────────────────────────────────────────────────────────────────────────────┘
```

- **The rule** is chosen at the top. **+ New rule** asks for a name and adds a rule with one group `Z
  Lainnya` and no filter, or a copy of the rule shown. **Rename…** asks for a new name: saved batches keep
  the name they recorded, and an amend of one whose name is gone uses its snapshot
  ([D-RULES-2](../decisions/rules.md)). **Delete rule** asks first; the last rule cannot be deleted.
- **Opened with a batch** (the Plan screen's **Edit sort rule**) the screen shows the batch's rule and the
  counts *In batch 3* are its counts. Opened from the title bar it shows the rule the next batch preselects.
- **A group** is a code box, a name box, its filter, the count it takes in the open batch, **↑ ↓** and **🗑**
  (asks first). The last group may have no filter (it takes the rest) and stays last. **+ Add group** adds
  one above it.
- **A box** is **All of these** (*and*) or **Any of these** (*or*), switched with its **▾**; it holds rows and
  other boxes. **+ condition** adds a row, **+ group** a box inside; **×** removes a row, **× group** a box.
  An empty box is an error ("This group is empty").
- **A row** is one comparison, field · operator · value:
  - **Field**: Product name (`name`), Display name, Variation, Seller SKU, Line quantity, Total quantity,
    Different items (`distinct_items`), Courier, Order ID, Tracking ID, Ship by
    ([fields](../rules-file/conditions.md#fields)).
  - **Operator**: text — contains, equals, starts with, and *is unknown* (Courier, Tracking ID) or *is
    empty* (the other text fields), which takes no value; number and date — =, ≠, <, ≤, >, ≥.
  - **Value**: typed without quotes; a whole number for a number field; `2026-10-07` for Ship by; Courier is
    a list of the couriers by name, stored by code. A wrong value is red with the language's message.
- **not** (per row and per box) reverses it: the first row of B reads "name does not contain sepatu".
- **Text view** switches one group between boxes and filter text, checked while typing with line and column;
  **Boxes** switches back once the text has no error. Every valid text can be shown as boxes: nested boxes
  of one kind merge (`a and (b and c)` is one *All of these* box with three rows).
- **Boxes and text agree**: the boxes come from the engine's parser and go back through its printer
  ([program](../app/program.md)). A filter changed in boxes is written in the printer's one-line form; one
  not changed keeps its text exactly, line breaks included.
- **In batch 3**: with a draft open, the count each group takes there, so a change can be judged before
  saving. No draft → no counts.
- **No groups yet**: a rule whose only group has no filter shows **No groups yet: all orders go into one
  PDF** ([D-UI-4](../decisions/ui.md)).
- **Edit file as text** shows the whole of `packing-workflows.toml` (comments included) for larger edits;
  the same checks apply.
- **Save** is possible only when every group is valid. It writes `packing-workflows.toml`, comments kept, and
  the open draft's plan is rebuilt; per-batch changes on the Plan screen are kept where they still apply.
