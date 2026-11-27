# Decisions: app — sources, data and checks

D-APP-1 to D-APP-12 and D-APP-32. The app's features are in [app-features.md](app-features.md).

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
version. A copy reads every version of its own major and passes over what it does not know (D-APP-34).
Data of an older major is not read and not changed (D-APP-27: nor deleted; a separate
script copies v0.4 dates once), except by a migration the owner starts (D-APP-33). A schema is raised once
for each release the owner's data moves to, never for a development step between two (R-APP-18): the
owner's data is at v0.7 (`session.toml` `2.0`), so everything v0.8 changes is `day.toml` `2.1` (D-APP-35).

**Why.** The owner's choice: the data is a working set, and the labels can be read again.

**Applies to.** [data](../data/README.md#types-versions-and-names), [conventions](../conventions.md).

## D-APP-4 — Order data comes from the label PDFs only

**Decision.** The platform's order export is not read, by the app or by any rule. Every order field comes from
the label and its packing slip; `orders.csv` is a view the app writes.

**Why.** The owner's choice: the export is not part of this project any more.

**Applies to.** [reading](../labels/reading.md), [filter language](../rules-file/conditions.md).

## D-APP-6 — The courier list is part of the parser file

**Decision.** The courier list and the clues that name each courier are in `parser.toml` ([D-APP-30](app-features.md)).
This replaces D-LABEL-1 (a `couriers.toml` of its own).

**Why.** The owner's choice: the couriers are part of how labels are read, with the logos and shapes the
app learns.

**Applies to.** [parser file](../rules-file/parser.md), [courier deduction](../labels/courier.md).

## D-APP-7 — The journal is the only truth

**Decision.** Each date has one `journal.jsonl` of sealed records, for every account; a commit is one append and
`sync_all`; a torn last line is cut and recorded; damage stops the date. It replaces `state.json`.

**Why.** The owner asked for no duplicated truth, ACID without a database, and every value traceable to its
source. An append-only file of sealed lines commits atomically, shows damage, and keeps what happened.

**Applies to.** [journal](../data/journal.md), [records](../data/records.md).

## D-APP-8 — The draft's data files are views of the journal

**Decision.** `day.toml`, `orders.csv` and `order-contents.toml` are written from the journal after every
commit, never read for a decision, and rebuilt when missing or when `journal_seq` is stale.

**Why.** The owner's files stay readable by people and other programs, and there is still one truth.

**Applies to.** [views](../data/views.md).

## D-APP-9 — Copies of the app share a data folder; one writes at a time

**Decision.** Several copies of the app may use one data folder at once. The data folder's `.lock` is held
with `File::try_lock` only for a write; another copy's write waits for it. A write folds the journal again
when it grew since it was read, and the append refuses a journal that grew meanwhile. Each copy looks for
the others' writes each time the window gets the focus and shows them.

**Why.** The owner wants several people to work on one data folder at the same time. Two writers of one
journal would interleave their commits; one at a time, each deciding on the journal as it is on disk,
cannot.

**Applies to.** [data › One writer](../data/README.md#one-writer), [program › Several copies](../app/program.md#several-copies).

## D-APP-10 — Inputs are stored once, by content

**Decision.** An input is kept in its date folder as `{batch} Input {pdf} {short hash}.pdf` —
`2 Input 03 61459e8a8203.pdf`: the batch whose record names it, the word *Input*, its `PdfID` on the date in
two digits, and the first 12 characters of its SHA-256 (R-APP-13). The same bytes named by a second batch are
stored again under that batch's number. The journal keeps its original name, size and page count. One file
chosen twice is one input. An input whose hash no longer matches is reported, never used.

**Why.** Two downloads with one name cannot overwrite each other, the same bytes are never read twice, and an
amend, a reprint or a restore reads exactly the bytes that were verified. Sorted by name, as Explorer lists
them, a batch's inputs stand beside its outputs in the order they were first read, and the short hash is the one the summary
PDF prints beside each input.

**Applies to.** [02 Read](../workflow/02-read.md#3-choose-the-files), [04 Save](../workflow/04-save.md#11-store-the-inputs).

## D-APP-11 — Outputs are written aside, verified, committed, then moved; a lost one is restored

**Decision.** Outputs are written to `.partial/` of the date folder, verified there, then the journal commit,
then moved into the date folder. On open, the journal decides whether `.partial/` goes forward or back. A lost
output is written again from the inputs and the journal.

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

## D-APP-32 — One flat folder per date

**Decision.** Every file of a date sits directly in its date folder: the journal, the views, the inputs, the
group PDFs, the summary PDF, the reprints and the summaries made on request (R-APP-12); there is no `input/`
and no `output/`. What an operation writes waits in `.partial/` of the date folder until its commit
(D-APP-11); an input is copied to `{name}.partial` beside its name and renamed. The names: `{batch} Input
{pdf} {short hash}.pdf` (D-APP-10), `{batch}-{code} {name} x{count}.pdf`, `{batch} Summary.pdf`,
`{batch} Summary - {account}.pdf`, `Reprint {n} x{count}.pdf`, `Day Summary.pdf` and `Day Summary -
{account}.pdf` (R-APP-14, R-APP-15). A name the journal records stays as recorded. A date folder of v0.7
(one with `input/` or `output/` that no Update has listed flat) is converted by the app's migration
(D-APP-33): each file of its `input/` and `output/` and each summary made on request is moved up under its
new name, checked by its hash, and listed in the Update's `update.toml` (a label PDF under the name of every
batch whose record names its bytes, the outputs unchanged, the summaries renamed, `output/.partial/` as
`.partial/`; [D-APP-36](versions.md)); an emptied `input/` or `output/` is removed. A `v0.7 backup/` left
by v0.8.8 or older without `backup complete.txt` is converted from there the same way; one with it is the
owner's, and `tools/remove-v07-backup.py` deletes it once every file of it is in the date folder
([development](../../development/README.md)). In read-only mode the Day screen names each date not yet flat.

**Why.** The owner's choice: one Explorer window shows a date's whole work, a batch's files side by side by
its number. `.partial/` keeps a save whole as before. The journal is the truth and is never rewritten, so a
name it records keeps its case.

**Applies to.** [data](../data/README.md), [journal](../data/journal.md#when-a-date-opens),
[04 Save](../workflow/04-save.md), [05 After the save](../workflow/05-after-save.md),
[accounts › Data of older versions](../app/accounts.md#data-of-older-versions).

## D-APP-33 — Migrations ask first; read-only writes nothing

**Decision.** The app never migrates data on its own. A migration is a new version of a stored file (a
view of an older `schema`) or data filled in from the stored PDFs (a field a newer reader records). When
the data folder needs one, the app asks at its start: what would change (which dates, how many batches),
with **Update** and **Open read-only**; choosing another data folder asks the same. Update shows its
progress: the step, the date and file, and how far (*Reading labels: 2026-11-25, batch 1, page 3 of
120*). **Read-only** writes nothing at all in the data folder: no migration, no lock file, no rules file
from the bundled copy, no recovery of a stopped save, no view, no batch, revert, amend, reprint, mark or
summary; the screens do not offer them, and the back end refuses every write. It lasts until the program
starts again. Outside read-only mode, recovery and upkeep that change no file's version stay automatic:
settling `.partial/` and writing a missing or stale view (journal › When a date opens).

The migrations, in this order for each date, after the files they change are copied and the moves listed
([D-APP-36](versions.md)): a date folder of v0.7 is made flat (D-APP-32); the views of
another `schema` are written again; the receiver's area and the weight of each order of a live batch saved
before they were read are read from the batch's stored label PDFs, an `areas_filled` and a `weights_filled`
record per batch (D-LABEL-26, D-LABEL-27). A batch whose label PDF is missing is not offered (it cannot be read); one whose
label PDF has changed is skipped and named, and the next start asks again. A damaged journal is passed
over. The program's own settings, kept in `packing-app.toml` beside the program, still change in read-only
mode: the theme, and **Change** of the data folder, which opens the folder chosen as at start and asks again
when it would be updated. The accounts, sort rules and logos are the data folder's and do not change.

**Why.** The owner's rule: no surprises; a long migration must not look like a frozen program, and the
owner must be able to look at data without changing it.

**Applies to.** [data](../data/README.md), [journal](../data/journal.md#when-a-date-opens),
[views](../data/views.md), [01 Start](../workflow/01-start-and-open.md#1-start),
[Start screen](../ui/start.md#update-the-data-folder).
