# Open questions: UI

Each question gives the problem, the options, and the **Default**: what the work does until the owner decides.
Raised 2026-11-11 with [v0.5](../../development/plan/v0.5-accounts.md).

## Q-UI-9 — The page of one account

The owner asked for a new page that shows and manages one account at a time: its daily order counts, its
shops, rename and delete (moved from Settings › Workspaces).

- **A. An Accounts screen**, opened from the title bar beside Search, Sort rules and Settings. An account
  chooser sits at its top; below it the account's dates with its orders per date, its shops (add, remove), and
  **Rename** and **Delete** at the foot. *(Default)*
- **B. An account section of the panel**: the account's dates and counts in the panel, its shops and its
  rename and delete in a dialog from the account's row menu. Nothing more in the title bar.

## Q-UI-10 — The panel without a workspace

The panel lists the dates of the data folder. Today its counts are one workspace's.

- **A. All accounts together.** The date rows show the orders of all accounts; the per-account counts are on
  the Accounts screen. *(Default)*
- **B. An account selector in the panel** (*All accounts* or one account) that narrows the date counts. The
  Day screen still shows every batch of the date, since a batch may hold several accounts.
