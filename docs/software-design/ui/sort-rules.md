# Sort rules

Edit the sort rules of `packing-workflows.toml` ([rules file](../rules-file/README.md)) without typing the
filter language: each filter is made from boxes. A text view is there for typing a filter instead.

```
┌ Sort rules ───────────────────────────────────────────────────────────────────────── × ┐
│ ┌─────────┬────────────────────┬───────────────────┐                                    │
│ │ default │ harian  last used  │ promo 11.11  off  │  + New rule                        │
│ └─────────┴────────────────────┴───────────────────┘                                    │
│ ┌────────────────────────────────────────────────────────────────────────────────────┐ │
│ │ (●━) On  New batch offers this rule.                       [Rename…]  [Delete rule] │ │
│ │ GROUP  Order matters: an order goes to the first group …            IN BATCH 3      │ │
│ ├────────────────────────────────────────────────────────────────────────────────────┤ │
│ │ [A] [Sepatu          ] [Boxes|Text]                                    130   ↑ ↓ 🗑 │ │
│ │      All of these ▾  [ ] not                                                        │ │
│ │      [Product name ▾] [contains ▾] [sepatu              ]  [ ] not  ×               │ │
│ │      [+ condition] [+ group]                                                        │ │
│ ├────────────────────────────────────────────────────────────────────────────────────┤ │
│ │ [B] [Spion & Knalpot ] [Boxes|Text]                                     20   ↑ ↓ 🗑 │ │
│ │      All of these ▾ … (a box inside a box: Any of these ▾ …)                        │ │
│ ├────────────────────────────────────────────────────────────────────────────────────┤ │
│ │ [Z] [Lainnya         ]                                                 106   ↑ ↓ 🗑 │ │
│ │      [✓] No filter: takes the rest                                                  │ │
│ ├────────────────────────────────────────────────────────────────────────────────────┤ │
│ │ + Add group                                                                         │ │
│ └────────────────────────────────────────────────────────────────────────────────────┘ │
│ Edit file as text                                 Unsaved changes  [Cancel]  [Save]    │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

## The rules

- **The tabs** at the top are the rules of the file, in file order, side by side as Settings' theme
  choice; the rule shown is raised. The rule of the workspace's newest batch carries *last used*; a rule
  that is switched off carries *off* and its name is faint. **+ New rule** asks for a name in a line
  under the tabs and adds a rule with one group `Z Lainnya` and no filter, or *a copy of* the rule shown;
  a new rule and a copy start on.
- **Opened with a batch** (the Plan screen's **Edit sort rule**) the screen shows the batch's rule and the
  counts are its counts. Opened from the title bar it shows the rule the next batch preselects.
- **Another tab**, or **+ New rule**, while the rule shown has unsaved changes asks first: *Discard the
  changes to harian?* [Discard] [Cancel].

## The rule's card

The rule shown is one card: its head is the rule, its rows the groups.

- **The switch** in the head: **On** — *New batch offers this rule.* — or **Off** — *Kept, but New batch
  does not offer it.* It writes the file at once, and leaves the groups being edited as they are. The
  only rule that is on cannot be switched off, so New batch always has one
  ([D-RULES-5](../decisions/rules.md)); its switch is greyed with that reason as its tooltip.
- **Rename…** turns the head into the name, edited in place, with **Rename** and **Cancel** (Enter and
  Esc); the groups being edited stay. Saved batches keep the name they recorded, and an amend of one whose
  name is gone uses its snapshot ([D-RULES-2](../decisions/rules.md)).
- **Delete rule** asks first. The last rule cannot be deleted, nor the only rule that is on: the button
  is greyed with the reason as its tooltip.
- **The column head** reads *Group*, the hint on order, and *In batch 3* over the counts while a draft
  is open. No draft → no counts.

## A group

- **A row** of the card is a group: its code (drawn as the blue group chip), its name, **Boxes | Text**
  for its filter, the count it takes in the open batch, and **↑ ↓** and **🗑** (asks first). The filter
  stands under the name, no wider than a line can be read in one glance (58 rem). The last
  group may have no filter: its tick **No filter: takes the rest** replaces the filter, and it stays last.
  **+ Add group** at the foot of the card adds one above it.
- **A box** is **All of these** (*and*) or **Any of these** (*or*), switched with its **▾**; it holds rows and
  other boxes. **+ condition** adds a row, **+ group** a box inside; **×** removes a row, **× group** a box.
  An empty box is an error ("This group is empty").
- **A condition** is one comparison, field · operator · value:
  - **Field**: Product name (`name`), Display name, Variation, Seller SKU, Line quantity, Total quantity,
    Different items (`distinct_items`), Courier, Order ID, Tracking ID, Ship by
    ([fields](../rules-file/conditions.md#fields)).
  - **Operator**: text — contains, equals, starts with, and *is unknown* (Courier, Tracking ID) or *is
    empty* (the other text fields), which takes no value; number and date — =, ≠, <, ≤, >, ≥.
  - **Value**: typed without quotes; a whole number for a number field; `2026-10-07` for Ship by. Courier
    *equals* is a list of the couriers by name, stored by code; Courier *contains* and *starts with* take
    typed text, matched against the couriers' names (`J&T` takes J&T Express and J&T Cargo,
    [D-RULES-4](../decisions/rules.md)). Switching between them carries the value over: a picked courier
    becomes its name, typed text picks the first courier whose name holds it. A wrong value is red with the
    language's message.
- **not** (per row and per box) reverses it: a row of B reads "name does not contain sepatu".
- **Text** switches one group to its filter as text, checked while typing with line and column, with
  **Help: examples ▸** under it; **Boxes** switches back once the text has no error. Every valid text can be
  shown as boxes: nested boxes of one kind merge (`a and (b and c)` is one *All of these* box with three
  rows).
- **Boxes and text agree**: the boxes come from the engine's parser and go back through its printer
  ([program](../app/program.md)). A filter changed in boxes is written in the printer's one-line form; one
  not changed keeps its text exactly, line breaks included.
- **No groups yet**: a rule whose only group has no filter shows **No groups yet: all orders go into one
  PDF** under the column head ([D-UI-4](../decisions/ui.md)).

## The foot

The foot stays at the bottom of the screen, in view while the groups scroll.

- **Edit file as text** shows the whole of `packing-workflows.toml` (comments included) for larger edits;
  the same checks apply, and the foot's **Cancel** returns to the card.
- *Unsaved changes* shows once a group differs from the file. **Save** is possible only then, and only
  when every group is valid. It writes `packing-workflows.toml`, comments kept, and the open draft's plan
  is rebuilt; per-batch changes on the Plan screen are kept where they still apply. **Cancel** (and the
  **×**, a click beside the screen, `Esc`) closes without saving.
