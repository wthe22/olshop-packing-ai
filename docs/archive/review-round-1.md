# Phase 2 — Owner review, round 1

Tasks: [plan.md](plan.md). Rows 1 and 2 were fixed by 2.15, row 3 belongs to 2.16 (and 2.19),
row 4 was fixed by 2.17.

Held 2026-10-11, on the owner's own day's data. The owner ran a real day
through the portable folder (2 batches, 4 label files, 98 orders, no terminal) and reported two
things; checking them found a third.

| # | Reported / found | What it is | Cause |
|---|---|---|---|
| 1 | "packing list isn't displayed correctly, some text overlap" | Every page of every layout prints the first body row or bar over the header (title 10–17 mm, counts 17–22 mm): the body starts at the top margin (10 mm) instead of below the header (23 mm) | The Rust header is a set of fixed ops that never advances the cursor; `Builder` starts and restarts at `MARGIN` |
| 2 | found while checking 1 (the owner has not seen it: the pick layout has no `×`) | `×` prints as `ÿ`, in the drawn glyph **and** in the text layer, in the run lines of `full`/`summary`; `·` and `—` are right | PDFium writes U+00D7 wrongly for a non-CID TrueType font; the app loads the fonts with `cid = false` |
| 3 | "some couriers are not recognized by the current parser" | 23 of 98 orders save with an empty courier | SiCepat labels carry the name **only as a logo image**, and their `00` + 10-digit tracking form is marked as shared with J&T Cargo, so the deduction refuses |
| 4 | "open folder doesnt open the correct folder. it opens the user's userprofile documents instead" | *Open folder* on a batch and on the data folder opened *Documents*, later *Home* — never the folder | `explorer.exe /select,<path>`: Explorer parses `/select,` itself and does not understand the quotes Rust puts around an argument with a space in it, so it read no target and fell back to Documents (*Home* when the target is empty) |

Evidence: the owner's `batch 1`/`batch 2` `packing-list.pdf` (both 1 page, layout `pick`)
rendered and read back, against the Python reference (`pc/spike/out/python-packing-list.pdf`);
`testdata/labels-slip.pdf` through `packing-cli prepare --layout full,summary,pick` — all three
layouts show the overlap; the 4 label files of 2026-10-11 and the 3 of `samples/` read for the
courier clues, counts only; the pages with no courier text rendered (they are `SICEPAT EKSPRES`,
the name only in the logo); PDFium probed with a non-CID and a CID font; the two Explorer forms run
from a probe in the owner's own session (the fixed form opened *batch 1 - File Explorer* on the
batch folder, the `/select,` form opened no such window and fell back to Documents/Home). The owner's data folder
was only read — nothing in it was written to or changed; every probe wrote outside it.
