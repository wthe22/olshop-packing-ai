# Workflow audit: where the built program differs from the workflow

The workflow in [software-design/workflow/](../../software-design/workflow/README.md) is the target;
this is what reading the code (2026-10-21) found different from it. Nothing here is fixed yet. The
owner reads the workflow first ([R-APP-2](../../reviews/app.md)); a step the owner finds wrong
changes the workflow, and this file follows. Tasks are written when the owner's answers to
[the feature questions](../../software-design/questions/README.md) are in.

The sample files hold no repeated order (11 files, 1,289 pages: 1,284 distinct Order IDs; the 5
that have a second page are label + continuation, each in one file, the second page right after the
first), so a test file with a repeat is made in `testdata/`.

| # | Step | The code does | Effect | Needs |
|---|---|---|---|---|
| **W-1** | 4, 7 (I2) | `find_or_create_group` in `crates/engine/src/orders/read.rs` joins every page with the same Order ID into one order, in one file or across files | A repeat doubles the order's item lines, its quantities and its pages; nothing counts it | Copies, the repeat step, [duplicates](../../software-design/app/duplicates.md) |
| **W-2** | 4 | A page with an Order ID is a new order or a page of an existing one; there is no test for a label part | A second copy right after the first cannot be told from a continuation | The label-part rule ([Q-LABEL-9](../../software-design/questions/duplicates.md)) |
| **W-3** | 11 (I4) | `expected_pdfs` in `crates/engine/src/verify.rs` builds the expected side from the plan's own orders, and the identity of a page is its Order ID | A doubled page is expected twice and found twice: the check passes; a swapped page of the same order is not seen | The page ledger and page fingerprint ([saved-pdf-check](../../software-design/app/saved-pdf-check.md)) |
| **W-4** | 6, 11 | An order left out as already saved is a warning text only (`drop_saved`, `state.json` `warnings`) | The ledger cannot add up; the Day screen cannot list what was left out | `left_out` in `state.json` ([data-folder](../../software-design/app/data-folder.md)) |
| **W-5** | 2 | `download/` copies are named by the file's own name (`crates/pdf/src/batch.rs`); no check of two chosen files with one name was found in `check_files` or the read command | Two files with one name from two folders: the second copy overwrites the first; an amend then reads one file twice and loses the other | Refuse a name used twice; test it first |
| **W-6** | 2 | *Used in batch n* compares only with earlier batches | The same bytes under two names in one batch are read twice | *Same file as n* in the file list |
| **W-7** | 12, 13 | A save over an existing batch folder (an amend) deletes the old folder with `remove_dir_all` **before** the rename (`crates/engine/src/day/save.rs`) | A saved PDF open in a viewer makes the delete stop half way: the old batch is partly gone, `state.json` still lists it, the new batch is not in place | Rename the old folder aside, rename the new one in, delete the old one last; restore on failure |
| **W-8** | 5 | `Qty Total` is compared with the sum of lines over all pages of an order | With a repeat the sum is doubled: a false *Slip total differs* | Check each copy (after W-1) |

## What the build needs, by crate

- **Sessions**: `days_in` and `AppState.day` already treat a day as any folder name under `labels/`;
  the date parse for ◀ ▶ and *today* must read the first ten characters; add the list/new/rename
  commands, the panel, `settings.json` panel state. No migration (option 1 of Q-APP-4).
- **Repeats**: `engine` — copies, fingerprint compare, groups, the choice, `left_out`; `pdf` — page
  fingerprint at read; `app` — the Repeated orders screen and the file-list marks.
- **Page ledger**: `pdf::verify` read-back fingerprints; `engine::verify` ledger over input pages and
  `left_out`; the new finding kinds and texts.
- **Tests first**: a `testdata/` file with a repeat in one file, across files, adjacent (`A A`), a
  *differs* repeat, a continuation page; a saved set with a doubled page and a missing page for the
  check.
