# Journal removal: day.toml, orders.csv and order-contents.toml are the truth

The owner's decision of 2026-11-27, [D-APP-39](../../software-design/decisions/app.md): the journal goes;
a date's three files are the truth and hold no history. The design is in
[data/files.md](../../software-design/data/files.md) and [data/save.md](../../software-design/data/save.md).
Open questions, worked to their default until answered: [Q-APP-25](../../software-design/questions/app.md)
(a file changed outside the program: Accept or Read only) and Q-APP-26 (the courier clue: not stored).
This replaces the data rework's T3 and T4.

Tasks in order; each is one commit, under the code rules ([code-rules.md](../code-rules.md): one job per
module, size ceiling, no over-generalising, extract on the third copy, comments only for the not obvious,
proper types). Tests: known-answer fixtures through read → plan → save → verify as the backbone, tables for
fixed truths, broken outputs the verifier must catch, coverage not lower. Nothing the owner did not approve
(AGENTS.md): no new field, file or mechanism beyond the design. No build until J5.

## J1 — packing-store: the three files are the date

- One value for a date read from `day.toml`, `orders.csv` and `order-contents.toml`, with the checks of
  [save › When a date opens](../../software-design/data/save.md#when-a-date-opens): damage, hashes, inputs,
  outputs. The next `BatchID`, `PdfID`, `RuleID` and `OrderContentID` come from it.
- A commit: the files into `.partial/`, replaced files into `.partial/replaced/`, the rename of
  `.partial/day.toml` as the commit, then the moves. Settling `.partial/`: back when it holds `day.toml`,
  else forward.
- `LabelGroup` has `press_print_time: Option<Datetime>` and `is_printed: bool`; no `journal_seq`.
- Another copy's commit is found by the last-line hash of `day.toml`; a write reads the date again under the
  lock.
- A file changed outside the program: named, with Accept and Read only (Q-APP-25 A).
- Goes: `journal.rs`, `line.rs`, the fold as the date's source, `views/edited.rs`, the `restored` and
  `recovered` records. The fold stays only where J4's migration reads an old journal.
- Tests: save, amend, revert, mark and reprint give the files of the design; a stop at each step of a commit
  settles back or forward (table); a file edited by hand is named; two copies, one write each.

## J2 — packing-verify reads the files

- It reads the new files from `.partial/` and the date's current files into its own types; V1 (seals, `seq`)
  goes; V5 checks each group PDF's pages against the source pages `orders.csv` names, in its row order.
- Still no other crate of the workspace and no PDFium.
- Tests: the broken outputs it catches today (doubled, missing, swapped, foreign pages, a wrong count on the
  summary PDF) are still caught.

## J3 — packing-batch and the app

- Save, amend, revert, reprint, restore, Print and the printed tick commit through J1.
- The Day screen: *Sent hh:mm* from `press_print_time`; no *amended* anywhere; the file-changed note of J1.
- Search's index reads every date's `orders.csv` and `order-contents.toml`.
- Tests: save → reopen → the same Day screen; Search finds what it found before.

## J4 — Update from a date with a journal

- A date with `journal.jsonl` (v0.7, and the v0.8 test builds) is a migration
  ([D-APP-33](../../software-design/decisions/app.md)): its three files are written from the journal's fold,
  each group's `press_print_time` from the time of its last `print_sent`; the journal moves into the update
  folder ([D-APP-36](../../software-design/decisions/versions.md)). Areas and weights fill `orders.csv`.
- Tests: a v0.7 date and a test-build date give the files the design names; the update folder holds the
  journal; read-only mode writes nothing.

## J5 — Finish

- `docs/development/` (code-rules glossary without *journal*, testing, cli) and the user guide with its
  pictures; the next `alpha.N` version.
- `bash tools/check.sh --coverage`, an independent review of the whole diff, `bash tools/make-portable.sh`,
  and the handoff in the status of [README.md](README.md).
