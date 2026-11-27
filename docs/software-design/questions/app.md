# Open questions: APP

Each question gives the problem, the options, and the **Default**: what the work does until the owner decides.
Answered questions are removed, and their answer goes into [decisions/](../decisions/README.md).

## Q-APP-18 — The saved orders of a renamed account

An order keeps the account's name it was saved with
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

A date migrated from an older version, or saved by an older copy, can lack what this version reads from a
label: the shop, the receiver's area, the weight. Nothing tells the owner which cells are empty because the
label prints nothing and which because an older version did not read them. Only the stored label PDFs can
fill them.

- **A. Report only.** The Day screen shows *Check data* for each date; the report names each empty cell
  that this version would read, and nothing is changed.
- **B. Report and read labels again.** As A, plus **Read labels again**: it reads the date's stored inputs
  and fills the empty cells of `orders.csv`, asked first, with its progress shown, like a migration. A date
  whose input PDFs are missing names the cells as not recoverable. *(Default)*

Rules for both: nothing is written in read-only mode ([D-APP-33](../decisions/app.md)); a fill is a choice
on screen, never automatic; a date of a newer version is only reported. A cell the label leaves empty (no
sender) is not reported.

## Q-APP-25 — A file of a date changed outside the program

`day.toml` holds the SHA-256 of `orders.csv`, `order-contents.toml` and itself
([files](../data/files.md)). A hash that does not match means the file was changed outside the program,
for example saved by Excel, which turns an Order ID into `5.8E+17`. With no journal there is no second copy
to rebuild the file from.

- **A. Accept or Read only.** The Day screen names the file. **Accept** takes it as it is when it reads and
  agrees with the other two (the next commit writes its hash); **Read only** shows it and writes nothing to
  the date until the program starts again. An Order ID Excel broke stays broken. *(Default)*
- **B. A, plus Read labels again.** As A, plus **Read labels again**: `orders.csv` and
  `order-contents.toml` are written again from the stored inputs of the live batches, in their saved groups
  and printing order. The marks and the reprints of `day.toml` stay.
- **C. Read only, always.** The date takes no write until the owner puts the file back by hand.

## Q-APP-26 — The clue that named an order's courier

Until now each saved order kept `courier_clue` (the logo hash, text or tag that named its courier) in the
journal ([D-LABEL-14](../decisions/label.md)); `orders.csv` 2.1 has no such column. The order's source page
(`pdf_num`, `page_start`) is kept, and reading that page again shows the clue.

- **A. Not stored.** A courier named wrong is traced by reading its page again. D-LABEL-14 goes.
  *(Default)*
- **B. A column `courier_clue` in `orders.csv`**: the kind and the value, `logo:3f9a0c71b2d4e5f6`.
