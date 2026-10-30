# Changing the last batch or the sort rules

## Revert and amend the last batch

Only the newest batch of a date can be changed — a later batch depends on the ones before it.

- **Revert** — undo the last batch. It asks: "Undo batch 2? Its 2 group PDFs and summary PDF are
  deleted. Throw away its printed labels." [Undo batch 2] [Cancel]. Its orders count as not saved
  again and the next batch takes its number. If a file is open in a PDF
  viewer, the app names the file to close and changes nothing.
- **Amend** — "I fixed a sort rule, do the last batch again". It asks: "Redo batch 2? Its labels
  are read again with the sort rule default." [Redo batch 2] [Cancel]. The batch's own label files are
  read again from the date's `input` folder, with the sort rule of that name as it is now (or as it was
  saved, when no rule has that name any more), the same copies of repeated orders kept, and the **Plan**
  screen opens titled *Redo batch 2*; **Save batch** writes it again. Print it again after saving.
  - **Back** (top) shows the batch's files, its rule fixed ("Sort rule: default (as now)"); add files
    with **Add files…**, take one off with **×** (its orders stay unsaved for a later batch), then
    **Read labels** again.
  - **Cancel amend** or **Cancel** on the file list leaves the batch as it was, and asks nothing.
  - The batch written again starts with its printed ticks cleared.

## Sort rules

![The Sort rules screen](images/07-sort-rules.png)

**Sort rules** in the title bar, or **Edit sort rule** on the Plan screen, opens the sort rules of
`packing-workflows.toml`, the file beside the program. A sort rule is a list of label groups in order: an
order goes to the first group whose filter matches; the last group has no filter and takes the rest.

- **The tabs** at the top are your rules; click one to show it (from the Plan screen it opens on the
  batch's rule). *last used* marks the rule of your newest batch, the one **New batch** starts on; *off*
  marks a rule that is switched off. **+ New rule** asks for a name and starts with one group
  *Z Lainnya*, or as *a copy of* the rule shown.
- **The switch** at the top of the rule: **On**, New batch offers the rule; **Off**, the rule is kept as
  it is but New batch does not list it. Switch it on again whenever it is needed; a batch saved with it
  can still be amended. One rule always stays on.
- **Rename…** changes the name in place; saved batches keep the name they were saved with. **Delete
  rule** asks first; the last rule, and the only rule that is on, stay.
- Each group has a **code** (1–3 letters or digits), a **name** and a filter made of boxes: a box is
  **All of these** (*and*) or **Any of these** (*or*) and holds rows of *field · operator · value* and
  boxes inside; **not** reverses a row or a box. **Courier · equals** lists the couriers by name;
  **Courier · contains** takes typed text and matches the names (`J&T` takes J&T Express and J&T Cargo).
- **Courier · is unknown** takes the orders whose courier the app could not name, and **Tracking ID · is
  unknown** those without a tracking ID (instant-service labels); with **not**, the others. *Is empty* does
  the same on the other text fields.
- **In batch 3** is the count each group takes in the batch being prepared, so a change can be judged
  before saving.
- **Boxes | Text** beside a group switches its filter between the boxes and typed text; **Help:
  examples ▸** under the text lists examples. **Edit file as text** shows the whole file, comments
  included.
- **Save** writes the file, keeping your comments and every filter you did not change; the batch being
  prepared follows the new rule, and its skips and file names stay where they still apply. *Unsaved
  changes* beside it says there is something to save; another tab asks first before it drops them.

Four example filters (more in [the rules file](../software-design/rules-file/conditions.md)):

| You want | Filter |
|---|---|
| everything with "sepatu" in the product name | `name contains "sepatu"` |
| orders of 3 items or more | `total_quantity >= 3` |
| everything sent by J&T Express | `courier equals "jnt"` |
| everything sent by any J&T courier | `courier contains "J&T"` |
| spion or knalpot, but not sepatu | `not name contains "sepatu" and (name contains "spion" or name contains "knalpot")` |
