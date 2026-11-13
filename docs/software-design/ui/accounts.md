# Accounts

One account at a time: its shops, its orders per date, and its name ([D-UI-13](../decisions/ui.md),
[accounts](../app/accounts.md)). **Accounts** in the title bar opens it over the current screen, in the look
of [Sort rules](sort-rules.md); **×**, a click beside it or `Esc` returns.

```
┌ Accounts ──────────────────────────────────────────────────────────────────────────── [×] ┐
│ [ Toko Alfa ][ Toko Beta ]  + New account                                                 │
│ ┌ Toko Alfa                                          [Rename]  [Delete account] ──────┐ │
│ │ SHOPS  The labels of these senders are this account's orders.                          │ │
│ │ [ Toko Alfa                       ] [×]                                                │ │
│ │ [ Toko Alfa Official              ] [×]                                                │ │
│ │ + Add shop                                                                             │ │
│ ├────────────────────────────────────────────────────────────────────────────────────────┤ │
│ │ DATES                                                       Orders    Batches          │ │
│ │ Wed, 21 Oct · today                                       88 orders  2 batches      ›  │ │
│ │ Tue, 20 Oct                                              301 orders  3 batches      ›  │ │
│ │ ▸ September 2026                                       4 120 orders                    │ │
│ └────────────────────────────────────────────────────────────────────────────────────────┘ │
│                                                    Unsaved changes  [Cancel]  [Save]      │
└───────────────────────────────────────────────────────────────────────────────────────────┘
```

## The accounts

- **The tabs** are the accounts of `config.toml`, in the order they were made, side by side as
  Settings' theme choice; the account shown is raised. It opens on the first account. With none yet it says
  *No accounts yet. The first read of labels asks which account each shop belongs to.* and **+ New account**.
- **+ New account** asks for a name in a dialog over the screen, with **Create** and **Cancel**; a name the
  rules refuse ([accounts › Names](../app/accounts.md#names)) is told under the field. The new account has no shop
  and is shown.
- **Another tab**, or **+ New account**, while the shops have unsaved changes asks first: *Discard the
  changes to Toko Alfa's shops?* [Discard] [Cancel].

## The account's card

- **The head**: the account's name, **Rename** and **Delete account**.
  - **Rename** puts the name in a field in place, with **Cancel** and **Rename**; Enter renames, `Esc`
    cancels. A refused name is told under the field. The line under it says: *Orders saved before keep the
    name they were saved with.*
  - **Delete account** asks "Delete account Toko Beta?" — "Its shops are asked about again at the next read.
    Its saved orders stay, under its name." [Delete account] [Cancel].
- **Shops**: one field per shop name, its **×**, and **+ Add shop**. A name listed twice, or listed by
  another account (spaces and case ignored), is told under its field and keeps **Save** off, with the
  reason as its tooltip ([accounts › Names](../app/accounts.md#names)). An account with no shop says
  *No shops: only a label with no sender can be put here.*
- **Dates**: the dates with orders of this account, newest first, months folded as in the [panel](panel.md);
  each with the account's orders and the batches that hold them, as count columns. A folded month shows its
  orders. A click on a date opens it on the Day screen and closes this page; a draft being prepared asks
  first. The counts come from the search index (*Loading* until it is built) and are the orders saved
  under this name.
- **The foot** stays in view: *Unsaved changes*, **Cancel** and **Save** for the shops, as on Sort rules.
  Save writes `config.toml` under the data folder's lock.
