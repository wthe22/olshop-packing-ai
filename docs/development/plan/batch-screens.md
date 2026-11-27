# Batch screens: reading in place, the sort rule after reading, the summary in the app

The owner's findings of 2026-11-27, written into the design: [D-UI-17 to D-UI-19](../../software-design/decisions/ui-screens.md)
and [D-RULES-7](../../software-design/decisions/rules.md). Archive mode itself is open
([Q-APP-27](../../software-design/questions/app.md)) and is not built here. Starts after
[journal-removal.md](journal-removal.md): U3 reads the date's files that J1 makes the truth.

Tasks in order; each is one commit, under the code rules ([code-rules.md](../code-rules.md): one job per
module, size ceiling, no over-generalising, extract on the third copy, comments only for the not obvious,
proper types). Tests: known-answer fixtures through read → plan → save → verify as the backbone, tables for
fixed truths, coverage not lower; the screens are screenshot-checked at 125 % against Day, Plan and Settings.
Nothing the owner did not approve (AGENTS.md).

## U1 — Boxes and words

- No example in any box (D-UI-17): `namePlaceholder` is *Account name*; the boxes editor's date box shows
  `yyyy-mm-dd`; look through every `placeholder` of `app/src`.
- *+ Add a group for this batch* is **Add Group**.

## U2 — Reading stays on New batch

- [New batch › Reading](../../software-design/ui/new-batch.md#reading): the progress bar under the file list,
  the list and drop area in place; a stop under its file with the row marked red, the files kept; **Read
  labels** again after a change. A stop names its file and pages in the back end's own type, not in a text.
- The Plan screen's **Warnings ▸** under the name of each file.
- Tests: a file with no packing slip stops with the message under that file; taking it off and reading again
  opens the Plan screen.

## U3 — The sort rule on the Plan screen, and *Custom*

- [Plan › The sort rule](../../software-design/ui/plan.md#the-sort-rule): the list in the Plan header, the rule
  used last preselected, another rule builds the plan again; New batch has no rule list.
- A tick, a move or **Add Group** makes the plan *Custom (from default)*; saved with its groups under the
  name *Custom*; *Also add to the sort rule* goes; the Sort rules screen refuses the name *Custom*.
- Amend: the rule of the batch's name, or its saved groups when *Custom* or gone.
- Tests: a table of preselection (last rule, off, gone, *Custom*); a *Custom* batch saves its groups and leaves
  `config.toml` unchanged; its amend plans as saved.

## U4 — The Summary screen

- [Summary](../../software-design/ui/summary.md): a batch's or the date's summary from the three files, the
  account list, **PDF**, **Back**; **Summary** and **Day summary** on the Day screen open it.
- A group row's **▸** on the Day screen lists its order contents.
- Tests: the screen's counts and rows equal the summary PDF's for the known-answer fixtures; it opens in
  read-only mode and on a date whose PDFs are deleted.

## U5 — Finish

- The user guide and its pictures; `bash tools/check.sh --coverage`; `bash tools/make-portable.sh`; the
  handoff in the status of [README.md](README.md).
