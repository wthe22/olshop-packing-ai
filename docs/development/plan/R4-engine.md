# R4 The engine of v0.2

Row R4 of [v0.2-rework.md](v0.2-rework.md). [code-rules.md](../code-rules.md) applies to every line; each
step's report states *size gate passed, no new lint suppression, coverage not lower*. The design:
[02 Read](../../software-design/workflow/02-read.md), [03 Plan](../../software-design/workflow/03-plan.md),
[reading](../../software-design/labels/reading.md), [courier](../../software-design/labels/courier.md),
[duplicates](../../software-design/app/duplicates.md), [groups](../../software-design/labels/groups.md),
[rules file](../../software-design/rules-file/README.md), [records](../../software-design/data/records.md).

## How

The v0.2 pipeline is built beside the v0.1 one in `packing-engine`, which the app keeps using until R6
moves the app onto the journal and deletes `day`, `verify` and what only they use; R8 moves the screens.
Shared code (the slip reader, the label fields, the filter grammar and evaluator) is changed in place,
with the v0.1 tests kept green.

## Steps, one commit each

1. **`workflows`**: `packing-workflows.toml` — types, load, every check of the design with the file's line
   and column, the bundled copy (the six couriers, one rule `default`). The filter fields of v0.2 (the
   design's table: `courier` compares the code, `total_quantity` is the printed total, `ship_by` a date).
2. **Copies** (step 5): pages → copies by the label-part rule and its stops; a copy's values with the
   courier code and its clue (logo, shape, text, form, tag), the ship-by date, the pickup code and the
   instant tag; the `Qty Total` check per copy; the page fingerprint.
3. **Repeats** (steps 7–8): orders with their copies; the already-saved guard from the fold's live
   orders; same or differs; the input groups; the four choices; one copy kept, the rest `left_out`.
4. **The plan** (steps 9–10): the sort rule's groups by first match, the rest `?`, order contents (items
   without Seller SKU; SKUs and the printed total on the order), printing order, file names.
5. **The record**: a `batch_saved` from the plan (inputs, the rule's snapshot, contents, orders, left out,
   warnings), the outputs given by the caller; a fixture with repeats (`labels-repeat-*.pdf`) and its
   answer, read → plan → record as the known-answer test.
6. **2.32**: a word a font change splits is read whole ([2.32](2.32-text-object-split-word.md)); the
   samples' baseline files it changes are written again and the difference named.

## Done when

`bash tools/check.sh --coverage` is green after each step; `bash tools/make-portable.sh` builds at the end.
