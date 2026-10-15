# ID map

Old IDs map to the scoped IDs of the live documents. Code comments and old commits may keep the old IDs.

## Requirements

| Old | New | | Old | New |
|---|---|---|---|---|
| P1 | F-LABEL-1 | | U1 | F-UI-1 |
| P2 | F-LABEL-2 | | U2 | F-APP-2 |
| P3 | F-LABEL-3 | | U3 | F-UI-2 |
| P4 | F-LABEL-4 | | U4 | F-APP-3 |
| P5 | F-LABEL-5 | | U5 | F-APP-4 |
| P6 | F-APP-1 | | U6 | F-UI-3 |
| | | | U7 | F-UI-4 |
| | | | U8 | F-APP-5 |

## Questions and decisions

| Old ID | Old meaning | New ID | Where |
|---|---|---|---|
| D2 | PDF library: PDFium for all three PDF jobs | D-APP-1 | `software-design/decisions/app.md` |
| D3–D12 | Owner answers of the first design round (script kept as fallback, portable program folder with data beside it, `batch N` folders, editable PDF names, one packing list per batch, download copy for amend, open in the PDF viewer, English only, Add files only, boxes categories editor) | none: folded into the design files | `software-design/app/`, `software-design/ui/` |
| D4 | The Python tools' rule for a CSV that holds none of the label orders (warned once and ignored) | none: analysis tools only | `tools/analysis/packing/picks.py` |
| D13 | Unticking the rest row | Q-UI-1 | `software-design/questions/ui.md` |
| D14 | A log file | Q-UI-2 | `software-design/questions/ui.md` |
| D15 | Courier named from two clues (logo and tracking-ID form) | D-LABEL-2 | `software-design/decisions/label.md` |
| D16 | The courier data in the data folder | D-LABEL-1 | `software-design/decisions/label.md` |
| D17 | The check of the saved PDFs runs on every batch | D-APP-2 | `software-design/decisions/app.md` |
| D18 | Reading a courier logo the encoded bytes cannot name | Q-LABEL-1 | `software-design/questions/label.md` |
| D19 | One courier file, not two | D-LABEL-1 | `software-design/decisions/label.md` |
| D20 | Taught logos bundled in the program, no file | D-LABEL-3, replaced by D-LABEL-4 (the app remembers them in the data folder) | `software-design/decisions/label.md` |
| D21 | An instant-service label names no courier | Q-LABEL-2 | `software-design/questions/label.md` |
| D22 | Where the pickup code shows during packing | Q-LABEL-3 | `software-design/questions/label.md` |
| D23 | The Shopee-style label: pack it, or leave it alone | Q-LABEL-4 | `software-design/questions/label.md` |
| D24 | Which image on a page is the courier's logo | Q-LABEL-5 | `software-design/questions/label.md` |
| D25 | Search: which days, and what an order saved before search can show | Q-UI-3 | `software-design/questions/ui.md` |
| (status note) | A taught logo is recognised only once a program version carries its fingerprint | Q-LABEL-6, answered: D-LABEL-4; the file question is Q-LABEL-7 | `software-design/decisions/label.md` |

## Files and sections

| Old | New |
|---|---|
| `desktop/design/01-requirements.md` | `software-design/requirements.md` |
| `desktop/design/02-design-questions.md` | `software-design/questions/` |
| `desktop/design/03-labels-and-picks.md` | `software-design/labels/`: Per-order data, Order ID, Multi-page orders, Reading the packing slip → `reading.md`; Courier deduction → `courier.md`; Picks, A saved PDF, Numbering and file names → `picks.md`; Packing list (A4) → `packing-list.md` |
| `shared/04-rules-file.md` | `software-design/rules-file/` (`README.md`: file format; `conditions.md`: condition language) |
| `desktop/design/07-pc-app.md` | `software-design/app/`: Inputs, `couriers.toml`, Fields the conditions can use → `inputs.md`; What happens in one batch, Revert and amend, Packing list → `batch.md`; Day folder, `state.json` → `data-folder.md`; Architecture, Stack, Code layout, Text runs → `architecture.md`; Commands, Settings, Program folder → `program.md`; Testing → `development/testing.md` |
| `desktop/design/08-pc-app-ui.md` | `software-design/ui/`: 1. Day → `day.md`; 2. New batch, Reading, The logo prompt → `new-batch.md`; 3. Plan → `plan.md`; 4. Messages → `messages.md`; 5. Categories → `categories.md`; 6. Settings → `settings.md`; 7. Search → `search.md`; Start → `start.md` |
| `shared/README.md` (conventions, how the tools relate) | `software-design/conventions.md`, `software-design/README.md` |
| `desktop/development/01-setup.md` | `development/setup.md` (log in `archive/setup-log.md`) |
| `desktop/development/README.md` | `development/app.md`, `development/cli.md` |
| `desktop/user-guide/01-pc-app.md` | `user-guide/01-setup.md` … `04-messages-and-settings.md` |
| `desktop/phases/p02-pc-app/plan.md` | open tasks in `development/plan/`; finished tasks in `archive/phase-2-finished-tasks.md` |
| `desktop/phases/p02-pc-app/review-1.md` | `archive/review-round-1.md` |
| `desktop/phases/p02-pc-app/review-2.md` | the "Owner review" list in `development/plan/README.md` |
| `desktop/phases/p01-pc-script/plan.md` | `archive/phase-1-plan.md` |
| `development/plan.md` | `development/plan/README.md` (old copy: `archive/handoff-2026-10-15.md`) |
| `development/README.md` | `development/README.md` |
| `mobile/` | the Android repository (managed there) |

Code comments in `tools/analysis/` cite the first-generation design files (`03-pc-script`); their rules now
live in `software-design/labels/` and `software-design/rules-file/`.
