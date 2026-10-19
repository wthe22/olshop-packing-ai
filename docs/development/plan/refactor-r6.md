# R6 — Comments

Read first: [../code-rules.md](../code-rules.md) (binding), [refactor.md](refactor.md) (brief, choices).
Effort M. Comment text only: no code line changes. Done in three parts, one after the other, each ending on a
green `bash tools/check.sh --coverage`: (a) `crates/engine`, (b) `crates/pdf` and `crates/cli`, (c)
`app/src-tauri`, `app/src`, `app/tests`, `tools/*.sh`, `tools/*.mjs` and the Rust and TypeScript tests.

Remove or rewrite, found with `grep`:

- Decision, question, review and requirement IDs (`D-LABEL-4`, `Q-UI-3`, `R-RULES-1`, `F-LABEL-3`; 92 of
  them) and task numbers (`2.21`, `task 2.12`, `(2.26)`).
- Pointers to a design file by number (`07 › Inputs`, `08 › 3. Plan`, `04-rules-file`: 702 of them, none of
  the numbered files exists any more). Where a pointer helps, name the file under `docs/software-design/`
  (`labels/courier.md`); most comments need none.
- Banner comments (`// ----- section`, 64), and the `// ---` lines in `tools/*.sh`.
- Comments that repeat the code, narrate history or phases (*now*, *later*, *first implementation*, *ported
  from*, *reference implementation*), or describe what a test name already says.
- `///` in `.ts` and `.svelte` (241): TypeScript uses `//` or `/** */`.
- Keep the comments that explain what is not obvious: a PDFium or Windows quirk, a unit (mm, pt), an order
  that matters, why a number is what it is, the reason for a rule of the business that a reader could
  mistake for a bug.

Then add the zero gate: `tools/check-lint.mjs` fails on any comment line in `crates/`, `app/src`,
`app/src-tauri/src`, `app/tests` and `tools/*.sh` that matches an ID pattern
(`\b[DQRF]-(LABEL|RULES|APP|UI)-\d+`), a task number, a numbered design pointer (`\b0\d ›`) or a banner (`//
-{5,}`). No baseline for it: it is zero from here on. Update `docs/software-design/conventions.md`: it says
code comments and tests may cite requirement IDs; they no longer may. Keep `docs/archive/id-map.md` as is.

Done when: the gate passes with the new check, `git diff --stat` shows only comment lines changed (state the
check: `cargo test` count unchanged, no code line in the diff), and the report gives counts before and after
for each removed kind.
