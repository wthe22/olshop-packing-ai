# Data rework: the screens read the views, an account is its shop's

The owner's answers of 2026-11-27: [D-APP-37](../../software-design/decisions/accounts.md) (an order's
account is the account that lists its shop now; *No account* for the rest) and
[D-APP-38](../../software-design/decisions/app.md) (the screens read the views; the journal is read to
write, recover, migrate and audit; a hash names a view changed by hand; a date with no journal still works).
T1 (an order's account looked up from its shop) and T2 (a view changed by hand is named) are done. Tasks in order; each is one commit, under the code rules ([code-rules.md](../code-rules.md): one job per
module, size ceiling, no over-generalising, extract on the third copy, comments only for the not obvious,
proper types). Tests: known-answer fixtures through read → plan → save → verify, tables for fixed truths,
broken outputs the verifier must catch, coverage not lower.

## T3 — The screens read the views

- A store function reads the three views of a date into one value (no journal).
- The date list, the Day screen, Search's index and the Accounts counts, the reprint list and the
  summaries read that value instead of folding the journal. The journal stays the source of a commit and
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
