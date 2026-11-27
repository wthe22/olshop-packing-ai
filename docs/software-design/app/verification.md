# Verification

Wrong code could copy a page twice, drop one, put an order in two group PDFs, write the pages in the wrong
order, or print a wrong count. **The verifier**, crate `packing-verify`, checks every output against the
`day.toml`, `orders.csv` and `order-contents.toml` about to be committed, before the commit ([D-APP-12](../decisions/app.md)). Which invariant
each check proves is the [verification matrix](../workflow/README.md#verification-matrix).

## Independent of what it checks

- Its only PDF library is `lopdf`; it does not use `packing-engine`, `packing-pdf`, `packing-store` or
  PDFium ([architecture](architecture.md#who-may-use-whom)).
- It reads the three files in `.partial/`, and the date's current files for the already-saved guard, into
  types of its own.
- Its expected side comes from the inputs and the three files, never from the files it checks.

## The page digest

A page's **digest** is XXH3-64 of the page as drawn, in a fixed form:

1. The page's effective `MediaBox`, `CropBox` (the `MediaBox` when absent) and `Rotate`, inherited from the
   page tree where the page has none.
2. Its content: every stream of `/Contents`, decoded, in order.
3. Its effective `Resources`, with every reference resolved and every stream decoded; dictionary keys in
   sorted order; `/Length`, `/Filter`, `/DecodeParms` and `/Parent` left out; a number written as its
   value to four decimals (PDFium writes a `1.0` as `1`); a reference already being written is written as
   a back-mark, so a loop ends. A stream `lopdf` cannot decode (an image's DCT) is taken as stored, with
   its filter.

A shared object (a font, an image) is digested once per file and the result reused. The form ignores how
a writer stores a page — compression, object numbers, the order of objects — and keeps what it draws: on
the sample downloads every page PDFium copies has the digest of its source page, and every page of a file
has a digest of its own.

## What it checks in a batch

| # | Check | Finding when it fails |
|---|---|---|
| V2 | Every input file in the date folder has the `hash_sha256` and `page_count` of `day.toml`; the verifier finds it by its name, ` Input ` in it and a space and the first 12 characters of the SHA-256 at its end, and checks the bytes | Error: an input changed |
| V3 | **The ledger**: every page of every input lies in exactly one written copy or one left-out copy; *pages read = written + left out* | Error: a page in no copy (1:0), a page in two (1:N) |
| V4 | Order IDs are unique in `orders.csv` and in no other live batch of the date; each `repeat` names a kept copy that is written; each `saved_earlier` names a live batch that holds the order | Error |
| V5 | Each group PDF has the page count its orders give, and its page *k* has the digest of the source page `orders.csv` puts at *k* | Error: missing (1:0), doubled (1:N), a page of no input, an order split over two files. Warning: every page there once, in another order |
| V6 | The summary PDF's printed counts (orders, units, pages read, written and left out, each group bar's orders and units), read as text, equal the files'; on an account's summary, those of its orders | Error |

The checks of a **reprint** and a **restore** are V2, V5 and V6 for the files they write
([reprint](reprint.md)); an **account's summary** is checked by V6 against its batch's files, the orders
whose shop the account lists in `config.toml` (*No account*: those of no listed shop) only ([summary PDF](../labels/summary-pdf.md#one-account)). A finding stops it and nothing is kept.

V4 also checks the accounts: no shop name (spaces and case ignored) stands under two accounts in
`config.toml`.

## Two severities, never one list

An **error** is a check that failed. A **warning** is V5's other order alone: every page is there exactly
once. Either stops the save ([D-APP-18](../decisions/app-features.md)):

- The dialog names the findings in plain words, errors and warnings in two lists, and offers
  **Save anyway** and **Cancel — nothing is saved** ([Plan › Saving](../ui/plan.md#saving)).
- **Cancel** deletes `.partial/` and keeps the draft.
- **Save anyway** commits the batch with the findings in its `errors` and `warnings`; the Day card shows
  them until the batch is reverted or amended.

A reprint or a restore has no *anyway*: a finding stops it and nothing is kept.

## Messages

Each finding names the input page and the order: *page 12 of `labels_1.pdf` (order 580000000000000042)
is written 2 times: 2-B page 4, 2-Z page 9.* — *page 13 of `labels_1.pdf` (order …) is in no output.*

## Cost

The digests are the work: about 7 s for 600 pages without the per-object reuse, much less with it. *Save
batch* shows its stage and a progress bar (`save-progress`: *Copying inputs*, *Writing*, *Verifying*,
*Saving*), so the window never looks frozen.

## What it cannot find

An order the reader never saw (a label missing from the download), and a choice the files state
consistently but the owner did not want (an order in the wrong group by a wrong filter): the verifier proves
that the outputs are what the files say. That the files say the right thing is proved by the
known-answer tests ([code rules](../../development/code-rules.md#tests)) and checked by the owner on the
Plan screen.
