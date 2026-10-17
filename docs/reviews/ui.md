# UI: the owner's feedback

The owner's findings on the window and the screens. Open items only; see [README](README.md) for the form.

## R-UI-3 — The Printed tick is too small and has a word beside it

**Present.** A saved-PDF row of the [Day screen](../software-design/ui/day.md) has a small **Printed** tick box,
and the word *Printed* stands beside it.

**Wanted.** The box is as large as the **Open** button of the same row, and the word goes: the tick alone
says that the PDF is printed.

## R-UI-4 — Call a set of categories a *Grouping*

**Present.** A set of categories is called *set*: the **Set** control on [New batch](../software-design/ui/new-batch.md),
**+ New set** and **Delete set** on the Categories screen, and the set's name in the Plan header.

**Wanted.** The word is *Grouping*: *Grouping: [default ▾]*, **+ New grouping**, **Delete grouping**. Only
the set of categories is renamed. A *PDF set* (the PDFs one batch saves) keeps its name. The design files
use the same word.
