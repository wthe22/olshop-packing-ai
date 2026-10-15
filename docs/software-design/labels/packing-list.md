# Packing list (A4)

- The layouts `full`, `summary`, `pick` each keep their own look. A heading is the saved PDF
  number and the pick: `1  A  Sepatu`. The groups under a heading are the runs, numbered `3-05`;
  in `full` each run's tracking IDs are printed.
- Scope, chosen per batch: `per-pdf` (one sheet per saved PDF), `whole` (one sheet with a
  section per saved PDF), `none`.
- A packing list needs the slips' item lines; every page of a batch carries a slip
  ([inputs.md](../app/inputs.md)).
- Header `Packing list · <day> · <n>  <code>  <name>` (scope `per-pdf`) or
  `Packing list · <day> · saved PDFs <first>-<last>` (scope `whole`), then the counts
  (`<orders> orders · <units> units · <runs> runs · printed hh:mm`) and `page x/y`.
- The header takes the top 23 mm: the title (13 pt bold) and `page x/y` in the first 7 mm, the
  counts line (9.5 pt) in the next 5 mm, then a 1 mm gap. **The body starts at 23 mm on every
  page** — the first page and after every page break, the `(continued)` line included. A body row
  at the top margin is printed over the title and the counts.
- Fonts: Arial 9.5–13 and Consola 10 from `C:\Windows\Fonts`, embedded whole. PDFium's *non-CID*
  font path writes `× ÷ é ® ™ ° ± ñ Ä ½ €` as `ÿ`, so those characters are drawn with a **CID copy
  of the same font**; the CID font's own text layer turns a plain space into U+00A0 and `-` into
  U+00AD, so it carries those characters only and the rest of the text keeps a clean text layer.
- The pick summary counts units per item = (product name, variation), as slip lines have no
  SKU ID.

```
Packing list · 2026-10-07 · 2  B Spion & Knalpot                               page 1/1
35 orders · 35 units · 4 runs · printed 08:05
▓ 2  B  Spion & Knalpot ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓ 35 orders
2-01  Spion Beat  ×1                                                                6
    JY0000001234  JY0000001235  JY0000001236  JY0000001237  JY0000001238  JY0000001239
──────────────────────────────────────────────────────────────────────────────────────
2-02  Spion Beat  ×1 + Knalpot Beat  ×1                                              1
    JY0000001240
```
