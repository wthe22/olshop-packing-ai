# Summary PDF (A4)

One per batch, `output/{batch} summary.pdf`, of all the batch's accounts, written with the group PDFs and
checked by the verifier ([D-APP-16](../decisions/app-features.md)). One account's summary is made on request
([below](#one-account)). There is no other packing list and no layout or scope to choose.
Three parts, each starting on a new page, in this order ([D-LABEL-16](../decisions/summaries.md)): the **facts
pages**, the **group pages**, the **pick pages**. Every page has the same header and every part the same
rows ([D-LABEL-19](../decisions/summaries.md)). The [day summary](day-summary.md) is drawn in the same layout.

```
Summary · All accounts · 2026-10-07 · batch 2                                           page 1/3
129 orders · 141 units · 2 groups · printed 09:12
▓ Accounts                                                                                    ▓
  Toko Alfa                                                   88 orders    96 units
  ─────────────────────────────────────────────────────────────────────────────────────────────
  Toko Beta                                                   41 orders    45 units
  ─────────────────────────────────────────────────────────────────────────────────────────────
▓ Couriers                                                                                    ▓
  J&T Express                                                 86 orders
  ─────────────────────────────────────────────────────────────────────────────────────────────
  SPX                                                         42 orders
  ─────────────────────────────────────────────────────────────────────────────────────────────
  Unknown                                                      1 order
  ─────────────────────────────────────────────────────────────────────────────────────────────
▓ Inputs                                                                                      ▓
  labels_1.pdf                                               130 pages            3f1c0a9be2d4
  ─────────────────────────────────────────────────────────────────────────────────────────────
  130 pages read = 129 written + 1 left out (1 repeats, 0 saved earlier)
```

## The layout

- The header takes the top 23 mm: `Summary · <All accounts | account> · <date> · batch <b>` (13 pt bold) and
  `page x/y` in the first 7 mm, `<orders> orders · <units> units · <groups> groups · printed hh:mm`
  (9.5 pt) in the next 5 mm, then a 1 mm gap. The body starts at 23 mm on every page.
- A section is a grey bar with its heading (11 pt bold) and, at its right, the section's own counts
  (9.5 pt); then its rows. A bar's heading, a row's text and everything at the right sit 2 mm in from the
  bar's ends, so they line up down the page.
- **A row** is the same in every part: 10.5 pt, 5.5 mm a line, wrapped at spaces, under a thin rule
  0.6 mm below its last line, the next row 0.6 mm below the rule. A **key** (bold: `A-1`, `2-S1`, a sort
  rule's group code) sits at the left and the text 15 mm right of it; a row without a key starts its text
  at the left. At the right, up to two **counts**, each a number ending at 152 or 178 mm with its word
  after it (*86 orders*, *1 order*), so the numbers of a page stand under each other; an input's SHA-256
  ends at the right edge.
- 1 mm above and below a bar, 3 mm after a section's last row. A section that breaks to a new page repeats
  its bar there as *<heading> (continued)*.

## The facts pages

The first pages say what the batch is, so the paper alone can be traced:

| Section | Rows |
|---|---|
| Accounts | The batch's orders and units per account, most orders first. Only on the summary of all accounts |
| Couriers | The batch's orders per courier, by the courier's **name** in `packing-workflows.toml` (its code when the file no longer names it), most first, *Unknown* last: *86 orders*. The count the owner hands over at the pickup |
| Warnings | The read warnings. The verifier's findings come after this file is written, so they are on the Day card and in the journal, not here |
| Inputs | Each input: the name it had, its pages, the first 12 characters of its SHA-256. Then the ledger in bold: *1000 pages read = 986 written + 14 left out (14 repeats, 0 saved earlier)* |
| Sort rule · `<name>` | Each group: its code as the key, then *Sepatu · name contains "sepatu"* |
| Versions | `schema 1.0 · Packing v0.2 · PDFium <version>` |

A section with nothing to list says *none*. The group PDFs are on the group pages; an instant order's
pickup code is on its own label and in the journal.

## The group pages

Each group PDF in group order, as the v0.1 packing list printed it: a bar with the group's code and name
(`A  Sepatu`) and, at its right, `<orders> orders · <units> units`; then each of its order contents in the
order the group PDF prints them, as rows: its **number** in the group as the key (`A-1`, `A-2`, … — the
group's code and the place, so a number names one content of the batch), its items as
`display name ×quantity`, one per line, and its number of orders at the right in bold. A content that does
not fit at the foot of a page starts on the next one under the bar again, *A  Sepatu (continued)*. No
tracking ID.

## The pick pages

What to take from stock for the whole batch: one line per item (product name and variation), its
**display name** and its units, in two columns, sorted by display name, under the bar
*Pick summary (units to take from stock)*.

## One account

A batch card's account button on the [Day screen](../ui/day.md) writes `{date}/{batch} summary
{account}.pdf` from the batch's record and opens it ([D-APP-29](../decisions/accounts.md)). It is a view
of the journal: written again on every request, not recorded, never read for a decision; a revert or an
amend deletes it. It is checked before it is kept (V6 on the account's orders); a finding stops it and
nothing is kept.

- The same layout and parts, of the orders whose `account` is that account: the header names the account
  and counts its orders, units and groups; the Couriers, the Warnings that name one of its orders, the group
  pages (a group with none of its orders is left out) and the pick pages are its own.
- **Inputs** lists the batch's inputs and the batch's ledger, as the summary of all accounts does: the
  inputs are the batch's, shared by its accounts. There is no Accounts section.
- It is written to `.partial` beside it and renamed over the old one; one open in a viewer that locks it
  stops the write with the file named.

## What the verifier reads

V6 of [verification](../app/verification.md) reads three things back as text and compares them with the
record: the first page header's `<orders> orders · <units> units · <groups> groups`, the ledger's
`<read> pages read = <written> written + <left> left out`, and each group bar's `<orders> orders · <units>
units`, in group order, looked for after the ledger. Their words and order are fixed for that reason — a group bar reads *1 orders* —
and the numbers have no thousands separator. A group bar is told from the header by the header's
`<groups> groups` after its units. The group's name before its counts is not read: `lopdf` reads no text of
a character drawn with the CID font (an `é` in a name). An account's row on the facts pages reads like a
group bar, which is why the bars are looked for only after the ledger.

## Fonts

Arial 9.5–13 and Consolas 10 from `C:\Windows\Fonts`, embedded whole (about 1.4 MB a file, accepted:
[D-APP-1](../decisions/app.md)). PDFium's non-CID font path writes `× ÷ é ® ™ ° ± ñ Ä ½ €` as `ÿ`, so those
characters are drawn with a **CID copy of the same font**; the CID font's text layer turns a space into
U+00A0 and `-` into U+00AD, so it carries those characters only and the rest keeps a clean text layer — the
verifier reads the counts from that layer. The non-CID path also writes too small an advance for
`— – … ’ “ ”`, so each of them is drawn as a piece of its own at its true place; in one piece with the
text, `—` would cover the space after it. Each piece's place is measured on a scratch page from which the
measured text is taken away again: PDFium's text layer merges a character with an identical one at the
same place, and a text measured a second time would put its `×` over the space before it.
