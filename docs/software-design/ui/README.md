# PC app screens

The screens of the PC app ([app/](../app/README.md)). One window, designed for a laptop
screen of 1366 × 768 or larger (smallest usable size 1024 × 640). Sketches show the English texts and
made-up IDs and counts.

| Screen | Content |
|---|---|
| [Day](day.md) | The start screen: the batches saved for the day, the way to the next batch, revert and amend |
| [New batch](new-batch.md) | Choosing the label files and the grouping of the batch, reading them and the logo prompt |
| [Plan](plan.md) | The picks, the per-batch changes, the packing list and Save |
| [Messages](messages.md) | Every stop message and warning, and what to do |
| [Categories](categories.md) | The boxes editor for a grouping (the data folder's `categories.toml` or a file in `categories/`) |
| [Settings](settings.md) | The settings and the data folder |
| [Search](search.md) | Finding saved orders of the day |
| [Start](start.md) | Finding the data folder at every start |

## Daily use in one picture

```
 Day screen ──[New batch]──► New batch ──[Read labels]──► (reading…) ──► Plan ──[Save batch]──┐
     ▲                         choose files                               adjust picks       │
     └────────────────────────────── back to the Day screen, new batch shown ◄────────────────┘
 From the Day screen: [Open] a saved PDF to print it · [Amend] / [Revert] the last batch ·
 [Search] · [Categories] · [Settings]
```

A normal batch: **New batch** → **Add files…** (choose the new downloads) → **Read labels** →
(the plan is the rules file, as every day) **Save batch** → **Open** each saved PDF and the
packing list to print.

## Window frame

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│ Packing v0.1  ·  ◀  Wed 7 Oct 2026  ▶  [Today]      [Search]  [Categories]  [Settings] │
├────────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                        │
│                               (the current screen)                                     │
│                                                                                        │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

- **Packing v0.1** is the window title and the title bar of every screen: the program's name and its
  version, from the one place ([Settings › the version](settings.md#the-version)).
- **Day** in the title bar: the day folder everything is saved to. ◀ ▶ step a day; **Today**
  goes back. Changing the day while a draft is open asks first ("Discard the batch being
  prepared?").
- **Search**, **Categories** and **Settings** open over the current screen; closing them returns to it
  (an open draft is kept and recounted).
