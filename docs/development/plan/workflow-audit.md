# Workflow audit: where the built program differs from the design

The workflow in [software-design/workflow/](../../software-design/workflow/README.md) is the target; this is
what reading the code (2026-10-21) found different from it. Nothing here is fixed yet; R3 to R6 of the
[v0.2 rework](v0.2-rework.md) close each row, and each fix starts with a test that fails on today's code.

The sample files hold no repeated order (11 files, 1,289 pages: 1,284 distinct Order IDs; the 5 that have a
second page are label + continuation, each in one file, the second page right after the first), so the
fixture writer makes the repeats.

| # | Step | The code does | Effect | Closed by |
|---|---|---|---|---|
| **W-1** | 5, 8 (I2) | `find_or_create_group` in `crates/engine/src/orders/read.rs` joins every page with the same Order ID into one order, in one file or across files | A repeat doubles the order's item lines, its quantities and its pages; nothing counts it | Copies and the repeat step ([duplicates](../../software-design/app/duplicates.md)) — R4 |
| **W-2** | 5 | A page with an Order ID is a new order or a page of an existing one; there is no test for a label part | A second copy right after the first cannot be told from a continuation | The label-part rule ([D-LABEL-12](../../software-design/decisions/label.md)) — R4 |
| **W-3** | 13 (I4) | `expected_pdfs` in `crates/engine/src/verify.rs` builds the expected side from the plan's own orders, and the identity of a page is its Order ID | A doubled page is expected twice and found twice: the check passes | The independent verifier ([verification](../../software-design/app/verification.md)) — R5 |
| **W-4** | 7, 13 | An order left out as already saved is a warning text only (`drop_saved`, `state.json` `warnings`) | The ledger cannot add up; the Day screen cannot list what was left out | `left_out` in the record ([records](../../software-design/data/records.md)) — R4, R6 |
| **W-5** | 3 | `download/` copies are named by the file's own name (`crates/pdf/src/batch.rs`) | Two files with one name from two folders: the second copy overwrites the first | Inputs by content ([D-APP-10](../../software-design/decisions/app.md)) — R3 |
| **W-6** | 3 | *Used in batch n* compares only with earlier batches | The same bytes under two names in one batch are read twice | One file chosen twice is listed once — R3, R8 |
| **W-7** | 14, 18 | A save over an existing batch folder (an amend) deletes the old folder with `remove_dir_all` **before** the rename (`crates/engine/src/day/save.rs`) | A saved PDF open in a viewer makes the delete stop half way: the old batch is partly gone | Old outputs moved aside before the commit ([04 Save](../../software-design/workflow/04-save.md#14-commit)) — R3, R6 |
| **W-8** | 6 | `Qty Total` is compared with the sum of lines over all pages of an order | With a repeat the sum is doubled: a false *Slip total differs* | The check per kept copy — R4 |
