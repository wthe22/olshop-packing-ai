# Decisions: app — accounts

D-APP-27, D-APP-29, D-APP-37: accounts, an order's account, the summaries of one account. The other APP records are
in [app.md](app.md) and [app-features.md](app-features.md). Where a record names a former question (`Q-…`),
the owner chose that option.

## D-APP-27 — Accounts replace workspaces; one folder per date

**Decision.** A shop owner's shops are an **account** (Q-APP-12): its name and shop names, under `accounts`
in `config.toml`. The data folder holds one folder per date for every account, with three files;
batch numbers restart at 1 each date (Q-APP-14) and the already-saved guard holds across accounts. A batch may
hold several accounts' orders; each order names its account and shop. Rename and Delete act on the saved
orders of its shops too (D-APP-37). v0.4's
workspace folders and v0.1 data are not read, changed or deleted, and the app says what it found (Q-APP-13). It replaces D-APP-5, D-APP-17 and D-APP-24.

**Why.** The owner's words (2026-11-11): one read takes every shop's PDFs, so New batch asks nothing first,
and one count per date is what is printed. *Owner* on screen was confused with the owner of the app. The account list belongs to
the program, as the owner chose.

**Applies to.** [accounts](../app/accounts.md), [data](../data/README.md),
[files](../data/files.md).

## D-APP-37 — An order's account is its shop's account now; an order of no listed shop is under *No account*

**Decision.** The longest listed shop name found in a copy's sender names its shop. An order's account is not
stored: it is the account that lists its shop in `config.toml` now, wherever it is shown, counted, filtered or
summarised. A sender no account lists is asked about once per name before the plan: an account, or **New
account** named after the shop, written into `config.toml` at once. A shop belongs to one account only
(Q-APP-16). An order whose shop no account lists (a shop taken off its account, an order saved before shops
were kept, a label with no sender) is under **No account**: not in `config.toml`, never made, renamed or
deleted, and shown, searched and summarised like any account. A label with no sender is not asked about.
Rename and Delete change the account of the saved orders of its shops too. It replaces D-APP-28 and the
rename and delete of D-APP-27.

**Why.** The owner's words (2026-11-27): shops belong to the account that lists them now; a history of
accounts is noise. Every shipping label of the samples has a sender; the one page without one is a packing
list, which stops the batch (D-LABEL-28). An order no account claims must still be found.

**Applies to.** [accounts](../app/accounts.md#the-account-of-an-order), [reading](../labels/reading.md#the-sender),
[02 Read](../workflow/02-read.md), [New batch](../ui/new-batch.md#the-shop-prompt), [files](../data/files.md).

## D-APP-29 — One account's summary is made on request; a summary is opened, not printed in the app

**Decision.** The batch's summary PDF of all accounts is saved and verified as before. An account's
summary of a batch, and the day summary of all accounts or of one, are made on request from the files,
checked, and not recorded (Q-APP-15). Summary PDFs have **Open** and no **Print**; group PDFs and reprints
keep **Print** (D-APP-26).

**Why.** The owner's words: *data only, final summary generated on demand, derived from data*, and no
**Print** on summaries. One summary per account at every save would double the files for a
paper that is seldom needed.

**Applies to.** [summary PDF](../labels/summary-pdf.md#one-account), [day summary](../labels/day-summary.md),
[Day](../ui/day.md).
