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

## Q-APP-27 — Archive mode

The owner wants, later, to keep a date with as little storage as possible: its label PDFs (inputs) and its
group, summary and reprint PDFs (outputs) removed, `day.toml`, `orders.csv` and `order-contents.toml` kept.
Every value still traces to its order, but no longer to a page that can be opened. What is decided now
([D-UI-19](../decisions/ui-screens.md)): the Day cards, the group contents, the [Summary](../ui/summary.md)
screen, Search and the Accounts counts read the three files only, so they work on such a date.

**How a date is archived**

- **A. Archive on the Day screen, one date at a time.** Asked first: *Remove the label PDFs and the
  printed PDFs of Wed, 7 Oct 2026? Its orders and summaries stay; it can no longer be printed, amended or
  read again.* `day.toml` gets `archive_time`. *(Default)*
- **B. A in Settings for every date older than a number of days**, asked at the start like a migration and
  shown with its progress ([D-APP-33](../decisions/app.md)).

**What an archived date shows** (both options)

- The Day screen: a muted line *Archived 27 Nov 2026: the PDFs were removed.* Each group keeps its counts,
  Sent time, printed tick and **▸**; **Print**, **Open**, **Open folder**, **Restore**, **Amend**, **Revert**,
  **Read labels again**, **Check data** and **+ New batch** are not shown, nor the missing-file and
  missing-input warnings. **Summary** and **Day summary** open the Summary screen without **PDF**.
- Search: its orders are found and shown as now, without a tick for **Reprint** and without **Open**; the
  trace names the input and pages that were.
- The verifier and a reprint never read an archived date; an order saved on it still counts as saved for
  the already-saved guard.
- Nothing archives on its own, and read-only mode archives nothing.
