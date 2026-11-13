# Decisions: app — accounts

D-APP-27 to D-APP-29: accounts, an order's account, the summaries of one account. The other APP records are
in [app.md](app.md) and [app-features.md](app-features.md). Where a record names a former question (`Q-…`),
the owner chose that option.

## D-APP-27 — Accounts replace workspaces; one folder per date

**Decision.** A shop owner's shops are an **account** (Q-APP-12): its name and shop names, under `accounts`
in `config.toml`. The data folder holds one folder per date for every account, with one journal;
batch numbers restart at 1 each date (Q-APP-14) and the already-saved guard holds across accounts. A batch may
hold several accounts' orders; each order names its account and shop. Rename changes the name for later
reads; Delete removes the account and its shops, and its saved orders stay under its name (Q-APP-17). v0.4's
workspace folders and v0.1 data are not read, changed or deleted, and the app says what it found (Q-APP-13). It replaces D-APP-5, D-APP-17 and D-APP-24.

**Why.** The owner's words (2026-11-11): one read takes every shop's PDFs, so New batch asks nothing first,
and one count per date is what is printed. *Owner* on screen was confused with the owner of the app. The
journal is never rewritten, so a saved order keeps the name it was saved with. The account list belongs to
the program, as the owner chose.

**Applies to.** [accounts](../app/accounts.md), [data](../data/README.md),
[journal](../data/journal.md), [records](../data/records.md).

## D-APP-28 — An order's account is found from the shop on its label

**Decision.** The longest listed shop name found in a copy's sender names its shop and account. A sender no
account lists is asked about once per name before the plan: an account, or **New account** named after the
shop, written into `config.toml` at once. A shop belongs to one account only (Q-APP-16). A file
with labels that have no sender is asked per file, for that read only. Every order has an account before the
plan. It replaces D-APP-25.

**Why.** The owner's words (2026-11-11): the account comes from the shop name on the label, and an unknown
name is asked. Only the sender is looked at, since a product name can carry a shop's name; the longest name
wins so `Toko Alfa Official` is not taken for `Toko Alfa`. A name that is not stored for a label with no
sender keeps one guess from deciding every later file.

**Applies to.** [accounts](../app/accounts.md#the-account-of-an-order), [reading](../labels/reading.md#the-sender),
[02 Read](../workflow/02-read.md), [New batch](../ui/new-batch.md#the-shop-prompt).

## D-APP-29 — One account's summary is made on request; a summary is opened, not printed in the app

**Decision.** The batch's summary PDF of all accounts is saved, verified and recorded as before. An account's
summary of a batch, and the day summary of all accounts or of one, are made on request from the journal,
checked, and not recorded (Q-APP-15). Summary PDFs have **Open** and no **Print**; group PDFs and reprints
keep **Print** (D-APP-26).

**Why.** The owner's words: *data only, final summary generated on demand, derived from data*, and no
**Print** on summaries. One summary per account at every save would double the files and records for a
paper that is seldom needed.

**Applies to.** [summary PDF](../labels/summary-pdf.md#one-account), [day summary](../labels/day-summary.md),
[Day](../ui/day.md).
