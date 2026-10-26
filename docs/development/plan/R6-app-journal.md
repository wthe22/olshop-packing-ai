# R6: the app saves through the journal

Started 2026-10-26, after the owner's go-ahead (the v0.1 program checked: it works as before).
[code-rules.md](../code-rules.md) is binding for every step and every subagent; each step's report states
*size gate passed, no new lint suppression, coverage not lower*.

**Proves it:** save → reopen → the same Day screen; every save passes `packing-verify`.

Read first: [04 Save](../../software-design/workflow/04-save.md), [05 After the save](../../software-design/workflow/05-after-save.md),
[journal](../../software-design/data/journal.md), [records](../../software-design/data/records.md),
[summary PDF](../../software-design/labels/summary-pdf.md), [verification](../../software-design/app/verification.md),
[program](../../software-design/app/program.md), [namespaces](../../software-design/app/namespaces.md).

## Steps

| Step | What | Done when |
|---|---|---|
| R6.1 (done) | `packing-pdf`: the summary PDF writer — pick pages and facts page from the batch's record | V6 passes on a summary PDFium wrote; `verify_outputs.rs` uses it instead of its lopdf stand-in |
| R6.2 (done) | New crate `packing-batch`: the batch steps over engine, pdf, store and verify — save (steps 11–15), revert (17), Printed (16), the amend's read from `input/` (18) | Known-answer test through read → plan → save → verify → reopen; a finding stops the save and *Save anyway* records it; revert and amend keep the numbers |
| R6.3 (done) | The app on v0.2: `packing-settings.toml` and `packing-workflows.toml` beside the program, the owner (namespace) and date, `read_labels` through copies → repeats → plan, Save/Revert/Amend/Printed through `packing-batch`, the Day screen from the fold; the version 0.2.0 | Save → reopen → same Day screen, in the app's tests and in the portable build |
| R6.4 | The CLI's `prepare`/`revert`/`amend` (and `dump`) on `packing-batch`; then the v0.1 code removed: engine `day`, `verify`, `plan`, `names`, `couriers_edit`; `packing-pdf` `batch`, `verify` and the v0.1 packing-list layouts (their fonts and measuring move under the summary writer). `groupings`, `rules_edit` and `search` go with R7 and R8 | Nothing uses them; all checks green |

## Notes for the steps

- The summary's counts are printed without thousands separators: V6 reads `<n> orders` with `\d+`.
- Until R6.4 the summary writer borrows the page drawing of `packing_list` (fonts, measuring, emit).
- The app no longer offers *Categories* and *Search*: both worked on v0.1 files. R7 brings Search back on
  the index, R8 the Sort rules screen on `packing-workflows.toml` (its Svelte screens stay for that);
  until then the sort rules are edited in `packing-workflows.toml` by hand. The Repeated orders screen
  (choosing a copy) is not built yet: every repeat keeps its first copy.
- `packing-pdf`'s read still decides which logos need a shape hash with the bundled v0.1 courier table;
  R6.4 gives it the rules' known logos.
- The CLI's `prepare`, `revert` and `amend` still run v0.1; they move onto `packing-batch` with R6.4.
