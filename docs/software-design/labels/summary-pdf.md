# Summary PDF (A4)

One per batch, `output/{batch} summary.pdf`, written with the group PDFs and checked by the verifier
([D-APP-16](../decisions/app-features.md)). There is no other packing list and no layout or scope to choose.

## The pick pages

What to take from stock for the whole batch: one line per item (product name and variation), its
**display name** and its units, in two columns, sorted by display name.

```
Summary · Toko Contoh · 2026-10-07 · batch 2                                           page 1/2
129 orders · 141 units · 2 groups · printed 09:12
▓ Pick summary (units to take from stock) ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓
  96  Sepatu Standar Samping Motor               12  Spion Beat — Standard, honda
  26  Sepatu Standar Samping Motor — Hitam        7  Knalpot Beat
```

- Header `Summary · <owner> · <date> · batch <b>`, then `<orders> orders · <units> units · <groups> groups ·
  printed hh:mm`, and `page x/y`.
- The header takes the top 23 mm: the title (13 pt bold) and `page x/y` in the first 7 mm, the counts line
  (9.5 pt) in the next 5 mm, then a 1 mm gap. The body starts at 23 mm on every page.

## The facts page

The last page says what the batch is, so the paper alone can be traced:

| Section | Holds |
|---|---|
| Groups | Each group PDF on a line of its own: *2-A Sepatu x88.pdf · 88 orders · 90 pages · 3 order contents* |
| Inputs | Each input: the name it had, pages, the first 12 characters of its SHA-256 |
| Ledger | *1000 pages read = 986 written + 14 left out (14 repeats, 0 saved earlier)* |
| Instant orders | Each instant-service order: Order ID, its group, its **pickup code** ([D-LABEL-8](../decisions/label.md)) |
| Warnings | The read warnings. The verifier's findings come after this file is written, so they are on the Day card and in the journal, not here |
| Sort rule | Its name and each group's code, name and filter, as used |
| Versions | `schema 1.0 · Packing v0.2 · PDFium <version>` |

A section with nothing to list says *none*.

The verifier reads three things back as text and compares them with the record (V6 of
[verification](../app/verification.md)): the header's `<orders> orders · <units> units · <groups> groups`,
the ledger's `<read> pages read = <written> written + <left> left out`, and each group's line by its tail
`<orders> orders · <pages> pages · <contents> order contents`, in group order. Their words and order are
fixed for that reason, and the numbers have no thousands separator. The file name before a group's tail is
not read: `lopdf` reads no text of a character drawn with the CID font (an `é` in a name).

## Fonts

Arial 9.5–13 and Consolas 10 from `C:\Windows\Fonts`, embedded whole (about 1.4 MB a file, accepted:
[D-APP-1](../decisions/app.md)). PDFium's non-CID font path writes `× ÷ é ® ™ ° ± ñ Ä ½ €` as `ÿ`, so those
characters are drawn with a **CID copy of the same font**; the CID font's text layer turns a space into
U+00A0 and `-` into U+00AD, so it carries those characters only and the rest keeps a clean text layer — the
verifier reads the counts from that layer.
