# PC app screens

The screens of the PC app ([app/](../app/README.md)). One window, designed for a laptop screen of
1366 × 768 or larger (smallest usable size 1024 × 640). Sketches show the English texts and made-up names,
IDs and counts.

| Screen | Content |
|---|---|
| [Panel](panel.md) | The side panel: the workspace, its dates, **+ New workspace** |
| [Day](day.md) | The start screen: the batches of the open date, printed ticks, revert, amend, restore, the day's reprints, the day summary |
| [New batch](new-batch.md) | Choosing the label files and the sort rule, reading them, the logo prompt |
| [Repeated orders](duplicates.md) | The orders that appear more than once in a batch, grouped, and the copy that stays |
| [Plan](plan.md) | The label groups, the per-batch changes and Save |
| [Messages](messages.md) | Every stop message and warning, and what to do |
| [Sort rules](sort-rules.md) | The boxes editor for the sort rules of `packing-workflows.toml` |
| [Search](search.md) | Finding saved orders, their trace, and **Reprint** |
| [Settings](settings.md) | The data folder and *About* |
| [Start](start.md) | The first start, the lock, the first workspace |

## Daily use in one picture

```
 Day ──[New batch]──► New batch ──[Read labels]──► (reading…) ──► Plan ──[Save batch]──► (writing, verifying)
  ▲                    choose files, sort rule                    adjust groups                 │
  └──────────────────────────────── back to the Day screen, new batch shown ◄───────────────────┘
 From the Day screen: [Open] a file to print it · the printed tick · [Amend] / [Revert] the newest batch ·
 [Search] (and Reprint) · [Sort rules] · [Settings] · the panel: another date or workspace
 Between (reading…) and Plan: the logo prompt, and Repeated orders when an order appears twice.
```

A normal batch: **New batch** → **Add files…** → **Read labels** → (the plan is the sort rule, as every day)
**Save batch** → **Open** each group PDF and the summary PDF to print.

## Window frame

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│ [▯] ◈ Packing (v0.2) │ Toko Contoh › Mon, 26 Oct 2026   🔍 Search  ≡ Sort rules  ⚙ Settings │ [☀]│
├────────────────────────────────────────────────────────────────────────────────────────┤
│                               (the current screen)                                     │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

- **[▯]** shows or hides the [panel](panel.md) (also `Ctrl+B`).
- **Packing v0.2** is the window title and the title bar of every screen: the program's name and its version
  from the one place ([Settings › The version](settings.md#the-version)).
- **The workspace › the date** in the title bar: where everything is saved. Changing the workspace or the
  date while a draft is open asks first ("Discard the batch being prepared?").
- **Search**, **Sort rules** and **Settings** open over the current screen; the **×**, a click beside it or
  `Esc` returns to it (an open draft is kept and recounted).
- **[☀]** is the theme menu: **Light**, **Dark** or **System** (Windows' own choice, the default), stored in
  `packing-settings.toml` ([D-UI-7](../decisions/ui.md)). The button shows the theme in use.

## Questions, waits and how things are written

- **A question** before an action that deletes or redoes something is the app's own dialog, never Windows'
  message box: a title that asks ("Undo batch 2?"), at most one short line, the button named for what it
  does and **Cancel**. Enter answers with the button, `Esc` with **Cancel**.
- **A wait**: a file the owner waits for (the day summary, an output written again) shows a dialog with its
  line and a moving bar once it takes longer than a moment, over a backdrop that takes the clicks; a second
  click while it is written starts nothing.
- **Dates** read *Wed, 21 Oct* (*Wed, 21 Oct 2026* in the title bar); **a time** always carries its word
  (*saved 09:12*, *made 09:40*, *downloaded 11:02*), never a bare `09:12`.
- **A hash or a pattern** (a logo's hash, a tracking-ID form, the rules file) is set in monospace.
