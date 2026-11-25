# Open questions: LABEL

Each question gives the problem, the options, and the **Default**: what the work does until the owner decides.
Answered questions are removed, and their answer goes into [decisions/](../decisions/README.md).

## Q-LABEL-14 — The courier of the regular SPX layout, which has no SPX logo

The regular SPX label (`2026-11-23 spx regular.pdf`) reads its tracking ID `CM` + 11 digits and its order ID.
Its courier is not named: the file has no image of the SPX wordmark (its images are the barcodes, a
`REGULAR SERVICE` badge and the Shopee wordmark), and the two-clue rule needs a logo or shape that agrees with
the tracking form. A page with a header image takes no text clue, so the printed text does not name it either.

- **A. Name the courier by the tracking form alone** when the form fits one courier and the page has no
  logo of that courier: a new clue kind, shown on the copy's clue. The two-clue rule stays for every other
  page.
- **B. Keep the two-clue rule.** The regular label reads with no courier (a warning, the order is still sorted
  and saved), and the owner adds its courier later from a logo the app shows, if the layout ever prints one.
  *(Default)*
- **C. Name it by the text `Reguler`** on this layout: a text clue that works beside a header image, for this
  layout only.

Default B is what the work does now. The tracking ID is read either way.

## Q-LABEL-13 — A product name the Shopee-style instant label breaks inside a word

The Shopee *Same-day* label (Grab Express) breaks a long product name or seller SKU wherever the line is
full, inside a word too: `Cover Standar Sa` / `mping Universal`. When the break falls on a space, the space
is printed on neither line, so the page cannot tell `Sa` + `mping` (one word) from `Standar |` +
`Penyangga` (two words).

- **A. Join the lines with nothing between.** *Samping* and *SEPATU-HITAM* read right; a break that fell on a
  space glues two words (`Standar |Penyangga`). A filter on a word of the name matches except where two
  words were glued. *(Default)*
- **B. Join with a space, as on the other labels.** Two words stay apart, but a word broken by the line reads
  as two (`Sa mping`, `SEPATU-HITA M`), and a filter on that word misses the order.
- **C. A, and Search and the filters ignore spaces** when they compare a name, so `Standar |Penyangga` and
  `Standar | Penyangga` match the same filter. The name is still saved as A reads it.

With A or B, the same product read from a TikTok or Tokopedia label can be a different order content from
the one read from this label, and the summary PDFs list the two apart.
