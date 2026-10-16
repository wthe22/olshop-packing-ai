# Open questions: app

Questions: Q-APP-1, Q-APP-2, Q-APP-3.

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
