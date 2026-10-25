# Development

How work is done in this repository: setup, build, run, test, and the plan of the unfinished work.
What to build is in [../software-design/](../software-design/README.md); what the business does today is in
[../business-process/](../business-process/README.md).

| File | Content |
|---|---|
| [setup.md](setup.md) | From a fresh clone to a working build: tools to install and the commands in order |
| [app.md](app.md) | Run the app in development, build the portable program folder, the gates |
| [cli.md](cli.md) | The command-line tools: `packing-cli` (batch steps for tests and sample runs, the read as JSON) and `packing-fixtures` (the made-up label PDFs) |
| [code-rules.md](code-rules.md) | The seven code rules, how they are applied, the glossary, the test rules, the checks and the process rules of a refactor. Binding for every task |
| [testing.md](testing.md) | The test levels, their fixtures and the commands that run them |
| [coverage.md](coverage.md) | Test coverage per language and where no test reaches (written by the refactor's first step) |
| [plan/](plan/README.md) | Only the unfinished work: status, next step, the reading list for a new session, one file per task |

## Start of every session

1. Read this file, then [plan/README.md](plan/README.md): the first unfinished task is the work. Open its
   file in `plan/`. With no task file left, the plan's Status block says what comes next (usually the
   owner's review).
2. Read the design files the task names (in [../software-design/](../software-design/README.md)). Read
   [../business-process/02-data-sources.md](../business-process/02-data-sources.md) before touching CSV or
   PDF code.
3. `git status` and `git log --oneline -5` show where the last session stopped.

## Rules

- **`samples/`, `work/` and `tmp/` are confidential** (real orders, buyer data). Never commit them, never
  copy real IDs, names or addresses into documents, code, tests or commit messages. They are in `.gitignore`.
- Committed test data (`testdata/`) is made up; its label PDFs and their answer are written by
  `cargo run -p packing-fixtures`. Tests that need `samples/` skip themselves when the folder is absent.
- Every change follows [code-rules.md](code-rules.md); its checks (`bash tools/check.sh`) are green before a commit.
- A one-time setup step (install, SDK, key, …) is written into [setup.md](setup.md) as part of the
  procedure, with the exact command or click path. `setup.md` is a procedure, not a log.
- Commit after each finished task (small commits; the message says what works). A finished task is deleted
  from `plan/` in the same commit; when a phase is finished its plan moves to `archive/`. Push to
  `origin main` (`https://github.com/wthe22/olshop-packing-ai`).
- At the end of each phase the owner reviews the running app. Findings go into `../reviews/<scope>.md` as
  `R-<SCOPE>-n`; the next session turns each into a task in `plan/` or, for a choice, into a question in
  [../software-design/questions/](../software-design/questions/README.md) with options and a default. The next
  phase starts after the review.
- A design change found while coding is written into `software-design/` in the same commit. An open choice for
  the owner goes into `software-design/questions/` with options and a default; it is never picked silently.
- Documents describe the current state only (no change history; git has that).
- When the next step is the owner's, end the session with a committed handoff in the status block of
  [plan/README.md](plan/README.md): what to review, where the owner writes answers, and what to do with
  each answer.

## Repository layout

```
Cargo.toml, Cargo.lock     Rust workspace
crates/engine/             rules, labels and slips → orders, plan, runs, day state (no PDF code)
crates/pdf/                PDFium: read text, copy pages, packing lists, the batch steps
crates/cli/                developer command line (tests, sample runs, the samples baseline)
crates/fixtures/           writes the made-up label PDFs of testdata/ and their answer (lopdf only)
crates/testkit/            fixtures the tests share (never built into the app)
app/                       Tauri app: Svelte screens (src/), Rust commands (src-tauri/)
tools/get-pdfium.sh        downloads the pinned pdfium.dll into vendor/ (git-ignored)
tools/make-portable.sh     builds the portable program folder target/portable/Packing/
tools/make-portable.bat    the same, for Explorer or cmd (runs the .sh through Git's bash)
tools/check.sh, *.mjs      the gates of code-rules.md
testdata/                  made-up fixtures for the tests
categories.toml            an example picks file: the grouping `default` (a data folder without one uses the one catch-all pick)
docs/                      see ../README.md
samples/  work/  tmp/      confidential real files and output, git-ignored
```
