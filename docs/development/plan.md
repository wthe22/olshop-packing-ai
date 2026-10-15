# Build plan

Phases in order, for both tools. Each phase has a folder with its task list (`plan.md`) and its
owner reviews (`review-N.md`). Each task ends with a commit. Tick the box (`[x]`) in the phase's
`plan.md` in the same commit that finishes the task. A phase is done when all its tasks are
ticked **and** its owner review is noted as passed in the phase's review file.

| Phase | Tool | Tasks | Owner review |
|---|---|---|---|
| 1 — PC script (done; now the analysis tools in `tools/analysis`) | desktop | [p01-pc-script](../desktop/phases/p01-pc-script/plan.md) | none |
| 2 — PC app (Rust + Tauri) | desktop | [p02-pc-app](../desktop/phases/p02-pc-app/plan.md) | [round 1](../desktop/phases/p02-pc-app/review-1.md), [round 2](../desktop/phases/p02-pc-app/review-2.md) |
| 3 — Android camera spike | mobile | [p03-camera-spike](../mobile/phases/p03-camera-spike/plan.md) | [review-1](../mobile/phases/p03-camera-spike/review-1.md) |
| 4 — App core (no camera) | mobile | [p04-app-core](../mobile/phases/p04-app-core/plan.md) | [review-1](../mobile/phases/p04-app-core/review-1.md) |
| 5 — Scanning | mobile | [p05-scanning](../mobile/phases/p05-scanning/plan.md) | [review-1](../mobile/phases/p05-scanning/review-1.md) |
| 6 — Files and settings | mobile | [p06-files-settings](../mobile/phases/p06-files-settings/plan.md) | [review-1](../mobile/phases/p06-files-settings/review-1.md) |
| 7 — Layouts and polish | mobile | [p07-layouts-polish](../mobile/phases/p07-layouts-polish/plan.md) | [review-1](../mobile/phases/p07-layouts-polish/review-1.md) |

## Status

Status: **Phase 2 (PC app): tasks 2.1–2.15 and 2.17 done. Open: 2.16, 2.18, 2.19, 2.20 and 2.21
(search, added 2026-10-15).** The owner's round-2 review at the end of the phase
([review-2](../desktop/phases/p02-pc-app/review-2.md)) waits for all of them; the phase-3 Android work
waits for that review.

**Owner answers of 2026-10-15** (written into the design; the questions are removed from 02):

- **D17: the check of the saved PDFs runs on every batch** (task 2.18, no size condition).
- **D20: taught logos are hardcoded in the app.** The app writes no file for them, and no settings file
  is needed for now. `couriers.toml` stays the courier list (D16, D19). **To confirm with the owner:** a
  logo named in the prompt is recognised in later batches only once a program version carrying its hash
  is built ([07 › `couriers.toml`](../desktop/design/07-pc-app.md#courierstoml)).
- **New task 2.21, search:** a Search button finds saved orders by tracking ID, item, quantity and the
  other rule fields ([08 › 7. Search](../desktop/design/08-pc-app-ui.md#7-search)). Saved orders now keep
  the searched fields in `state.json` ([07 › `state.json`](../desktop/design/07-pc-app.md#statejson)).
  Its scope is open question **D25**.

Open choices, each with a *Built for now* default in
[02-design-questions.md](../desktop/design/02-design-questions.md): D18, D21, D22, D23, D24, D25.

**Next step.** 2.16 (the courier: the form-and-tag rule; no owner answer needed), then 2.19 (courier data
in the data folder), 2.18 (the saved-PDF check), 2.20 (one hash, XXH3-64), 2.21 (search). Each task ends
with a commit and its tick in [p02 plan.md](../desktop/phases/p02-pc-app/plan.md); push to `origin main`.

**Reading for the next session** (in this order; each phase task also names its own design sections):

1. [AGENTS.md](../../AGENTS.md) and [README.md](README.md): rules, commit and push, repository layout.
2. [p02 plan.md](../desktop/phases/p02-pc-app/plan.md): tasks 2.16 and 2.18–2.21; [review-2.md](../desktop/phases/p02-pc-app/review-2.md) (what the owner will check).
3. [07-pc-app.md](../desktop/design/07-pc-app.md): Inputs and `couriers.toml`; *What happens in one batch*; *Day folder* and `state.json`; *Architecture* (stack, code layout, commands, settings, program folder); *Testing*.
4. [08-pc-app-ui.md](../desktop/design/08-pc-app-ui.md): window frame; screens 1–6; *7. Search*; *Start*; the logo prompt.
5. [03-labels-and-picks.md](../desktop/design/03-labels-and-picks.md): per-order data, Order ID, multi-page orders, packing slip, *Courier deduction*, picks, packing list.
6. [02-design-questions.md](../desktop/design/02-design-questions.md): the open questions D18 and D21–D25.
7. [01-requirements.md](../desktop/design/01-requirements.md): concepts and words; [desktop/design/README.md](../desktop/design/README.md) for the folder.
8. [shared/04-rules-file.md](../shared/04-rules-file.md): the condition language that search reuses (fields, operators, the item-field "some line" rule).
9. [business-process/02-data-sources.md](../business-process/02-data-sources.md): read before touching any CSV or PDF code; [01-packing-workflow.md](../business-process/01-packing-workflow.md) for the daily steps.
10. [desktop/development/01-setup.md](../desktop/development/01-setup.md): PDFium, Tauri, the build, the portable folder, test commands; and [desktop/development/README.md](../desktop/development/README.md).
11. [desktop/user-guide/01-pc-app.md](../desktop/user-guide/01-pc-app.md): the owner's guide; update it in the commit that changes what the owner sees (2.18, 2.21).
12. Code, after the docs: `pc/` (Cargo workspace: `crates/engine`, `crates/pdf`, `crates/cli`; `app/` with Svelte in `src/` and Rust in `src-tauri/`); `testdata/` (made-up fixtures, `make_testdata.py`); `tools/analysis/packing/couriers.toml` (the built-in courier list, 07 › *Inputs*); then `git status` and `git log --oneline -5`.

`samples/` and `work/` are confidential: never commit them or copy real data from them.
