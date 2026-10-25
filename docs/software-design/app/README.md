# PC app

The PC app is a Windows program with windows and buttons: the owner adds a shop owner's label downloads,
sees the label groups with their counts, adjusts them if needed, and saves one label PDF per group and a
summary PDF, without a terminal. The screens are in [ui/](../ui/README.md); the reading and sorting rules
are in [labels/](../labels/README.md); the files are in [data/](../data/README.md); every step in order is
in [workflow/](../workflow/README.md).

## What the app decides

| Topic | Decision |
|---|---|
| How it is used | Windows program, buttons ([Screens](../ui/README.md)) |
| Whose labels | One namespace per shop owner, one folder per date inside it ([namespaces](namespaces.md)) |
| The truth | One journal per namespace and date; every other file is an input, an output or a view of it ([data](../data/README.md)) |
| Order data | All of it comes from the label and its packing slip ([D-APP-4](../decisions/app.md)) |
| Plain labels (no packing slip) | **Refused**: the batch stops and asks for the *Shipping label + Packing slip* download |
| Sorting | A sort rule from `packing-workflows.toml`, chosen per batch, applied as a proposal; the owner can skip, move or add a group for this batch before saving ([Plan](../ui/plan.md)) |
| Outputs | One PDF per label group, `{batch}-{code} {name} x{count}.pdf`, and one summary PDF per batch ([groups](../labels/groups.md), [summary PDF](../labels/summary-pdf.md)) |
| Correctness | An independent verifier checks every output before it is committed ([verification](verification.md)) |
| Undo | **Revert** (undo the newest batch of a date) and **Amend** (redo it in place) |
| Printing again | Search, tick, **Reprint**; a lost output is restored from the stored inputs ([reprint](reprint.md)) |

## Words used in this folder

| Word | Meaning |
|---|---|
| **Batch** | One round of the daily steps ([the packing workflow](../../business-process/01-packing-workflow.md)): arrange shipment, download the labels (one or more files of up to 200 pages), prepare them in the app. Numbered 1, 2, 3, … within a namespace and date. One **Save** = one batch |
| **Draft** | A batch read but not saved yet. It lives only in the app's memory; closing the app forgets it and nothing is written |
| **Plan** | The label groups of a draft, in order, with their counts. Starts as the groups of the batch's sort rule |
| **Live batch** | A saved batch that has not been reverted |
| **Data folder** | The folder `packing-settings.toml` names: one folder per namespace ([data](../data/README.md)) |
| Others | As in [Concepts](../requirements.md#concepts) |

## Files in this folder

- [architecture.md](architecture.md) — diagram, the crates and who may use whom, stack, threads, text runs
- [program.md](program.md) — commands between the window and Rust, the program folder, start
- [namespaces.md](namespaces.md) — shop owners and dates: names, listing, creating, a v0.1 data folder
- [duplicates.md](duplicates.md) — an order that appears more than once in a batch: copies, repeats, the choice
- [verification.md](verification.md) — the verifier: the page digest, its checks, the two severities
- [reprint.md](reprint.md) — the search index, search, reprint and restore
