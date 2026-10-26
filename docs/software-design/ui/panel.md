# Panel

The side panel names the open workspace ([workspaces](../app/workspaces.md)), lists its dates, and opens
another date or workspace. It sits at the left of every screen; the owner of the app shows or hides it
([D-UI-5](../decisions/ui.md)).

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│ [▯] ◈ Packing (v0.2) │ Toko Contoh › Wed, 21 Oct 2026  Search  Sort rules  Settings │ [☀]│
├──────────────────────────┬─────────────────────────────────────────────────────────────┤
│ WORKSPACE                │  This day: 2 batches · 729 orders · 5 group PDFs            │
│ [Toko Contoh      ▾] [+] │                                         [ + New batch ]     │
│                          │  ┌ Batch 2 · saved 09:12 · 129 orders ─ [Amend] ┐           │
│ ▾ October 2026           │  │  2-A  Sepatu         88 orders  [✓] [Open]   │           │
│ ▌ Wed 21 Oct · today 729 │  │  2-Z  Lainnya        41 orders  [ ] [Open]   │           │
│   Tue 20 Oct         530 │  └──────────────────────────────────────────────┘           │
│   Mon 19 Oct         412 │                                                             │
│ ▸ September 2026         │                                                             │
│ ⚠ Sat 3 Oct  damaged     │                                                             │
└──────────────────────────┴─────────────────────────────────────────────────────────────┘
```

## The panel

- **Workspace** chooses the workspace; the list is every workspace of the data folder in name order. Choosing
  one opens it on today's date. **+ New workspace** asks for a name ([workspaces › Names](../app/workspaces.md#names))
  and opens the new workspace on today's date; a name taken is refused beside the field. It is the **[+]** beside
  the list, which opens the name field with **Cancel** and **Create**.
- **Dates** of the workspace, newest first, each with its number of orders from the search index. Today is
  always listed, with *· today*, even before it has a folder. The open date is marked ▌ and shaded.
- **Months fold**: the month of the open date and the current month start open, the others closed.
- **Click a date** to open it ([01 Start and open](../workflow/01-start-and-open.md#2-open-a-date)). A draft
  being prepared asks first.
- **A damaged date** shows ⚠ and *damaged*; it opens for viewing only, with the message.
- **Row menu** (**⋯**, on hover): **Open folder**. The app never renames or deletes a workspace or a date.
- **[▯]** in the title bar (or `Ctrl+B`) hides and shows the panel. Hidden, the screen takes the full width and
  the title bar still names the workspace and the date. It is shown at every start, and below the smallest
  window width it hides on its own.
- The list is read again when the window regains focus, so a folder added in Explorer shows up.

There is no day stepper in the title bar: the panel's *today* row is the way back.
