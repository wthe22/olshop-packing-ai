# 06 — Android App UI

Every screen has four layouts: phone portrait, phone landscape, tablet portrait, tablet
landscape. Width below ~600 dp: one pane. Wider: list and detail side by side.
All texts exist in Indonesian and English (setting: Indonesian / English / System).
Sketches below show English texts and made-up IDs.

## Navigation

Start screen = session list. Inside a session: bottom bar on phone portrait, side rail on
landscape and tablet.

| Tab | Purpose |
|---|---|
| **Scan** | Check parcels with the camera |
| **Orders** | List, search, detail, manual marks |
| **Problems** | Orders marked wrong packing / label lost-damaged / pending |
| **More** | Batches and import, history, session export/rename, settings |

Top bar inside a session: session name, progress `1,102 / 1,433 checked` (removed orders not
counted).

## Sessions (start screen)

- List: name, create time, batches, progress. Buttons: **New session**, **Import session**.
- Per session (menu): Open, Rename, Export, Delete.
- **Delete** always asks: "Delete session *6 Oct 2026*? 1,102 marks will be lost. This cannot be
  undone." [Cancel] [Delete]. During the protected time of today's session the Delete item is
  disabled with the reason: "Today's session can be deleted after 17:00 — change in
  Settings".

## Scan

```
Phone portrait                       Landscape (phone / tablet)
┌──────────────────────────┐        ┌───────────────┬──────────────────────────┐
│ Filter: Sepatu ×1   ▾ ✎  │        │               │ Filter: Sepatu ×1  ▾ ✎   │
│ 12 / 143 checked         │        │    CAMERA     │ 12 / 143 checked         │
├──────────────────────────┤        │           [⟲] ├──────────────────────────┤
│                          │        │               │ ✓ CHECKED  …1234         │
│        CAMERA       [⟲]  │        │               │ 1-01 Sepatu Standar ×1   │
│                          │        │               │ [Wrong packing]          │
├──────────────────────────┤        │               │ [Label lost/damaged]     │
│ ✓ CHECKED   …1234        │        │               │ [Pending]                │
│ 1-01  Sepatu Standar ×1  │        │               ├──────────────────────────┤
│ [Wrong packing] [Label]  │        │               │ Recent marks             │
│ [Pending]                │        │               │ ✓ …1234  1-01     Undo   │
├──────────────────────────┤        │               │ ✓ …5678  1-01     Undo   │
│ Recent marks             │        │               │ …            See all ›   │
│ ✓ …1234 1-01       Undo  │        └───────────────┴──────────────────────────┘
│ ✓ …5678 1-01       Undo  │
│ [⌨ type last digits]     │        [⟲] = switch camera
└──────────────────────────┘
```

- **Scan = checked.** The result card shows tracking ID (last 4 digits large), group, and every
  item with quantity. Three large buttons change the mark: **Wrong packing** ·
  **Label lost/damaged** · **Pending**. No press = stays checked. The buttons act on the order
  on the card until the next scan.
- **Filter bar**: active filter name; ▾ picks a saved filter or "No filter"; ✎ opens the
  condition editor for a one-off change (can be saved as a new filter).
- **Counter**: checked / all active orders that pass the filter.
- **Recent marks**: the last 5 marks (camera and manual), each with Undo; "See all" → History.
- **Type last digits**: keypad; one match → treated like a scan; several → pick from a list.
- Sound and vibration differ per result (success, warning, error), so the screen need not be
  watched for every scan.

### Scan messages

Every result has a colour, a title, and a sentence saying what happened and what to do.
Placeholders in `<…>`.

| Result | Colour | Message |
|---|---|---|
| Checked | green | **Checked.** `<tracking>` · `<group>` · `<items>` |
| Already checked | yellow | **Already checked** at `<time>` (`<camera/manual>`). Nothing changed. If this is a second parcel with the same label, set it aside. |
| Has a problem mark | yellow | **Marked *`<mark>`* at `<time>`.** Process this scan (mark checked) or cancel? [Process] [Cancel] |
| Filter rejected | orange | **Not for this stack.** `<tracking>` has `<field>` `<value>`; the filter needs `<condition part>`. Not marked. |
| Removed order | grey | **No longer to send.** `<tracking>` was not in the orders CSV of batch `<n>` (`<time>`): cancelled or already picked up. Not marked; no action needed. |
| Unknown order | red | **Unknown order.** `<tracking>` is not in this session. It may be from another day, or its batch is not imported yet (newest: batch `<n>`, `<time>`). |
| Not a label code | red | **Not a label code.** Read "`<value>`" — not a tracking ID or Order ID. Aim at the label's QR code or barcode. |
| Two labels in view | red | **Two labels in view.** Show one label at a time. Nothing marked. |
| No camera permission | red | **No camera access.** Allow the camera in Android settings. [Open settings] |
| Camera unavailable | red | **Camera busy or not found.** Close other apps using the camera, or switch camera. [Switch camera] |
| No session data | grey | **No orders yet.** Import the orders CSV first (More › Batches › Import). |

