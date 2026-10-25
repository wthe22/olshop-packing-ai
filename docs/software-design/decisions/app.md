# Decisions: app — sources, data and checks

D-APP-1 to D-APP-12. The app's features are in [app-features.md](app-features.md).

## D-APP-1 — PDFium reads, copies and writes the PDFs

**Decision.** PDFium reads the text of a page with its positions, copies pages unchanged into the group PDFs
and writes the summary PDF. It does not check them (D-APP-12).

**Why.** On the sample downloads it read the Order ID on every page and gave the same slip lines as an
independent reading; copied pages render with no pixel difference; reading 601 pages takes 1.6 s. Its
written files embed whole fonts (about 1.4 MB); they print the same, and the size is accepted.

**Applies to.** [architecture](../app/architecture.md), [summary PDF](../labels/summary-pdf.md).

## D-APP-3 — Data carries an X.Y schema version; no migration

**Decision.** Every TOML file and the journal's first record carry `schema` as an `X.Y` version, starting at
`1.0`. The schema version is not the app version: app `v0.2` writes `1.0`, and a schema change raises the app
version. Data written by an older version is not read and not migrated (D-APP-17: nor deleted).

**Why.** The owner's choice: the data is a working set, and the labels can be read again.

**Applies to.** [data](../data/README.md#types-versions-and-names), [conventions](../conventions.md).

## D-APP-4 — Order data comes from the label PDFs only

**Decision.** The platform's order export is not read, by the app or by any rule. Every order field comes from
the label and its packing slip; `orders.csv` is a view the app writes.

**Why.** The owner's choice: the export is not part of this project any more.

**Applies to.** [reading](../labels/reading.md), [filter language](../rules-file/conditions.md).

## D-APP-5 — A namespace replaces the session

**Decision.** The session (a day plus an optional name) is replaced by the namespace: one folder per owner,
one folder per date inside it. Batch numbers run through a namespace and date. There is no session name and
no *Main*.

**Why.** The owner's choice: the session concept does not suit this workflow.

**Applies to.** [namespaces](../app/namespaces.md), [data](../data/README.md).

## D-APP-6 — The courier list is part of packing-workflows.toml

**Decision.** The courier list and the clues that name each courier are in `packing-workflows.toml`; there is
no `couriers.toml`. This replaces D-LABEL-1 (one courier file in the data folder).

**Why.** The owner's choice: a separate courier file has no purpose once the workflows file exists.

**Applies to.** [rules file](../rules-file/README.md#couriers), [courier deduction](../labels/courier.md).

## D-APP-7 — The journal is the only truth

**Decision.** Each namespace and date has one `journal.jsonl` of sealed records; a commit is one append and
`sync_all`; a torn last line is cut and recorded; damage stops the date. It replaces `state.json`.

**Why.** The owner asked for no duplicated truth, ACID without a database, and every value traceable to its
source. An append-only file of sealed lines commits atomically, shows damage, and keeps what happened.

**Applies to.** [journal](../data/journal.md), [records](../data/records.md).

## D-APP-8 — The draft's data files are views of the journal

**Decision.** `session.toml`, `orders.csv` and `order-contents.toml` are written from the journal after every
commit, never read for a decision, and rebuilt when missing or when `journal_seq` is stale.

**Why.** The owner's files stay readable by people and other programs, and there is still one truth.

**Applies to.** [views](../data/views.md).

## D-APP-9 — One copy of the app per data folder

**Decision.** The data folder's `.lock` is held with `File::try_lock` while the app runs; a second copy says
which one has it and writes nothing.

**Why.** Two writers of one journal would interleave their commits.

**Applies to.** [data › One writer](../data/README.md#one-writer), [Start](../ui/start.md).

## D-APP-10 — Inputs are stored once, by content

**Decision.** An input is kept as `input/{sha256}.pdf`; the journal keeps its original name, size and page
count. One file chosen twice is one input. An input whose hash no longer matches is reported, never used.

**Why.** Two downloads with one name cannot overwrite each other, the same bytes are never read twice, and an
amend, a reprint or a restore reads exactly the bytes that were verified.

**Applies to.** [02 Read](../workflow/02-read.md#3-choose-the-files), [04 Save](../workflow/04-save.md#11-store-the-inputs).

## D-APP-11 — Outputs are written aside, verified, committed, then moved; a lost one is restored

**Decision.** Outputs are written to `output/.partial/`, verified there, then the journal commit, then moved
into `output/`. On open, the journal decides whether `.partial/` goes forward or back. A lost output is
written again from `input/` and the journal.

**Why.** A batch is saved whole or not at all even when the app stops between two steps, and an output is
reproducible, so losing one costs nothing.

**Applies to.** [04 Save](../workflow/04-save.md), [journal](../data/journal.md#when-a-date-opens), [reprint](../app/reprint.md#restore).

## D-APP-12 — An independent verifier checks every output

**Decision.** Crate `packing-verify`, whose only PDF library is `lopdf` and which uses none of the crates that
read and write, checks every save, reprint and restore by page digest, ledger and counts. It replaces D-APP-2
(the read-back with the app's own page reader).

**Why.** Correctness first, with no single point of failure: what checks an output is never the code that made
it. On 1,309 pages, every page PDFium copied kept its `lopdf` digest and every page had a digest of its own.

**Applies to.** [verification](../app/verification.md), [architecture](../app/architecture.md#who-may-use-whom).
