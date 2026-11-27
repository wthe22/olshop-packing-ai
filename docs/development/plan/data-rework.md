# Data rework: the screens read the views, an account is its shop's

The owner's answers of 2026-11-27: [D-APP-37](../../software-design/decisions/accounts.md) (an order's
account is the account that lists its shop now; *No account* for the rest) and
[D-APP-38](../../software-design/decisions/app.md) (the screens read the views; the journal is read to
write, recover, migrate and audit; a hash names a view changed by hand; a date with no journal still works).
Tasks in order; each is one commit, under the code rules ([code-rules.md](../code-rules.md): one job per
module, size ceiling, no over-generalising, extract on the third copy, comments only for the not obvious,
proper types). Tests: known-answer fixtures through read → plan → save → verify, tables for fixed truths,
broken outputs the verifier must catch, coverage not lower.

## T1 — An order's account is looked up from its shop

- The journal stops writing `SavedOrder.account` (journal `2.1`, the one raise of v0.8: read with
  `#[serde(default)]`, never written, never used). The account comes from `config.toml` wherever it is
  needed: account counts and `of_account` take the accounts list; an order whose shop no account lists is
  under **No account** (one constant, a name no account may take).
- The shop prompt asks only about shops no account lists: the per-file *No shop name* rows go
  (`engine/account_questions.rs`, `commands/batch/shops.rs`, `ShopPrompt.svelte`). An amend carries no account.
- Search and its index, the Accounts screen's counts, the sort rules' `account` field, the batch and day
  summaries, and the account summary buttons look the account up. Rename and Delete then act on saved orders
  without a write to any journal; the Accounts screen's texts follow `ui/accounts.md`.
- The verifier reads `config.toml` itself for V4 (no shop under two accounts) and V6 (an account's
  summary: the orders whose shop it lists; *No account*: the rest). It still uses no other crate.
- Tests: a table of shop → account (listed, renamed, deleted, none, no sender); an account summary whose
  account was renamed after the save; a broken account summary the verifier must catch.

## T2 — `day.toml` holds the views' hashes; a view changed by hand is named

- `write_views` writes `orders_csv_sha256`, `order_contents_sha256` and, as the last line, `sha256` of the
  text before it (`data/views.md`). A store function reads the three views of a date into one value (no
  journal) and says which file's hash differs.
- The Day screen names a differing view with **Rebuild**, **Read only** and **Ignore** (D-APP-38); Read only
  lasts until the program starts again and refuses every write to that date; Ignore lasts for the session.
- Tests: views written then read back equal the fold's; each view edited by a byte is named; a `day.toml`
  without hashes (an older copy) is not checked.

## T3 — The screens read the views

- The date list, the Day screen, Search's index and the Accounts counts, the reprint list and the
  summaries read the value of T2 instead of folding the journal. The journal stays the source of a commit and
  its checks, the already-saved guard, amend, revert, restore, the migrations, recovery, the verifier and the
  journal notes.
- Tests: save a batch, delete `journal.jsonl`, and the Day screen's data, Search's results, the reprint
  list and the day summary are the same as before.

## T4 — A date with views and no journal takes saves

- Such a date is shown from its views with a line saying its journal is missing. Its first commit asks
  first, then starts the journal with a `restored` record: the views' batches as they state them. A restored
  batch can be reprinted and summarised, not amended or reverted; batch, input and content numbers go on
  after the views'.
- Tests: views of two batches, no journal, a third batch saved: the views hold all three, the third
  verifies, and amend and revert offer only the third.
