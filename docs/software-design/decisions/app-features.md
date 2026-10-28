# Decisions: app — features

D-APP-13 to D-APP-23. Sources, data and checks are in [app.md](app.md). Where a record names a former
question (`Q-…`), the owner accepted its default.

## D-APP-13 — packing-workflows.toml beside the program

**Decision.** The couriers, their clues, `not_courier` and the sort rules are one file beside the program
(Q-APP-11). It replaces `couriers.toml`, `categories.toml` and `categories/`. A saved batch keeps a snapshot of
the sort rule it used. The label formats (Order ID captions, slip headers) stay in code.

**Why.** The owner's draft; one portable file of rules. The snapshot keeps every batch traceable. The formats
are the platforms' facts, and a new platform needs new code and tests anyway: a format language would have no
user.

**Applies to.** [rules file](../rules-file/README.md).

## D-APP-14 — Reprint the labels of chosen orders

**Decision.** Search has a tick per row, *Select all results* and **Reprint**: one PDF of the chosen orders'
pages from `input/`, in their original printing order, unchanged, verified, recorded as `reprint_made` in
today's journal. It never changes a batch or the already-saved guard.

**Why.** The owner's draft: a label sometimes needs printing again (damage, a mistake), and printing twice by
mistake must be avoidable and traceable.

**Applies to.** [reprint](../app/reprint.md#reprint), [Search](../ui/search.md).

## D-APP-15 — Search runs on an in-memory index of the journals

**Decision.** The index is built from the workspace's journals when it opens and updated after each commit. It
is only read; every write decision folds the journal.

**Why.** Fast search over every date, and still one truth.

**Applies to.** [reprint › The search index](../app/reprint.md#the-search-index).

## D-APP-16 — One summary PDF per batch, and no output options

**Decision.** Each batch writes `{batch} summary.pdf`: facts, groups with their order contents, picks
([D-LABEL-16](label.md)). The packing-list
scopes and layouts, and their setting, are gone.

**Why.** The owner asked for few options with sensible defaults; the pick layout was the one in use.

**Applies to.** [summary PDF](../labels/summary-pdf.md), [Plan](../ui/plan.md), [Settings](../ui/settings.md).

## D-APP-17 — One workspace per shop owner; v0.1 data is left alone

**Decision.** A workspace holds one shop owner's labels (Q-APP-10); all of that shop owner's shops share its
numbers and its guard. The screens, the code and the documents call it a **workspace**, never an owner (the
owner's word, 2026-10-26).
Two workspaces share nothing but the files beside the program (Q-APP-5: nothing shared). A v0.1 data folder is
not read and never deleted; the app says what it found. Q-APP-4 (session folders) has no subject any more.

**Why.** The owner's draft: a shop owner may have several shops. "Owner" on screen was confused with the
owner of the app.

**Applies to.** [workspaces](../app/workspaces.md).

## D-APP-18 — A finding of the verifier stops the save

**Decision.** An error, or a warning alone, stops the save (Q-APP-1). **Save anyway** commits the batch with the
findings in its record; a reprint or a restore has no *anyway*.

**Why.** A finding is never a message that scrolls away, and the record keeps what was accepted.

**Applies to.** [verification](../app/verification.md#two-severities-never-one-list), [Plan](../ui/plan.md#saving).

## D-APP-19 — An amend may add any file and take files off

**Decision.** An amend's file list adds any PDF on the PC (Q-APP-2); a file taken off leaves its orders unsaved,
for a later batch (Q-APP-3).

**Why.** An amend is the newest batch done again; orders left unsaved are lost nowhere, since their labels are
read again.

**Applies to.** [05 After the save](../workflow/05-after-save.md#18-amend-newest-live-batch-only), [New batch](../ui/new-batch.md#the-amends-file-list).

## D-APP-20 — An item is its product name and variation

**Decision.** Name and variation identify an item (Q-APP-8): the order content of a group PDF and the units of
the pick pages. The Seller SKU is stored with the order's line but is not part of its order content.

**Why.** The platform's `SKU ID` is not on the label PDFs, and one item can come with or without a Seller SKU.

**Applies to.** [groups](../labels/groups.md#a-group-pdf), [03 Plan](../workflow/03-plan.md#10-order-contents-and-file-names), [conventions](../conventions.md).

## D-APP-21 — The ship-by deadline is a date

**Decision.** An order keeps `ship_by_date: Date`; the clock time of `In transit by` is not kept, and the filter
field `ship_by` compares dates. The owner's draft wrote this in place of Q-APP-9's default (one field with an
optional time).

**Why.** The owner's draft. Every label style prints a date; a time is printed by some only.

**Applies to.** [reading](../labels/reading.md#per-order-data), [filter language](../rules-file/conditions.md).

## D-APP-22 — The Repeated orders screen opens on its defaults

**Decision.** Every group starts on *Keep the first copies*, so **Continue** works at once (Q-APP-6).

**Why.** One press accepts the usual case; the Plan header still counts the copies left out.

**Applies to.** [duplicates](../app/duplicates.md#choosing), [Repeated orders](../ui/duplicates.md).

## D-APP-23 — One copy of a repeated order stays

**Decision.** Exactly one copy of each repeated order is written; there is no *keep both* (Q-APP-7).

**Why.** A repeat is one order to pack once; a second label is printed by a reprint.

**Applies to.** [duplicates](../app/duplicates.md#choosing).

## D-APP-24 — A workspace can be renamed and deleted

**Decision.** Settings › Workspaces renames or deletes a workspace (R-APP-8). **Rename** renames its folder;
everything in it stays, and its journals keep the name in their `started` record, so the fold checks the date
of `started` and no longer its workspace. **Delete** moves the folder, with all its dates, to the Recycle Bin.
The open workspace follows: renamed, it stays open under the new name; deleted, the app opens the one it would
open at start, or asks for a first workspace when none is left. An open draft of it is dropped, after asking.

**Why.** The owner asked for it. The journal is never rewritten, so a rename cannot change what it recorded;
the folder is the workspace's name. The Recycle Bin lets a delete by mistake be undone in Explorer.

**Applies to.** [workspaces](../app/workspaces.md), [journal](../data/journal.md), [Settings](../ui/settings.md).
