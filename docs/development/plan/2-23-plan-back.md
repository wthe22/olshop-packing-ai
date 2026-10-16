# 2.23 Plan screen: Back at the top, no discard button, Cancel amend at the bottom

The Plan screen's top button is **Back**, not Cancel ([plan.md](../../software-design/ui/plan.md)):
it returns to the PDF selection page with the files still listed, without a confirmation, and writes
nothing; the per-batch changes are lost when **Read labels** is pressed again (reading starts again
from the files). A new batch's Plan screen has no bottom discard button. On an amend, a bottom
**Cancel amend** beside **Save batch 2 again** leaves the batch as it was and returns to the Day
screen, and never asks.

- **Code:** `PlanScreen.svelte` (the top **Back**, no discard button, the amend's bottom **Cancel
  amend**), `NewBatchScreen.svelte` and the shell `+page.svelte` (**Back** shows the file list again
  with the files still listed; the top **Cancel** there discards), `texts.ts` (*Back* and *Cancel
  amend*; the *Discard batch n?* text goes). `app/src-tauri/commands/batch.rs`: **Back** keeps the
  draft; a new read replaces it, so no plan edit survives. The amend's **Back** target is the amend's
  file-selection page ([R-APP-1](../../reviews/app.md)); until that page exists it returns to the Day
  screen where the amend started.
- **Tests:** **Back** returns to the file list with the files still there and no confirmation; a read
  after **Back** starts again from the files (the plan edits are gone); a new batch's Plan screen has
  no discard button; **Cancel amend** leaves the batch as it was and never asks; the top button reads
  **Back**.
- **Result:** (not started)
