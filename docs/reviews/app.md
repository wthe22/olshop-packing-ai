# APP: the owner's feedback

The owner's findings on batches, the data folder, revert and amend, and the program folder. Open items
only; see [README](README.md).

## R-APP-1 — The check must prove every input page appears exactly once

**Present.** The check before saving compares the saved files with the plan, which is built from the same
order list as the files. A page read twice is written twice and the check agrees with it.

**Wanted.** Every input page maps to an output **1:1**: exactly one appearance, never 1:N (written
twice) and never 1:0 (lost).

**Design written, waits for the owner.** [The page ledger](../software-design/app/saved-pdf-check.md);
the page identity is [Q-LABEL-10](../software-design/questions/duplicates.md).

## R-APP-2 — A full workflow of the program, to verify

**Present.** The steps are spread over the app, labels and ui folders.

**Wanted.** The whole workflow in one place — steps, processing logic and algorithms — so the owner can
verify that they are correct.

**Written, waits for the owner's reading.** [The program's workflow](../software-design/workflow/README.md).
What the owner finds wrong goes here as an item. Where the program as built differs from it:
[workflow-audit](../development/plan/workflow-audit.md).
