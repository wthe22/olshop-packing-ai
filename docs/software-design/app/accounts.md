# Accounts and dates

An **account** is one shop owner's group of shops ([D-APP-27](../decisions/accounts.md)). Every order
names the shop its label was sent by; its account is the account that lists that shop now
([D-APP-37](../decisions/accounts.md)). The data folder holds one folder per date for all accounts
together ([data](../data/README.md#the-data-folder)): a batch may hold the orders of several accounts.

| Word | Meaning |
|---|---|
| **Account** | A name and the shop names its labels are sent by, in `config.toml` |
| **Shop** | A name an account lists, as the label prints it after `Pengirim` ([reading](../labels/reading.md#the-sender)) |
| **Date** | A calendar date of the data folder: one folder with its three files, inputs and outputs |

## What a date holds for all accounts

- The batches, numbered 1, 2, 3, … per date; a new date starts at 1.
- The **already-saved guard**: an order saved in a live batch of a date is left out of a later batch of
  that date, whatever its account ([03 Plan](../workflow/03-plan.md#7-orders-already-saved)).
- The *used in batch n* hint, revert and amend: each date has its own newest batch.
- The reprints ([reprint](reprint.md)). The search index holds every date ([reprint](reprint.md#the-search-index)).

## The accounts file

The accounts are the key `accounts` of `config.toml` in the data folder
([data](../data/README.md#the-files)), in the order they were made:

```toml
[[accounts]]
name = "Toko Alfa"
shops = ["Toko Alfa", "Toko Alfa Official"]

[[accounts]]
name = "Toko Beta"
shops = ["Toko Beta"]
```

- Written by the Accounts screen ([Accounts](../ui/accounts.md)) and by the shop prompt, under the data
  folder's lock, each write reading the file again first so another copy's change is kept. Read when labels
  are read and by the Accounts screen; never to decide what a saved date holds.
- It travels with the data folder: every copy that uses the folder shares the list ([D-APP-30](../decisions/app-features.md)),
  and a data folder given to another PC brings its accounts.

## Names

- An account's name follows the default file-name rule ([data](../data/README.md#types-versions-and-names))
  and is 1–40 characters. Two accounts cannot share a name (upper and lower case count as the same).
- A shop name is 1–80 characters. **A shop belongs to one account only**: a name that another account, or
  the same one, already lists (spaces and upper and lower case ignored) is refused.

## The account of an order

- A copy's **shop** is the listed name found in the sender of its first page, spaces and upper and lower case
  ignored ([reading](../labels/reading.md#the-sender)). When several listed names are found, the longest
  wins (`Toko Alfa Official` before `Toko Alfa`), then the one listed first. Its **account** is the account
  that lists that shop in `config.toml` now: not stored, looked up wherever an order is shown, counted,
  filtered or summarised ([files](../data/files.md)).
- **No account** holds every order whose shop no account lists: a shop taken off its account, a label with
  no sender. An order saved before shops were kept (v0.4) gets its shop from its label by the Update
  ([D-APP-33](../decisions/app.md)). It is not in `config.toml`, cannot be made,
  renamed or deleted, and no account can take its name; Search, the counts and the summaries show it like
  any account, and only while it has orders.
- **A sender no account lists** is asked about before the plan, once per name: an account, or **New
  account**, named after the shop unless the owner types another name. The answer is written into the accounts file at once, so the name is
  asked about once ever ([New batch](../ui/new-batch.md#the-shop-prompt)).
- **A label with no sender** (no `Pengirim` caption) is not asked about: its order has no shop and is under
  **No account**.
- The plan waits for the answers to the shop prompt.

## Creating, renaming and deleting

- **+ New account** on the Accounts screen, or **New account** in the shop prompt, adds an account with that
  name (and the shop, from the prompt).
- **Rename** changes the name of the account of every order of its shops, saved ones too.
- **Delete** (asked first) removes the account and its shop names from the accounts file. Its saved orders
  are then under **No account**; their shops are asked about again at the next read.

## Listing and opening dates

- **The dates** are the folders of the data folder named as a date (`2026-10-07`), read at start and when
  the window regains focus, so a folder added in Explorer shows up.
- **At start** the app opens today's date (the PC's date). Another date is opened from the
  [panel](../ui/panel.md): a download just after midnight that belongs to the evening before is saved under
  that evening's date.
- **A date folder** is created at its first commit; today is listed before it exists. A date is never renamed
  or deleted by the app.

## Data of older versions

The app reads only the date folders and its own `parser.toml` and `config.toml`, and never changes or
deletes anything else in the data folder. It says
what it found, a line on the Day screen with **Open folder** ([Messages](../ui/messages.md)):

- **v0.1**: `labels/`, `state.json`, `couriers.toml`, `categories.toml` or `categories/`.
- **v0.4**: a folder not named as a date that holds a `settings.toml` (a v0.4 workspace).
- **v0.7**: a date folder that holds `input/` or `output/`. Its inputs and outputs
  are not found until the migration has moved them ([D-APP-32](../decisions/app.md)); as the program asks
  for it at its start, the line shows only in read-only mode, and says to start again and choose **Update**.
