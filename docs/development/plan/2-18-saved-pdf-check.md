# 2.18 Check the saved PDFs before the batch is saved

The owner's request: the saved PDFs and the packing lists are read back and compared with the plan
before the batch is saved, so a doubled page, a dropped page, an order in two saved PDFs or in none,
or a wrong page order is caught before printing. The findings, their two severities, what the dialog
offers and how the check reports progress are in
[saved-pdf-check.md](../../software-design/app/saved-pdf-check.md); the check runs on every save, with
no size condition ([D-APP-2](../../software-design/decisions/app.md)).

- **Steps:** after the saved PDFs and the packing lists are written and before `batch <b>.partial/`
  is renamed, the app reads its own output back with the batch's page reader
  ([batch.md](../../software-design/app/batch.md), *What happens in one batch* step 2, so no new
  parsing) and compares it with the plan.
- **Code:** `engine/src/verify.rs` (the arithmetic, no PDF), `pdf/src/verify.rs` (page counts, the
  read-back comparison, the packing-list counts), called from `pdf/src/batch.rs` between the writes
  and the rename; `PlanScreen.svelte` + `texts.ts` for the dialog and the two lists. Design into
  [batch.md](../../software-design/app/batch.md) (a *Check before saving* step) and
  [plan.md](../../software-design/ui/plan.md) / [messages.md](../../software-design/ui/messages.md).
- **Tests:** a doubled page, a dropped page, an order in two picks (and in none) and a swapped page
  order must each be reported, each in its own list (the swapped one as a warning only); the tests
  fail when the check is switched off, as the 2.15 display tests do. Behind it: the smaller seams too
  — a page count changed in a written file, a packing-list count changed.
- **Result:** (not started)
