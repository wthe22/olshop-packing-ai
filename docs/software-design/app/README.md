# PC app

The PC app is a Windows desktop program with windows and buttons: the owner adds the day's label
downloads, sees the picks with their counts, adjusts them if needed, and saves the label PDFs and
packing lists without a terminal. The screens are in [ui/README.md](../ui/README.md); the reading
and sorting rules it uses are in [labels/README.md](../labels/README.md).

## What the app decides

The rules and the files that hold them:

- [Reading the labels](../labels/reading.md) — the Order ID of a page (two-step search), multi-page orders, reading the packing slip by text position (columns, rows, wrapped cells, the `Qty Total` check)
- [Courier deduction](../labels/courier.md) — the courier from the label text and the tracking-ID form (`couriers.toml`)
- [Picks](../labels/picks.md) — picks taken in order from what is left, the rest, runs inside a saved PDF, saved-PDF numbers through the day and run numbers `3-05`, the duplicate guard
- [Packing list](../labels/packing-list.md) — the three layouts `full`, `summary`, `pick` and their look
- [Rules file](../rules-file/README.md) — the groupings of categories (`categories.toml` and `categories/`)
- [Conditions](../rules-file/conditions.md) — the condition language
- [Screens](../ui/README.md) — daily use and the window frame
- The files of this folder: [inputs.md](inputs.md), [batch.md](batch.md), [saved-pdf-check.md](saved-pdf-check.md), [data-folder.md](data-folder.md), [architecture.md](architecture.md), [program.md](program.md)

Owner decisions about how the app works:

| Topic | Decision |
|---|---|
| How it is used | Windows program, buttons ([Screens](../ui/README.md)) |
| Orders CSV | **Not read at all.** All order data comes from the label and its packing slip |
| Plain labels (no packing slip) | **Refused**: the batch stops and asks for the *Shipping label + Packing slip* download |
| Rules file vs interactive menu | One screen: the categories of the batch's grouping are applied in order as a proposal; the owner can skip, move or add a pick for this batch before saving ([Plan screen](../ui/plan.md)) |
| Undo | **Revert** (undo the last batch) and **Amend** (redo the last batch in place), like git |
| Output folder | `labels/<day>/batch <b>/` inside the data folder, one subfolder per batch, with a copy of the batch's label downloads |
| Saved-PDF name | Suggested as `<n> <code> <name> ×<orders>.pdf` ([Numbering and file names](../labels/picks.md)); the owner can change the whole name before saving |
| Packing-list default | One per batch (`whole`), layout `pick` (changeable per batch) |
| Fields in conditions | Only fields the label/slip carry ([the fields the conditions can use](inputs.md#fields-the-conditions-can-use)) |

## Words used in this document

| Word | Meaning |
|---|---|
| **Batch** | One round of the daily steps ([the packing workflow](../../business-process/01-packing-workflow.md)): arrange shipment, download the labels (one or more files of up to 200 pages), prepare them in the app. Numbered 1, 2, 3, … within the day. In the app, one **Save** = one batch |
| **Draft** | A batch that has been read but not saved yet. It lives only in the app's memory; closing the app forgets it and nothing is written |
| **Plan** | The list of picks of a draft, in order, with their counts. Starts as the categories of the batch's grouping in file order |
| **Data folder** | The folder where the app keeps the picks groupings (`categories.toml` and the `categories/` folder, [rules file](../rules-file/README.md)), `couriers.toml` and the `labels/` folder with every day's output. By default the program's own folder ([Program folder](program.md#program-folder-portable)) |
| **Pick**, **saved PDF**, **run**, **contents** | As in [Concepts](../requirements.md#concepts) |

## Files in this folder

- [inputs.md](inputs.md) — the inputs, `couriers.toml`, the packing-slip requirement, the fields the conditions can use
- [batch.md](batch.md) — one batch step by step, revert and amend, the packing list in a batch
- [saved-pdf-check.md](saved-pdf-check.md) — the check before saving
- [data-folder.md](data-folder.md) — the day folder and `state.json`
- [architecture.md](architecture.md) — diagram, stack, code layout, text runs from PDFium
- [program.md](program.md) — commands between the window and Rust, settings, the portable program folder
