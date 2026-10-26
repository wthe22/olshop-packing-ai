# Decisions

Settled choices of the owner that shape the design, one record each. The rule itself is written in the design
files; a record gives the reason and links to where the rule applies.

IDs are `D-<SCOPE>-n`; the scopes are listed in [../../README.md](../../README.md#scopes). A number is never
reused. A decision that changes gets a new record that names the one it replaces, and the old record goes.

| File | Records |
|---|---|
| [app.md](app.md) | D-APP-1, D-APP-3 to D-APP-12: PDFium, the schema version, order data from the labels, the workspace, the courier list, the journal, views, one writer, inputs by content, outputs, the verifier |
| [app-features.md](app-features.md) | D-APP-13 to D-APP-23: the rules file's place, reprint, the search index, the summary PDF, one workspace per shop owner, findings stop the save, amend, item identity, ship-by date, repeats |
| [label.md](label.md) | D-LABEL-2, D-LABEL-4 to D-LABEL-18: two clues, remembered images, the Shopee-style label, appearance hash, instant labels, pickup code, the header band, copies, page identities, the courier clue, order contents, the summary PDF's parts, the day summary, numbered order contents and one layout |
| [rules.md](rules.md) | D-RULES-1, D-RULES-2: choosing and renaming a sort rule |
| [ui.md](ui.md) | D-UI-1 to D-UI-6: the rest row, no log file, search scope, *No groups yet*, the panel, where repeats are looked for |

Open choices are in [../questions/](../questions/README.md).
