# Open questions: LABEL

Each question gives the problem, the options, and the **Default**: what the work does until the owner decides.
Answered questions are removed, and their answer goes into [decisions/](../decisions/README.md).

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
