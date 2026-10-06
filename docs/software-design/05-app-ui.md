# 05 — Android App UI

Four layouts per screen: phone portrait, phone landscape, tablet portrait, tablet landscape.
Below ~600 dp width: one pane; wider: list + detail side by side.
All texts in Indonesian and English (language setting: Indonesian / English / System).

## Navigation

Inside a session: bottom bar on phone portrait, side rail on landscape and tablet.

| Tab | Purpose |
|---|---|
| **Scan** | Check parcels |
| **Orders** | List, search, detail, manual marks |
| **Problems** | Orders marked wrong packing / label lost-damaged / pending |
| **More** | Batches and import, history, session export, settings |

Top bar: session name, progress `1,102 / 1,433 checked` (removed orders not counted).

## Sessions (start screen)

List: name, create time, batches, progress. Actions: new, open, rename, export, import, delete.
Delete asks for confirmation; during the protected time of today's session the delete button is
disabled with the reason ("Today's session can be deleted after 17:00 — change in Settings").

## Scan

```
Phone portrait                       Landscape (phone / tablet)
┌──────────────────────────┐        ┌───────────────┬──────────────────────────┐
│ Filter: [sepatu ×]       │        │               │ Filter chips  [+]        │
│ [total qty = 1 ×] [+]    │        │    CAMERA     │ 12 / 143 checked         │
│ 12 / 143 checked         │        │           [⟲] ├──────────────────────────┤
├──────────────────────────┤        │               │ ✓ CHECKED  …1234         │
│                          │        │               │ 1-01 Sepatu ×1           │
│        CAMERA       [⟲]  │        │               │ [Wrong packing]          │
│                          │        │               │ [Label lost/damaged]     │
├──────────────────────────┤        │               │ [Pending]                │
│ ✓ CHECKED   …1234        │        │               ├──────────────────────────┤
│ 1-01  Sepatu ×1          │        │               │ Recent marks (5)         │
│ [Wrong packing] [Label]  │        │               │ ✓ …1234  1-01     Undo   │
│ [Pending]                │        │               │ ✓ …5678  1-01     Undo   │
├──────────────────────────┤        │               │ …                        │
│ Recent marks             │        └───────────────┴──────────────────────────┘
│ ✓ …1234 1-01       Undo  │
│ ✓ …5678 1-01       Undo  │        [⟲] = switch camera
│ [type last digits…]      │
└──────────────────────────┘
```

- **Scan = checked.** The result card shows the order's items and three large buttons to change
  the mark: **Wrong packing** · **Label lost/damaged** · **Pending**. No press = stays checked.
  The buttons apply to the order on the card until the next scan.
- **Filter chips**: tap to edit, × to remove, `+` adds a condition or picks a saved filter.
  The editor builds AND/OR groups with dropdowns; values come from lists where possible
  (product names, categories, groups of the session), so little typing.
- **Counter** = checked / total orders passing the filter.
- **Recent marks**: the last 5 marks, each with Undo; "See all" opens the scan history.
- **Typed input**: last digits of the tracking ID; several matches → pick from a list.

### Scan messages

Every result has a colour, a sound, a title and a sentence that says what happened and what
to do.

| Result | Colour | Message (example) |
|---|---|---|
| Marked checked | green | **Checked.** `JY…1234` · 1-01 · Sepatu Standar Samping Motor ×1 |
| Already checked | yellow | **Already checked** at 10:31 (camera). Nothing changed. If this is a second parcel with the same label, set it aside. |
| Has a problem mark | yellow | **Marked *wrong packing* at 10:20.** Process this scan (mark checked) or cancel? [Process] [Cancel] |
| Filter rejected | orange | **Not for this stack.** `…5678` has total quantity 2; the filter needs 1. Not marked. |
| Removed order | grey | **No longer to send.** `…5678` was not in the orders CSV of batch 3 (15:10): cancelled or already picked up. Not marked; no action needed. |
| Not in session | red | **Unknown order.** `JY…9012` is not in this session. It may be from another day, or its batch is not imported yet (newest: batch 2, 16:29). |
| Not a label code | red | **Not a label code.** Read "`XYZ…`" — not a tracking ID or Order ID. Aim at the label's QR code or barcode. |
| Two labels in view | red | **Two labels in view.** Show one label at a time. Nothing marked. |
| Camera permission refused | red | **No camera access.** Allow the camera in Android settings. [Open settings] |
| Camera unavailable | red | **Camera busy or not found.** Close other apps using the camera, or switch camera. [Switch] |

## Orders

```
Phone portrait                          Tablet landscape
┌──────────────────────────┐           ┌──────────────────────┬─────────────────────┐
│ 🔍 search                │           │ 🔍 search            │ JY0000001234        │
│ [Unchecked][Checked]     │           │ status chips         │ Order 5800…0001     │
│ [Problems][Removed][All] │           │ Cat ▾ Group ▾ Batch ▾│ Group 1-01 · J&T    │
├──────────────────────────┤           ├──────────────────────┤ Items               │
│ …1234  1-01  ×1    ✓     │           │ …1234 1-01 ×1   ✓    │  Sepatu ×1          │
│ Sepatu Standar Samping   │           │ …5678 1-06 ×1   ✗    │ Buyer message: —    │
│ …5678  1-06  ×1    ✗     │           │ …9012 2-02 ×2   ·    │ History             │
│ Spion Beat — Standard    │           │ …                    │  10:31 ✓ camera Undo│
│ …                        │           │                      │ [✓][✗ pack][label][⏸]│
└──────────────────────────┘           └──────────────────────┴─────────────────────┘
  tap → full-screen detail
```

- Row: last 4 digits of the tracking ID in bold, group (batch-group), total quantity, status
  icon, first item's display name.
- Grouped view: rows by batch → category → pack group, with progress per group.

## Problems

Three sections — Wrong packing · Label lost/damaged · Pending — each row with full tracking ID
and Order ID in large text, group, items, mark time. Tap → order detail (mark checked after
fixing, or undo).

## More

- **Batches**: list (number, import time, orders, groups, removed, ignored); **Import CSV** →
  summary with warnings → **Apply import** / **Cancel**.
- **History**: every scan and mark, newest first; filter by result; Undo on marks.
- **Session**: export `.zip`, rename.
- **Settings**: categories, saved scan filters, language, delete-protection time,
  sounds/vibration, export/import settings `.zip`.