## Condition editor (scan filters and categories)

```
┌ All of these (AND) ─────────────────────────── ▾ ┐
│ [Product name ▾] [contains ▾] [sepatu      ] [×] │
│ [Total quantity ▾] [= ▾] [1] [×]                 │
│ ┌ Any of these (OR) ───────────────────── ▾ ─┐   │
│ │ [Product name ▾] [contains ▾] [spion ] [×] │   │
│ │ [Product name ▾] [contains ▾] [knalpot] [×]│   │
│ │ [+ condition] [+ group]                    │   │
│ └────────────────────────────────────────────┘   │
│ [+ condition] [+ group]                          │
└──────────────────────────────────────────────────┘
Matches 143 of 1,433 orders in this session      [Text view]
```

- Each row: field · operator · value; `NOT` toggle per row and per box.
- Value suggestions from the session (product names, variations, couriers, categories).
- Live count of matching orders, so a wrong filter is seen before scanning.
- **Text view** shows and edits the same condition as text (04-rules-file); errors are shown
  with the line and column.

## Orders

```
Phone portrait                          Tablet landscape
┌──────────────────────────┐           ┌──────────────────────┬─────────────────────┐
│ 🔍 search                │           │ 🔍 search            │ JY0000001234        │
│ [Unchecked][Checked]     │           │ status chips         │ Order 5800…0001     │
│ [Problems][Removed][All] │           │ Cat ▾ Batch ▾ Group ▾│ Group 1-01 · J&T    │
├──────────────────────────┤           ├──────────────────────┤ Items               │
│ …1234  1-01  ×1    ✓     │           │ …1234 1-01 ×1   ✓    │  Sepatu Standar ×1  │
│ Sepatu Standar Samping   │           │ …5678 1-06 ×1   ✗    │ Buyer message: —    │
│ …5678  1-06  ×1    ✗     │           │ …9012 2-02 ×2   ·    │ History             │
│ Spion Beat — Standard    │           │ …                    │  10:31 ✓ camera Undo│
│ …                        │           │                      │ [✓][Wrong][Label][⏸]│
└──────────────────────────┘           └──────────────────────┴─────────────────────┘
  tap → full-screen detail
```

- Row: last 4 digits of the tracking ID in bold, group (`batch-group`), total quantity, status
  icon, first item's display name.
- Default filter: Unchecked. Removed orders appear only under Removed / All.
- Grouped view (toggle): rows by batch → category → pack group, each with its progress.
- Detail: full tracking ID and Order ID, batch, group, category, courier, items (display name,
  quantity, Seller SKU small), buyer message, seller note, history of marks with Undo, the four
  mark buttons.

## Problems

Three sections — Wrong packing · Label lost/damaged · Pending — each row: full tracking ID and
Order ID in large text, group, items, mark time. Tap → order detail (scan or mark checked
after fixing, or undo).

## More

- **Batches**: list (number, import time, orders, groups, removed, ignored rows).
  **Import orders CSV** → file picker → summary:

  ```
  Import orders-06-2.csv as batch 2?
    New orders        481   in 20 groups
    Removed             2   (not in this CSV any more)
    Back again          0
    Ignored rows        0   (other status)
    Warnings            0
  Compare with the packing list: "481 orders · 20 groups"
                                          [Cancel] [Apply import]
  ```
  Warnings (changed orders) are listed with tracking ID and what changed.
- **History**: every scan and mark, newest first; filter by result; Undo on marks.
- **Session**: Rename, Export `.zip` (Android share sheet or save to a folder).
- **Settings**:
  - Categories: ordered list (drag to reorder), edit with the condition editor;
    **Import categories.toml** / **Export categories.toml**. Changes affect batches imported
    afterwards only.
  - Scan filters: saved list; add, rename, edit, delete.
  - Language: System / English / Indonesia.
  - Delete protection: on/off and time (default 17:00).
  - Sound, vibration.
  - **Export settings** / **Import settings** (`packing-settings.zip`).
