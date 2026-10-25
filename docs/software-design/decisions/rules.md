# Decisions: the rules file

Where a record names a former question (`Q-…`), the owner accepted its default.

## D-RULES-1 — A batch chooses its sort rule; the newest batch's rule is preselected

**Decision.** The sort rule is chosen per batch on New batch, before **Read labels**; the rule of the owner's
newest batch is preselected (Q-RULES-1 (a)). The rules live in `packing-workflows.toml` (D-APP-13), which
answers Q-RULES-1 (b).

**Why.** An owner who always uses one rule never touches the control, and nothing new is stored to remember
the last choice: the journal already says it.

**Applies to.** [New batch › The sort rule](../ui/new-batch.md#the-sort-rule), [rules file](../rules-file/README.md#sort-rules).

## D-RULES-2 — A sort rule can be renamed; a saved batch keeps its snapshot

**Decision.** **Rename…** changes the name in `packing-workflows.toml` only. An amend uses the rule of the batch's
recorded name as it is now; when no rule has that name any more, the snapshot in the batch's record.

**Why.** Q-RULES-2's default — the rename follows the batches, so an old batch can always be amended — needs a
rename to rewrite stored batches, and the journal is never rewritten. This gives the same outcome: nothing is
ever missing, and a renamed rule amends with the groups the batch had.

**Applies to.** [rules file](../rules-file/README.md#sort-rules), [Sort rules](../ui/sort-rules.md), [05 After the save](../workflow/05-after-save.md#18-amend-newest-live-batch-only).
