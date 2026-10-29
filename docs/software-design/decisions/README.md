# Decisions

Settled choices of the owner that shape the design, one record each. The rule itself is written in the design
files; a record gives the reason and links to where the rule applies.

IDs are `D-<SCOPE>-n`; the scopes are listed in [../../README.md](../../README.md#scopes). A number is never
reused. A decision that changes gets a new record that names the one it replaces, and the old record goes.

| File | Records |
|---|---|
| [app.md](app.md) | D-APP-1, D-APP-3 to D-APP-12: PDFium, the schema version, order data from the labels, the workspace, the courier list, the journal, views, one writer, inputs by content, outputs, the verifier |
| [app-features.md](app-features.md) | D-APP-13 to D-APP-24: the rules file's place, reprint, the search index, the summary PDF, one workspace per shop owner, findings stop the save, amend, item identity, ship-by date, repeats, renaming and deleting a workspace |
| [label.md](label.md) | D-LABEL-2, D-LABEL-4 to D-LABEL-6, D-LABEL-9 to D-LABEL-15, D-LABEL-21: two clues, remembered images, the Shopee-style label, appearance hash, the header band, copies, page identities, the courier clue, order contents, instant labels by their tag |
| [summaries.md](summaries.md) | D-LABEL-16, D-LABEL-19, D-LABEL-20, D-LABEL-23: the summary PDF's parts, one row everywhere and couriers by name, the day summary as the Day cards, its inputs and its order contents by most orders |
| [rules.md](rules.md) | D-RULES-1 to D-RULES-4: choosing and renaming a sort rule, *is empty*, a courier typed by name or picked by code |
| [ui.md](ui.md) | D-UI-1 to D-UI-4, D-UI-6, D-UI-8, D-UI-9, D-UI-11, D-UI-12: the rest row, no log file, search scope, *No groups yet*, where repeats are looked for, the theme in Settings, the panel, the look, Search's rows and *contains* |

Open choices are in [../questions/](../questions/README.md).
