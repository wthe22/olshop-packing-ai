# 02 — Design Questions

Questions that need the owner's choice before the affected part is built. Add new ones as
`D1`, `D2`, … with the options; remove one once its answer is written into the other documents.

## D1 — The Android app and the pick flow

The PC script now works with picks ([03-pc-script.md](03-pc-script.md)); the app design is still
the old one: batches, pack groups, group numbers and scan filters on batch/group
([01-requirements.md](01-requirements.md) A-sections,
[05-app-architecture.md](05-app-architecture.md), [06-app-ui.md](06-app-ui.md)). Before phase 2
the app is revised to match the pick flow. Open until then.

## D3 — Packing-list default

The owner chooses the packing-list scope per invocation (`per-pdf`, `whole`, `none`), and the
layouts `full`, `summary`, `pick` keep their look ([03-pc-script.md](03-pc-script.md)). What is
open is the default:

1. `per-pdf`, layout `full` (a sheet with tracking IDs for every saved PDF).
2. `none`; print on request.
3. Ask at the end of every run.
