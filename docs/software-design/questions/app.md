# Open questions: app

Questions: Q-APP-1.

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
