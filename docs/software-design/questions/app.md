# Open questions: app

Questions: Q-APP-1, Q-APP-2, Q-APP-3, Q-APP-8 to Q-APP-11.

## Q-APP-1 — A warning alone in the check before saving

**Words.** The [check before saving](../app/saved-pdf-check.md) has two severities. An *error* is a
missing, doubled or misplaced page or a wrong count. A *warning* is only a different page order inside
a saved PDF: every page is there exactly once.

**Example.** Batch 3 is written with all 600 pages, but two orders sit in the other order inside a
saved PDF. Nothing else is wrong.

| Option | What the owner sees |
|---|---|
| **1. A warning stops the save like an error** (**Default**) | The dialog names the swapped order; **Save anyway** saves, **Cancel — nothing is saved** keeps the draft |
| 2. A warning does not stop the save | The batch saves at once; the Day screen's batch card shows *Warnings ▸* |

**What the answer changes:** the stop condition in `crates/pdf/src/batch.rs`, the dialog, and the
wording in `saved-pdf-check.md`, `ui/messages.md` and `ui/day.md`.

## Q-APP-2 — Where an added PDF comes from on an amend

**Words.** An amend's Plan screen's **Back** shows a page where the owner selects the batch's label
PDFs ([New batch](../ui/new-batch.md#the-amends-selection-page)): the current selection is listed
(the batch's own PDFs at first), already selected. **Add files…** adds more. A new batch adds files
from anywhere with **Add files…** (the Windows file dialog) or by dropping them on the window, and
every batch keeps a `download/` copy of each file it read ([day folder](../app/data-folder.md)), so a
file read earlier the same day can be listed again without the original.

**Example.** Batch 2 was saved from two of the download's three files. The owner presses **Amend** and
wants the third file in: it is still in Downloads, and it was never read today.

| Option | What the owner sees |
|---|---|
| **1. The file picker: any PDF on the PC** (**Default**) | **Add files…** opens the Windows file dialog as in a new batch, and a drop works too. The added file is listed, **Read labels** reads it, and its orders join the plan |
| 2. Only PDFs already read on this day | **Add files…** lists the day's read files (their `download/` copies) to pick from; a PDF the app has not read today cannot be added, even if it lies in Downloads |

**What the answer changes:** **Add files…** and the drop target on the selection page in
`NewBatchScreen.svelte`, the file list the amend's start command offers
(`app/src-tauri/src/commands/batch.rs`), and the wording in [batch.md](../app/batch.md) and
[new-batch.md](../ui/new-batch.md).

## Q-APP-3 — What happens to the orders of a PDF removed on an amend

**Words.** Every listed file has a **×** that takes it off the list
([New batch](../ui/new-batch.md#the-amends-selection-page)); on an amend that removes the PDF from the
batch. Its orders are then not in the plan and not saved by the amend. The labels stay readable, so the
next batch finds those orders unsaved and reads them again.

**Example.** Batch 2 holds 129 orders from two files. The owner amends it and removes the second file
(41 orders).

| Option | What the owner sees |
|---|---|
| **1. They stay unsaved** (**Default**) | The plan of *Redo batch 2* holds the kept file's 88 orders; after saving, the 41 orders are not saved today and come back in the next batch with the same labels |
| 2. The owner must move them into another PDF first | **Save batch 2 again** waits until the 41 orders sit in another kept file or another pick; they cannot be left unsaved |

**What the answer changes:** the plan for an amend and the save check (`crates/engine/`), the **×** on
the selection page, and the wording in [batch.md](../app/batch.md) and
[new-batch.md](../ui/new-batch.md).

## Q-APP-8 — Which field identifies an item

**Words.** An item is one product line of an order. The platform's `SKU ID` is not on the label PDFs, so the
app has only the name, the variation and the Seller SKU (the Shopee-style slip's `SKU` column, often empty).

**Example.** Two orders each have *Sepatu, 42, Seller SKU SP-0001*; a third has *Sepatu, 42* with no Seller SKU.

| Option | What the owner sees |
|---|---|
| **1. Name and variation** (**Default**) | The three lines are one item: the packing key of the runs and of the saved items |
| 2. Name, variation and Seller SKU | The third line is a different item, so its order sits in another run |

**What the answer changes:** the contents key ([picks.md](../labels/picks.md)), the runs, the item fields of `saved[]`, and [conventions.md](../conventions.md).

## Q-APP-9 — A deadline with a date only

**Words.** The TikTok label prints `In transit by: dd/mm/yyyy hh:mm` (a `Datetime`). The Shopee-style label prints `Batas Kirim: dd-mm-yyyy`, a date with no time.

| Option | What the owner sees |
|---|---|
| **1. One field, optional time** (**Default**) | `ship_by_time` is a `Datetime`, empty for a date-only label; the date is still shown |
| 2. A separate date field | `ship_by_date` (a `Date`) for date-only labels, `ship_by_time` for the rest |

**What the answer changes:** the `ship_by` condition field ([conditions.md](../rules-file/conditions.md)), `orders.csv` and `saved[]`.

## Q-APP-10 — One namespace per owner, or one per shop

**Words.** The owner's draft says an owner may have several shops. A namespace is the owner; a shop's orders on the same day would share its numbers and its already-saved guard.

| Option | What the owner sees |
|---|---|
| **1. One namespace per owner** (**Default**) | All shops of an owner on one date share one numbering; the already-saved guard covers the whole namespace and date |
| 2. One namespace per shop | Each shop has its own numbers and guard, as the sessions had |

**What the answer changes:** the namespace folders, the numbering and the guard ([D-APP-5](../decisions/app.md)).

## Q-APP-11 — Where packing-workflows.toml lives

**Words.** The owner's draft places `packing-settings.toml` and `packing-workflows.toml` beside the program. Today the groupings and `couriers.toml` are in the data folder ([data folder](../app/data-folder.md)).

| Option | What the owner sees |
|---|---|
| **1. Beside the program** (**Default**) | One portable file with the couriers and the groupings; the data folder holds only the days |
| 2. In the data folder | The file travels with the owner's data; the program folder has no state |

**What the answer changes:** the file's location in the app ([architecture](../app/architecture.md)) and [D-APP-6](../decisions/app.md).

