# PC app screens

The screens of the PC app ([app/](../app/README.md)). One window, designed for a laptop screen of
1366 × 768 or larger (smallest usable size 1024 × 640). Sketches show the English texts and made-up names,
IDs and counts.

| Screen | Content |
|---|---|
| [Panel](panel.md) | The side panel: the owner (namespace), its dates, **+ New owner** |
| [Day](day.md) | The start screen: the batches of the open date, printed ticks, revert, amend, restore, the day's reprints |
| [New batch](new-batch.md) | Choosing the label files and the sort rule, reading them, the logo prompt |
| [Repeated orders](duplicates.md) | The orders that appear more than once in a batch, grouped, and the copy that stays |
| [Plan](plan.md) | The label groups, the per-batch changes and Save |
| [Messages](messages.md) | Every stop message and warning, and what to do |
| [Sort rules](sort-rules.md) | The boxes editor for the sort rules of `packing-workflows.toml` |
| [Search](search.md) | Finding saved orders, their trace, and **Reprint** |
| [Settings](settings.md) | The data folder and the version |
| [Start](start.md) | The first start, the lock, the first owner |

## Daily use in one picture

```
 Day ──[New batch]──► New batch ──[Read labels]──► (reading…) ──► Plan ──[Save batch]──► (writing, verifying)
  ▲                    choose files, sort rule                    adjust groups                 │
  └──────────────────────────────── back to the Day screen, new batch shown ◄───────────────────┘
 From the Day screen: [Open] a file to print it · the printed tick · [Amend] / [Revert] the newest batch ·
 [Search] (and Reprint) · [Sort rules] · [Settings] · the panel: another date or owner
 Between (reading…) and Plan: the logo prompt, and Repeated orders when an order appears twice.
```

A normal batch: **New batch** → **Add files…** → **Read labels** → (the plan is the sort rule, as every day)
**Save batch** → **Open** each group PDF and the summary PDF to print.

## Window frame

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│ Packing v0.2  ·  [☰]  Toko Contoh · Wed 7 Oct 2026     [Search]  [Sort rules] [Settings]│
├────────────────────────────────────────────────────────────────────────────────────────┤
│                               (the current screen)                                     │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

- **Packing v0.2** is the window title and the title bar of every screen: the program's name and its version
  from the one place ([Settings › The version](settings.md#the-version)).
- **The owner and the date** in the title bar: where everything is saved. **[☰]** shows or hides the
  [panel](panel.md); changing the owner or the date while a draft is open asks first ("Discard the batch
  being prepared?").
- **Search**, **Sort rules** and **Settings** open over the current screen; closing them returns to it (an
  open draft is kept and recounted).
