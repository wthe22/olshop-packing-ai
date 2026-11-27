# Decisions

Settled choices of the owner that shape the design, one record each. The rule itself is written in the design
files; a record gives the reason and links to where the rule applies.

IDs are `D-<SCOPE>-n`; the scopes are listed in [../../README.md](../../README.md#scopes). A number is never
reused. A decision that changes gets a new record that names the one it replaces, and the old record goes.

| File | Records |
|---|---|
| [app.md](app.md) | D-APP-1, D-APP-3, D-APP-4, D-APP-6, D-APP-9 to D-APP-12, D-APP-32, D-APP-33, D-APP-39: PDFium, the schema version, order data from the labels, the courier list, a date's truth in day.toml and its two files, one writer, inputs by content, outputs, the verifier, one flat folder per date, migrations ask first |
| [app-features.md](app-features.md) | D-APP-14 to D-APP-16, D-APP-18 to D-APP-23, D-APP-26, D-APP-30, D-APP-31: the three config files' places, reprint, the search index, the summary PDF, findings stop the save, amend, item identity, ship-by date, repeats, Print and the Sent mark, a revert's saved label PDFs |
| [date-files.md](date-files.md) | D-APP-40, D-APP-41: a changed or missing file of a date, Check data; both read labels again |
| [ui-screens.md](ui-screens.md) | D-UI-17 to D-UI-19: no examples in a box, reading stays on the file list with each stop under its file, a summary read in the app |
| [versions.md](versions.md) | D-APP-34, D-APP-35: a copy reads its own major version and passes over what it does not know; `day.toml` and the version column of `orders.csv` |
| [accounts.md](accounts.md) | D-APP-27, D-APP-29, D-APP-37: accounts replace workspaces, an account's summary on request, an order's account is its shop's account now |
| [label.md](label.md) | D-LABEL-2, D-LABEL-4 to D-LABEL-6, D-LABEL-9 to D-LABEL-13, D-LABEL-15, D-LABEL-21, D-LABEL-24 to D-LABEL-27, D-LABEL-29: two clues, remembered images, the Shopee-style label, appearance hash, the header band, copies and the split label, page identities, order contents, instant labels by their tag, the pickup code as the tracking ID, the regular SPX badge, the receiver's area and the weight as printed, the courier clue not stored |
| [summaries.md](summaries.md) | D-LABEL-16, D-LABEL-19, D-LABEL-20, D-LABEL-23: the summary PDF's parts, one row everywhere and couriers by name, the day summary as the Day cards, its inputs and its order contents by most orders |
| [rules.md](rules.md) | D-RULES-2 to D-RULES-7: renaming a sort rule, *is empty*, a courier typed by name or picked by code, switching a rule off, account and shop in the filter language, the rule chosen after reading and *Custom* |
| [ui.md](ui.md) | D-UI-1 to D-UI-4, D-UI-6, D-UI-8, D-UI-11, D-UI-13 to D-UI-16: the rest row, no log file, search scope, *No groups yet*, where repeats are looked for, the theme in Settings, the look, the Accounts screen, the panel of dates, a batch on a date that is not today, Search's cards under heads and *contains* |

Open choices are in [../questions/](../questions/README.md).
