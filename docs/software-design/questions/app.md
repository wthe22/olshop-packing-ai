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

## Q-APP-21 — The name of `session.toml`

"Session" is a word of v0.1: the glossary calls one date with its journal folded a *Day* and lists
*session* as the word not to use ([code rules](../../development/code-rules.md)). The file holds the
date's inputs, sort rules, batches and reprints, and its `journal_seq` vouches for the other views.

- **A. `day.toml`**: the glossary's word, as on the Day screen and in `Day Summary.pdf`. A date folder
  reads `journal.jsonl`, `day.toml`, `orders.csv`, `order-contents.toml`. *(Default)*
- **B. `batches.toml`**: names what it mostly holds, as `orders.csv` and `order-contents.toml` do; but it
  also holds the inputs, the sort rules and the reprints.
- **C. `manifest.toml`**: names its job, the list of the date's files that vouches for the other views;
  a programmer's word.

Any new name comes with v0.8 ([D-APP-3](../decisions/app.md)): **Update** already writes the views of a
v0.7 date again (`session.toml` 2.0 to 2.1), so it writes the new name and deletes `session.toml`; the
owner is asked nothing more.

## Q-APP-22 — An older copy opens a journal a newer copy wrote

A record type is added without a new journal schema ([records](../data/records.md)). v0.8 adds
`areas_filled` and `weights_filled`, which **Update** appends for each batch saved before; on a date of one
batch the first is line 3. v0.7 knows neither, so it says *The journal of this date is damaged at line 3: it
is not a record. The date can be viewed; nothing is written to it.* The journal is sound; the copy is older.
What v0.7 says cannot change; the choice is what v0.8 and later say to a journal of a newer copy.

- **A. As now**: a line this copy cannot read is damage. Every copy on a shared data folder is updated
  together. Nothing to build.
- **B. Name the newer copy**: a line this copy cannot read whose `app` is newer than this copy says *This
  date was saved by Packing 0.9.0, newer than this copy (0.8.6). Update this copy to work on it.* As now,
  the date shows what comes before that line and nothing is written to it. *(Default)*
- **C. B, and show past it**: the older copy skips a record type it does not know and shows the rest of
  the date, writing nothing to it. More is shown, but a skipped record may change what is shown (a later
  record that moves an order would be missing).
- **D. Guard the data folder**: the data folder keeps the newest app version that wrote it, and an older
  copy opens the whole folder read-only at its start, with one message, before any date. It catches every
  newer change, a new field's meaning too, not only a new record type; one more file in the data folder.

## Q-APP-23 — The version of `orders.csv`

Every TOML file and the journal carry `schema`; `orders.csv` has none. Its columns changed in v0.8
(`province`, `city`, `district`, `weight_grams`), told only by `session.toml` 2.1. A program that reads
`orders.csv` alone cannot tell which columns to expect.

- **A. `session.toml`'s `schema` is the views' version** (renamed by Q-APP-21), written into the design:
  `orders.csv` has the columns of that version; a reader finds a column by its header; a column may be
  added, and is never renamed or given another meaning without a new version. The CSV does not change and
  Excel opens it as now; a copy of `orders.csv` alone carries no version. *(Default)*
- **B. A `schema` column**: the first cell of every row is `2.1`. The file tells its version alone; Excel
  shows one more column.
- **C. The version in the name**: `orders 2.1.csv`. A program built for one version does not find
  another's file; each new version renames it (**Update** deletes the old one), and a link or a query to the
  old name breaks.
- **D. A first line `# schema 2.1`** above the header. Some tools read CSV so, but Excel shows the line as
  a row and most CSV readers must be told to skip it.
- **E. A description beside it**, `orders.csv-metadata.json` in the W3C *CSV on the Web* format: the
  version, the columns and their types. Tools that know the format read it, others ignore it; one more file
  per date.
