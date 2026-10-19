# R5 — Functions that do several jobs or nest too deep

Read first: [../code-rules.md](../code-rules.md) (binding), [refactor.md](refactor.md) (brief, choices).
Effort M. Split by job, not by length: a function stays whole when it does one job, however long. Behaviour
does not change; tests are not edited. Judge each lead by reading it; record in the report which leads
were split and which were left whole with the reason (one line each).

Leads from the survey (paths and lengths as before R1; find them by name):

| Function | Lines | Why a lead |
|---|---|---|
| `pdf::batch::save` | 183 | write, check, record, clean up in sequence |
| `engine::orders::read_batch` | 178 | nests 5 deep, reads, groups, deduces, warns |
| `engine::rules::parse_comparison` | 128 | one long match; split only if its arms are separate jobs |
| `engine::verify::compare` | 126 | nests 5 deep |
| `engine::rules::tokenize` | 123 | nests 6 deep |
| `engine::rules::load_rules_named` | 122 | parse, validate, build |
| `commands::batch::plan_view` | 110 | rows, numbers, runs, flags |
| `pdf::worker::serve` | 103 | nests 7 deep: the command loop and each command's work |
| `commands::rules::rules_check_impl` | 89 | nests 5 deep |
| `pdf::read::build_runs_geometry`, `batch::summary_text`, `packing_list::emit`, `engine::verify::fmt`, `commands::batch::save_batch_impl`, `commands::rules::rules_save_impl` | 71–86 | read each |

Also: every function deeper than four levels in `tools/check-lint.mjs`'s cognitive-complexity list (threshold
25); lower the baseline (`--update`) for each one fixed. A split keeps the new functions private to their
module unless another module needs them. No new `allow`; no parameter list over five (group them, R2's types).

Done when: `bash tools/check.sh --coverage` is green, no function nests deeper than four levels, and the
complexity-warning count in the lint baseline is lower than before the step (zero where the function was
split; a function left whole stays in the baseline with its reason in the report).
