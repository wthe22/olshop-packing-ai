# Rules for coding agents

- Start with `docs/development/README.md`, then `docs/development/plan/README.md`: the first unfinished
  task there (its file is in `docs/development/plan/`) is the work. Read the design files the task names
  (`docs/software-design/`).
- `samples/`, `work/` and `tmp/` are **confidential** real data. Never commit them; never copy real
  IDs, names, phone numbers or addresses into code, tests, docs or commit messages. Use made-up
  values (`JY0000001234`, `580000000000000001`). Committed fixtures live in `testdata/`; the label
  PDFs and their answer are written by `cargo run -p packing-fixtures`.
- Code rules: `docs/development/code-rules.md` is binding for every line you write and every subagent
  you start (seven rules in short: one job per module; files under the size ceiling; no over-generalising
  or over-engineering; extract on the third copy; comment only what is not obvious, no IDs or history in
  comments; proper types, no overloaded values). Each rule states its reason; an exception carries a one-line
  reason in its commit message. Tests: known-answer fixtures through read → plan → save → verify as the
  backbone, tables for fixed truths, broken outputs the verifier must catch, coverage as the measure. Name it
  in every task file and every subagent instruction.
- What checks an output is never the code that made it: `packing-verify` uses no other crate of this
  workspace and no PDFium (`docs/software-design/app/verification.md`).
- Checks: a quick check while working (`cargo test -p <crate>`); before every commit the full check,
  `bash tools/check.sh --coverage` (format, clippy, all tests, size and lint ratchets, front-end check and
  tests, coverage not lower). A task report states: size gate passed, no new lint suppression, coverage not
  lower. Never weaken a check or add to a baseline.
- The project is in heavy development: build for what is decided now; no compatibility layer or option for
  a change that might come. A change to the shape of stored user data is a migration.
- Another session may be editing: run `git status` and `git log --oneline -5` before editing a file, and
  never touch a file that session owns.
- This repository is the PC app.
- Docs (`docs/README.md` has the folder table, the scope list and the writing rules): `business-process/` =
  facts true without the software; `software-design/` = the target system only (no plans, no phases;
  `workflow/` is every step with its invariants, `data/` the journal and files, `decisions/` and
  `questions/` inside); `development/` = setup, commands, tests and `plan/` (unfinished
  work only; phase and task numbers live only there); `reviews/` = the owner's feedback on the running
  app, one file per scope; `user-guide/` = how the owner uses the app; `archive/` = superseded records
  and `id-map.md` (never linked from live docs). Current state only, no change history. A design change
  found while coding is written into `software-design/` in the same commit.
- IDs: `D-<SCOPE>-n` decision, `Q-<SCOPE>-n` open question, `R-<SCOPE>-n` review item; scopes LABEL,
  RULES, APP, UI. Numbers are never reused. Code comments may keep old IDs; `docs/archive/id-map.md`
  covers them.
- Naming: timestamps `*_time` (`create_time`, `modify_time`). Orders are identified by `Order ID`; items
  by product name and variation (the platform `SKU ID` is not on the label PDFs). The words of code are the
  glossary of `code-rules.md` (workspace, sort rule, label group, group PDF, summary PDF, journal).
- Data: the journal (`{workspace}/{date}/journal.jsonl`) is the only truth; every other data file is an
  input, an output or a view of it, and nothing but the journal decides (`docs/software-design/data/`).
- Writing: the owner is "the owner", never he/she/they/it/you. The dropship supplier's real name never
  appears: write "the supplier". About 150 lines per file at most; each fact has one home.
- Setup is one procedure in `docs/development/setup.md`; keep it current when a one-time step changes.
- After any development work on the app, always run `bash tools/make-portable.sh` so the owner can test
  the program folder `target/portable/Packing/` immediately. Report it in the handoff; if the build
  fails, say so and do not call the work ready.
- Commit after each finished task and delete the task from `docs/development/plan/`; push to
  `origin main`. The owner reviews at the end of each phase; findings go into `docs/reviews/<scope>.md`.
- The owner is often away and may answer a review in a new session. Whenever the next step is the
  owner's, end with a committed handoff in the status block of `docs/development/plan/README.md`: what
  to review, where to write answers, and what to do with each answer. A new session starts there.
- Shell is Git Bash on Windows. The project has no Python: every tool is Rust or Node.
- The owner decides open choices: put them in `docs/software-design/questions/` with options and a
  default, do not pick silently.
