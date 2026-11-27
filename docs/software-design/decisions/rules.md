# Decisions: the rules file

Where a record names a former question (`Q-…`), the owner accepted its default.

## D-RULES-7 — The sort rule is chosen after reading; a plan changed for one batch is *Custom*

**Decision.** The sort rule is chosen on the Plan screen, after the labels are read; choosing another one
builds the plan again. The rule of the data folder's newest batch is preselected, on whichever date it was
saved; a rule that is off is never preselected (D-RULES-5). A tick, a move or **Add Group** on the Plan
screen makes the plan *Custom (from <rule>)*: it never changes `config.toml`, and the batch is saved with its
groups as they are under the name *Custom*, which no rule may take. A *Custom* batch preselects the rule it
came from. It replaces D-RULES-1 (chosen on New batch, before reading) and the Plan screen's *Also add to the
sort rule*.

**Why.** The owner's words (2026-11-27): a rule chosen before reading cannot be changed once the counts show,
which makes no sense; a group changed for one batch must not edit the rule. Nothing new is stored to remember
the last choice: the newest batch already says it.

**Applies to.** [Plan › The sort rule](../ui/plan.md#the-sort-rule), [Sort rules](../ui/sort-rules.md).

## D-RULES-2 — A sort rule can be renamed; a saved batch keeps its snapshot

**Decision.** **Rename** changes the name in `config.toml` only. An amend uses the rule of the batch's
recorded name as it is now; when no rule has that name any more, the snapshot in the batch's `day.toml`.

**Why.** Q-RULES-2's default — the rename follows the batches, so an old batch can always be amended — needs a
rename to rewrite stored batches, and a saved batch is never rewritten. This gives the same outcome: nothing is
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
`parser.toml` (the code when the rules no longer name it), so `courier contains "J&T"` takes J&T
Express and J&T Cargo; `courier equals` compares the **code** (R-RULES-4). On screen *equals* picks one courier
from the list by name and the other operators take typed text, in Sort rules, Search and the Plan screen's
one-off group alike.

**Why.** The owner's words: only *equals* needs the list; typing part of a name must find every courier of
that name. The names are what the owner sees everywhere; the code stays the one exact key.

**Applies to.** [Filter language](../rules-file/conditions.md#fields), [Sort rules](../ui/sort-rules.md),
[Search](../ui/search.md).

## D-RULES-5 — A sort rule can be switched off

**Decision.** A sort rule is **on** or **off** (R-RULES-6): `enabled = false` in `config.toml`,
no key when on. The Plan screen offers only the rules that are on; a rule that is off keeps its groups, can be
edited, renamed and deleted, and an amend of a batch saved with it still reads with it. At least one rule
is on: the only one cannot be switched off or deleted, and a file with every rule off fails its check.

**Why.** The owner asked for it: a rule not in use leaves the list of rules offered without being deleted, so it
can be switched on again as it was, and an old batch saved with it can still be amended.

**Applies to.** [rules file](../rules-file/README.md#sort-rules), [Sort rules](../ui/sort-rules.md),
[Plan › The sort rule](../ui/plan.md#the-sort-rule).

## D-RULES-6 — Account and shop are fields of the filter language

**Decision.** `account` and `shop` are text fields of the filter language (Q-RULES-3), so Search narrows to one
account or shop and a sort rule may take one account's orders. *Account equals* picks an account on the
screens; `shop is_empty` takes the orders whose label printed no sender.

**Why.** One language for Search and the sort rules, as for every other field; a second way to filter Search
would be more code for less.

**Applies to.** [filter language](../rules-file/conditions.md#fields), [Search](../ui/search.md).
