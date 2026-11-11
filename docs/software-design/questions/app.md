# Open questions: APP

Each question gives the problem, the options, and the **Default**: what the work does until the owner decides.
Raised 2026-11-11 with [v0.5](../../development/plan/v0.5-accounts.md). Answered questions are removed, and
their answer goes into [decisions/](../decisions/README.md).

## Q-APP-12 — The word for a shop owner's group of shops

The owner wrote *account/owners*. *Owner* is the word the documents use for the person who runs the app, and
D-APP-17 records that *owner* on screen was confused with that.

- **A. Account** — on screen *Account*, in the design and the code `account`. *(Default)*
- **B. Owner** — on screen *Owner*; the docs say *the owner* in two senses, so the app's owner is then called
  something else in every document.

## Q-APP-13 — The dates of v0.4 workspaces

v0.5 keeps one folder per date for the whole data folder. The dates of each v0.4 workspace (its journals and
batch numbers) cannot be read as one journal.

- **A. Not read.** The old folders stay on disk, untouched, and the Start screen says what it found, as it
  does for v0.1 data. The old dates are then only in the old copy of the program. *(Default)*
- **B. One-time migration** started by the owner: each old workspace's dates are written into the new layout,
  batch numbers renumbered per date, with a new journal. The old folders stay as a backup and nothing is
  deleted. Printed labels keep the numbers they were printed with, so reprints and search work for old
  orders only if the numbers are kept.

## Q-APP-14 — Batch numbers across dates

- **A. 1, 2, 3, … per date**, across all accounts; a new date starts at 1. *(Default; as today, per workspace.)*
- **B. One running number** across all dates.

## Q-APP-15 — The summary PDFs of one account

Today a batch's summary is written, verified and recorded in the journal. An account's summary is another
document of the same batch, and the day summary is a view written on request.

- **A. All accounts: stored and verified as today.** An account's summary (batch or day) is written on request
  from the record, verified against it, and kept as a file but not recorded: it is a view.
  Example file: `output/3 summary Toko Alfa.pdf`. *(Default)*
- **B. Every account's summary stored and recorded** like the all-accounts one: more outputs per batch and
  more journal records.

## Q-APP-16 — A shop name no account lists

Asked once per distinct shop name in a read (a line per file that has it), before the plan, the way the logo
prompt is asked.

- **A. Choose the account, or New account** (named after the shop, editable). A shop name belongs to one
  account only: listing a name another account has is refused. A file with no sender is asked the same way,
  but its name is not stored. *(Default)*
- **B. Choose per file**, not per name: the same shop in two files is asked twice. Nothing is stored.

## Q-APP-17 — Deleting an account

- **A. Delete removes the account and its shop names.** Its saved orders stay in the journals under the
  account's name and stay searchable; a journal is never rewritten. *(Default)*
- **B. Archive:** the account is hidden from the lists and its shops stop matching, and it can be restored.
  Its orders stay as in A.
