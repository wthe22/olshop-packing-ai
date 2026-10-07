# 02 — Design Questions

Questions that need the owner's choice before the affected part is built. Add new ones as
`D1`, `D2`, … with the options; remove one once its answer is written into the other documents.

## D1 — The Android app and the pick flow

The PC script now works with picks ([03-pc-script.md](03-pc-script.md)); the app design is still
the old one: batches, pack groups, group numbers and scan filters on batch/group
([01-requirements.md](01-requirements.md) A-sections,
[05-app-architecture.md](05-app-architecture.md), [06-app-ui.md](06-app-ui.md)). Before phase 2
the app is revised to match the pick flow. Open until then.

## D2 — How interactive mode looks

With `--interactive` the owner builds the picks by hand ([03-pc-script.md](03-pc-script.md)).
The flow is fixed (show what is left, type a condition, save, repeat, save the rest); the
wording and what is shown are open. Options:

1. **Plain prompts.** A count of what is left (and the largest contents groups), then a prompt
   for a condition; an empty line saves the rest. Least code.
2. **Numbered menu.** The rules-file entries that still match, numbered with their counts; the
   owner types a number to save that pick, `c` to type a condition, `r` to save the rest. More
   helpful, more code.
3. **Rules file, editable.** The script offers the rules-file entries one by one
   (`Save A Sepatu (172 orders)? [Y/n/edit]`); `edit` replaces the condition; the rest is saved
   at the end.

## D3 — Packing-list default

The owner chooses the packing-list scope per invocation (`per-pdf`, `whole`, `none`), and the
layouts `full`, `summary`, `pick` keep their look ([03-pc-script.md](03-pc-script.md)). What is
open is the default:

1. `per-pdf`, layout `full` (a sheet with tracking IDs for every saved PDF).
2. `none`; print on request.
3. Ask at the end of every run.
