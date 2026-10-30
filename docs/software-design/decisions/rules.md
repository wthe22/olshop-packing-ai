# Decisions: the rules file

Where a record names a former question (`Q-…`), the owner accepted its default.

## D-RULES-1 — A batch chooses its sort rule; the rule used last is preselected

**Decision.** The sort rule is chosen per batch on New batch, before **Read labels**; the rule of the
workspace's newest batch is preselected, on whichever date it was saved, so a new day starts on the rule
used last (Q-RULES-1 (a), R-RULES-5). While a batch is being prepared, its own rule is preselected, so
the Plan screen's **Back** keeps the rule just read. A rule that is off is never preselected (D-RULES-5).
The rules live in `packing-workflows.toml` (D-APP-13), which answers Q-RULES-1 (b).

**Why.** An owner who always uses one rule never touches the control, and nothing new is stored to remember
the last choice: the journal of each date already says it.

**Applies to.** [New batch › The sort rule](../ui/new-batch.md#the-sort-rule), [rules file](../rules-file/README.md#sort-rules).

## D-RULES-2 — A sort rule can be renamed; a saved batch keeps its snapshot

**Decision.** **Rename…** changes the name in `packing-workflows.toml` only. An amend uses the rule of the batch's
recorded name as it is now; when no rule has that name any more, the snapshot in the batch's record.

**Why.** Q-RULES-2's default — the rename follows the batches, so an old batch can always be amended — needs a
rename to rewrite stored batches, and the journal is never rewritten. This gives the same outcome: nothing is
ever missing, and a renamed rule amends with the groups the batch had.

**Applies to.** [rules file](../rules-file/README.md#sort-rules), [Sort rules](../ui/sort-rules.md), [05 After the save](../workflow/05-after-save.md#18-amend-newest-live-batch-only).

## D-RULES-3 — `is_empty` takes the orders with no courier or no tracking ID

**Decision.** The filter language has `<text field> is_empty` (R-RULES-3). On screen it is *is unknown* for
Courier and Tracking ID, *is empty* for the other text fields, and takes no value. `courier equals` still takes a
courier's code only.

**Why.** An unnamed courier is stored empty, and an empty `equals` value is refused as "names no courier"; a
word of its own reads plainly in the file and on screen.

**Applies to.** [Filter language](../rules-file/conditions.md), [Sort rules](../ui/sort-rules.md).

## D-RULES-4 — A typed courier is compared with the couriers' names, a picked one by code

**Decision.** `courier contains` and `courier starts_with` compare the courier's **name** in
`packing-workflows.toml` (the code when the rules no longer name it), so `courier contains "J&T"` takes J&T
Express and J&T Cargo; `courier equals` compares the **code** (R-RULES-4). On screen *equals* picks one courier
from the list by name and the other operators take typed text, in Sort rules, Search and the Plan screen's
one-off group alike.

**Why.** The owner's words: only *equals* needs the list; typing part of a name must find every courier of
that name. The names are what the owner sees everywhere; the code stays the one exact key.

**Applies to.** [Filter language](../rules-file/conditions.md#fields), [Sort rules](../ui/sort-rules.md),
[Search](../ui/search.md).

## D-RULES-5 — A sort rule can be switched off

**Decision.** A sort rule is **on** or **off** (R-RULES-6): `enabled = false` in `packing-workflows.toml`,
no key when on. New batch offers only the rules that are on; a rule that is off keeps its groups, can be
edited, renamed and deleted, and an amend of a batch saved with it still reads with it. At least one rule
is on: the only one cannot be switched off or deleted, and a file with every rule off fails its check.

**Why.** The owner asked for it: a rule not in use leaves the New batch list without being deleted, so it
can be switched on again as it was, and an old batch saved with it can still be amended.

**Applies to.** [rules file](../rules-file/README.md#sort-rules), [Sort rules](../ui/sort-rules.md),
[New batch › The sort rule](../ui/new-batch.md#the-sort-rule).
