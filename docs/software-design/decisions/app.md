# Decisions: app

## D-APP-1 — PDFium does all three PDF jobs

**Decision.** One PDF library, PDFium, reads the text of a page with its positions, copies pages
unchanged into a saved PDF, and writes the A4 packing list.

**Why.** On the sample label downloads it read the Order ID on every page and gave the same slip
lines as an independent reading of the same files; copied pages render with no pixel difference;
reading 601 pages takes 1.6 s. The packing-list files are larger (about 1.4 MB for one page, 1.6 MB
for five) because PDFium writes whole fonts; they print the same, and the size is accepted.

**Applies to.** [architecture.md](../app/architecture.md), [packing-list.md](../labels/packing-list.md).

## D-APP-2 — The check before saving runs on every save

**Decision.** The saved PDFs and the packing lists are read back and compared with the plan on
every save, with no size condition.

**Why.** A wrong page order or a doubled page is otherwise found only at the printer; the check
costs 1–2 s on a large day.

**Applies to.** [saved-pdf-check.md](../app/saved-pdf-check.md), [batch.md](../app/batch.md).
