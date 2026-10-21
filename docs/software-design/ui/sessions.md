# Sessions panel

The side panel lists every session of every day ([sessions](../app/sessions.md)) and opens one. It
sits at the left of every screen, and the owner shows or hides it.

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│ Packing v0.1  ·  [☰]  Wed 21 Oct 2026 · Main        [Search]  [Categories]  [Settings] │
├──────────────────────────┬─────────────────────────────────────────────────────────────┤
│ Sessions            [«]  │  This session: 2 batches · 729 orders · 5 saved PDFs        │
│ [ + New session ]        │                                         [ + New batch ]     │
│                          │  ┌ Batch 2 · saved 09:12 · 129 orders ─ [Amend] ┐           │
│ ▾ October 2026           │  │  4  A  Sepatu          88 orders  [✓] [Open] │           │
│   Wed 21 Oct · today     │  │  5  Z  Lainnya         41 orders  [ ] [Open] │           │
│ ▌ Main        729 orders │  └──────────────────────────────────────────────┘           │
│   Tue 20 Oct             │  ┌ Batch 1 · saved 07:40 · 600 orders ──────────┐           │
│     Main       410 orders│  │  1  A  Sepatu         172 orders  [ ] [Open] │           │
│     Shop B     120 orders│  └──────────────────────────────────────────────┘           │
│     Try setup  empty     │                                                             │
│ ▸ September 2026         │                                                             │
└──────────────────────────┴─────────────────────────────────────────────────────────────┘
```

## The panel

- **Rows.** A day heading (`Tue 20 Oct`) with its sessions under it: Main first, then the named
  ones in name order. A session row shows its name and its number of orders (`empty` when nothing is
  saved); the tooltip adds batches and saved PDFs. The open session is marked ▌ and shaded.
- **Today** is always listed, with *· today* beside its heading, even before it has a folder.
- **Order.** Newest day first. Months are groups with ▾ / ▸; the month of the open session and the
  current month start open, the others closed ([Q-UI-5](../questions/sessions.md)). The list is
  read from the folders when the app starts and when the window regains focus
  ([sessions](../app/sessions.md#listing-opening-and-creating)); a row's number of orders is read
  from its `state.json` when its month is opened, so a long list opens at once.
- **Click a row** to open that session: the Day screen shows it and the title bar names it. A
  batch being prepared asks first ("Discard the batch being prepared?"), as a change of day did
  before. The panel stays as it is.
- **Row menu** (**⋯**, on hover): **Rename…** and **Open folder** ([Q-UI-6](../questions/sessions.md)).
- **[«]** hides the panel; **[☰]** in the title bar (or `Ctrl+B`) shows it again. Hidden, the
  screen takes the full width and the title bar still names the day and the session. The choice is
  remembered in `settings.json` and is *shown* on a first start.
- The title bar's ◀ ▶ **Today** are gone: the panel replaces them ([Q-UI-7](../questions/sessions.md)).
  The panel's *today* row is the way back.
- The panel is 220 px wide; below the smallest window width (1024 px) it is hidden on its own.

## New session

**+ New session** opens a small form over the panel:

```
┌ New session ────────────────────────────┐
│  Day   [ 2026-10-21 ▾ ]                 │
│  Name  [ Shop B                       ] │
│        Leave empty for Main.            │
│                                         │
│  [ Cancel ]                 [ Create ]  │
└─────────────────────────────────────────┘
```

- **Day** defaults to the day of the open session; a date picker changes it (a download just
  after midnight belongs to the evening before).
- **Name** follows the rules of [sessions](../app/sessions.md#where-a-session-lives): **Create** is
  disabled with the reason beside the field when the name is taken ("Shop B already exists on this
  day.") or when the name is empty and the day already has a Main.
- **Create** makes the folder, opens the new session on its Day screen ("Nothing saved in this
  session yet.") and lists it.
- **Rename…** is the same field, with the session's day shown and not editable.

## What changes on the other screens

| Screen | Change |
|---|---|
| [Window frame](README.md#window-frame) | `Wed 21 Oct 2026 · Main` replaces ◀ ▶ [Today]; [☰] shows the panel |
| [Day](day.md) | The summary line says *This session*; the empty text is "Nothing saved in this session yet." |
| [Search](search.md) | Scope *This session* (default) or *All sessions*; a row in *All sessions* shows its day and session |
| [Plan](plan.md) | The header and the saved-PDF numbers belong to the session; a named session's name is in the packing-list header |
