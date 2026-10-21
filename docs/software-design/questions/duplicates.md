# Open questions: repeated orders and the page check

Questions: Q-LABEL-9, Q-LABEL-10, Q-APP-6, Q-APP-7, Q-UI-8. The design is
[duplicates](../app/duplicates.md), [Repeated orders](../ui/duplicates.md) and the page ledger in
[saved-pdf-check.md](../app/saved-pdf-check.md); each default is what they describe.

## Q-LABEL-9 — How a repeat is told from a continuation page

**Words.** An order whose slip does not fit on one page continues on a second page with the same
Order ID. A repeated order is the same label twice. Both give a second page with an Order ID the
reader has already seen.

**Example.** Pages 12, 13 are one order (label + continuation). Pages 12, 40 are the same order twice.
Pages 12, 13 where 13 is a second copy of the label would look like the first case by position alone.

| Option | What the owner sees |
|---|---|
| **1. By the page's content** (**Default**) | A page with an Order ID and a tracking ID or a header image starts a copy; one with neither continues. Both examples above are read right, also `A A` side by side. Measured on the sample files (1,289 pages): 5 orders have a second page, each right after its first in the same file, none a repeat; the business-process notes record that a continuation page has no tracking ID and no courier logo |
| 2. By position | A page with the same Order ID as the page right before it, in the same file, is a continuation; any other second appearance is a repeat. A copy placed right after its original is read as a long label |

**What the answer changes:** the copy rule in `crates/engine/src/orders/read.rs` and
[duplicates.md](../app/duplicates.md); the samples hold no repeat, so a test file is made in
`testdata/`.

## Q-LABEL-10 — What identifies a page

**Words.** The page-for-page check ([saved-pdf-check.md](../app/saved-pdf-check.md)) needs to know
that a saved page *is* a given input page, and a repeat's copies need comparing. A page copied into a
saved PDF is unchanged.

| Option | What the owner sees |
|---|---|
| **1. A hash of the page's text, positions and header images** (**Default**) | Fast (the text is read already); catches a wrong, missing or doubled page. Two pages with identical text are the same page, which is what a repeat is |
| 2. A hash of the rendered page | Also catches a barcode or image changed in the copy; a render of every page, read and written (slower; not measured) |
| 3. The Order ID and the page's place in its order | What the check compares today; a doubled page is invisible |

**What the answer changes:** the page fingerprint ([reading.md](../labels/reading.md#page-identity)) and
the read-back in `crates/pdf/src/verify.rs`.

## Q-APP-6 — What the owner must do about repeats

**Example.** Batch 3 holds 14 repeated orders in two groups.

| Option | What the owner sees |
|---|---|
| **1. The screen opens with every group on *Keep the first copies*; Continue works at once** (**Default**) | One press accepts the defaults; the Plan header says "14 copies left out ▸" |
| 2. Every group must be chosen | **Continue** waits until each card has a choice picked; no default is marked |
| 3. No screen, the first copies are kept | The batch goes to the Plan; a warning lists the copies left out, as the already-saved guard does today |

**What the answer changes:** the step between reading and the Plan in `crates/pdf/src/batch/read.rs`
and the screen ([Repeated orders](../ui/duplicates.md)).

## Q-APP-7 — May the owner keep both copies

**Words.** A repeat is normally one order that must be packed once. A damaged label reprinted on
purpose is the exception.

| Option | What the owner sees |
|---|---|
| **1. One copy stays** (**Default**) | Each repeated order has exactly one kept copy; a second label is printed from the saved PDF if needed |
| 2. **Keep both** per order | A third radio per order; the order's pages and item lines would be doubled in the plan, the packing list counts its units twice, and the page ledger expects the page twice |

**What the answer changes:** the order's pages in the plan, the ledger's expected count and
[duplicates.md](../app/duplicates.md).

## Q-UI-8 — Which PDFs are checked for repeats

**Words.** The request says *which PDFs contain duplicates*. The label files of a batch are where a
repeat comes from; a saved PDF written by an older version of the app could hold a doubled order.

| Option | What the owner sees |
|---|---|
| **1. The label files of the batch being prepared** (**Default**) | Marks on the file list and the **Repeated orders** screen; a saved batch cannot hold a repeat |
| 2. Also a **Check saved PDFs** button on a Day card | Reads the batch's saved PDFs back and lists the pages that appear twice; reads only, changes nothing |

**What the answer changes:** a button on the Day card ([Day](../ui/day.md)) and a read-only command
over `check_saved`.
