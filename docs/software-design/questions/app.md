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
