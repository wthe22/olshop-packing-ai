# Workspaces and dates

A **workspace** is a folder of the data folder that keeps its batches apart from every other workspace's
([D-APP-17](../decisions/app-features.md)); one folder per date sits inside it
([data](../data/README.md#the-data-folder)). The screens call it a **workspace** too. What a workspace stands
for is the owner's choice; it was made for one shop owner of the
[business process](../../business-process/01-packing-workflow.md), all of whose shops and platforms share it.

| Word | Meaning |
|---|---|
| **Workspace** | A folder of the data folder with its own dates, batch numbers and guard |
| **Date** | A calendar date of a workspace: one folder with its journal, inputs and outputs |

## What a workspace holds on its own

- The batches of each date, numbered 1, 2, 3, … per date.
- The **already-saved guard**: an order saved in a live batch of a date is left out of a later batch of
  that date ([03 Plan](../workflow/03-plan.md#7-orders-already-saved)). Two workspaces never touch each
  other: the same labels read under another workspace are new orders there.
- The *used in batch n* hint, revert and amend: each date has its own newest batch.
- The search index and the reprints ([reprint](reprint.md)).

What workspaces share is beside the program: `packing-workflows.toml` (couriers and sort rules) and
`packing-settings.toml`.

## Names

- A name follows the default file-name rule ([data](../data/README.md#types-versions-and-names)) and is
  1–40 characters. Two workspaces cannot share a name (upper and lower case count as the same).
- `labels` and `categories` are not names: they are the folders of a version 0.1 data folder.
- The name is the folder's name and is written into the `started` record of each of its journals.

## Listing, opening, creating, renaming and deleting

- **The list** is every folder of the data folder whose name is a workspace name, read at start and when
  the window regains focus, so a folder added in Explorer shows up.
- **At start** the app opens the workspace whose `journal.jsonl` was written last, on today's date (the
  PC's date). With no workspace yet it asks for the first workspace's name ([Start](../ui/start.md)).
- **+ New workspace** creates the folder at once, empty, and opens it on today's date.
- **Another date** is opened from the [panel](../ui/panel.md): a download just after midnight that
  belongs to the evening before is saved under that evening's date.
- **A date folder** is created at its first commit; today is listed before it exists.
- **Rename** (Settings › Workspaces) renames the folder; everything in it stays. Its journals keep the name
  they were started under, and a date opens whatever name its folder has now
  ([D-APP-24](../decisions/app-features.md)); a folder renamed in Explorer works the same. A file of the
  workspace open in another program stops the rename, which says so.
- **Delete** (Settings › Workspaces, asked first) moves the workspace's folder, with all its dates, to the
  Recycle Bin, where Explorer can restore it.
- **The open workspace follows**: renamed, it stays open on the same date; deleted, the app opens the one it
  opens at start, or asks for the first workspace's name when none is left. An open draft of it is dropped,
  after asking.
- **A date** is never renamed or deleted by the app.

## A version 0.1 data folder

A data folder that holds `labels/`, `state.json`, `couriers.toml`, `categories.toml` or `categories/` was
written by v0.1. The app does not read it, never deletes it, and says what it found: a line on the Day
screen, *This folder also holds data of Packing v0.1 (`labels\`). It is not read.* with **Open folder**.
