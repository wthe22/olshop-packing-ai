# Open questions: APP

Each question gives the problem, the options, and the **Default**: what the work does until the owner decides.
Answered questions are removed, and their answer goes into [decisions/](../decisions/README.md).

## Q-APP-18 — The saved orders of a renamed account

An order keeps the account's name it was saved with: the journal is never rewritten
([D-APP-27](../decisions/accounts.md)). After **Rename** of *Toko Alfa* to *Toko Alfa Jaya*, the orders saved
before are still *Toko Alfa* in Search and in the summaries, and the Accounts screen counts only the orders of
the name the account has now.

- **A. The old name stays on the old orders.** Search finds them with `account equals "Toko Alfa"`; the
  Accounts screen shows *Toko Alfa Jaya* from the rename on. Nothing more is stored. *(Default)*
- **B. The Accounts screen also lists the names of saved orders that no account has now**, below the
  accounts, each with its dates and counts and no shops, so a renamed or deleted account's history stays one
  click away. Nothing more is stored.
- **C. An account keeps its earlier names** in `config.toml`: Search's *account* and the Accounts
  screen count an order saved under an earlier name as the account's. A new account cannot take an earlier
  name.


## Q-APP-24 — A check of a date's data against the current version

A date migrated from an older version, or saved by an older copy, can differ from what this version would write
from the same journal: a file missing (`orders.csv`, `day.toml`, `order-contents.toml`), a column the older
version did not write (the shop, the receiver's area, the weight), or a cell left empty that a label gave. Nothing
tells the owner which. The views are a copy of the journal ([views](../data/views.md)), so most differences can
be made right from the journal; a value the journal never recorded needs the label PDFs read again.

The check compares each date's files with what the current version writes from its journal, and names every
difference: file, column or record, and the order. A difference with no sender on the label (the shop empty)
is not a difference, since that is what the label holds ([accounts](../app/accounts.md#the-account-of-an-order)).

Where the check is shown and what it offers:

- **A. Report only.** The Day screen shows *Check data* for each date; the report lists the differences, and
  nothing is changed. Repair is done by hand.
- **B. Report and regenerate.** As A, plus **Regenerate views** when the differences are only views (a missing
  or older file, a column or an empty cell the journal can fill). It writes the views from the journal under the
  lock, like any commit, and never changes a record. *(Default)*
- **C. B, and re-read.** As B, plus **Read labels again** for a value the journal does not hold (for example the
  shop of an order saved before the shop was kept). It reads the date's input PDFs and writes the difference as
  an amend, so the journal keeps the change. Each re-read is asked first and shows its progress.

Rules that hold for every option: a check writes nothing in read-only mode ([D-APP-33](../decisions/app.md));
a repair is a choice on screen, never automatic at start; a date whose views are of a newer version is only
reported. The check reads the journal and the files and never the PDFs, except for a re-read.

**The case that decides C.** A migrated v0.4 date can hold only its `journal.jsonl`, its input PDFs and its output
PDFs. Then every order's shop is empty in the journal (the v0.4 migration recorded none), and its receiver's area
and weight are not in it either. Only **Read labels again** can fill them, from the sender line and the label
itself. Option B alone would rebuild the toml and csv with those cells empty and say nothing more. If no input PDF
is there, the check names the cells as not recoverable and offers nothing.

Options B and C need a decision on the default:

- **Default for re-reading**: none in B (re-read is opt-in per date, C). Under B the report says *Read labels
  again needs option C*, and on a migrated date with its input PDFs the report offers C.
- Where the check lives: a button on the Day screen per date (*Default*) or one list on Settings › Data over
  all dates.
