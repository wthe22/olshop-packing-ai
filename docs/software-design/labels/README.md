# Reading labels and sorting them

The rules by which the app reads a label download and turns it into label groups, group PDFs and the
summary PDF: the Order ID of a page, the packing slip, the courier, the groups and their order contents, the
file names. All order data comes from the label and its packing slip ([D-APP-4](../decisions/app.md)).
Requirements: `F-LABEL-1` to `F-LABEL-6` in [requirements.md](../requirements.md).

## In this folder

- [reading.md](reading.md) — the per-order data, the Order ID, copies and multi-page orders, the page
  fingerprint, the packing slip.
- [courier.md](courier.md) — the courier from the logo and the tracking-ID form, the logo prompt, the clue
  each order records.
- [groups.md](groups.md) — label groups, order contents, file names and the already-saved guard.
- [summary-pdf.md](summary-pdf.md) — the one A4 summary PDF of a batch.
- [day-summary.md](day-summary.md) — the A4 summary of a whole date, made on request.
