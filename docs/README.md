# Packing Checker (PC app) — Documentation

The PC app turns the shipping-label PDFs with packing slip of the shop's daily orders (TikTok Shop +
Tokopedia, shipped by J&T, SiCepat, IDX) into picks — one saved label PDF per pick — and A4 packing
lists. It is a Windows program (Rust + Tauri).

| Folder | Contains | Read when |
|---|---|---|
| [`business-process/`](business-process/README.md) | How the packing work is done today and the files it uses. True even without the software | You need to know *why*, or what the real data looks like |
| [`software-design/`](software-design/README.md) | The target system only: requirements, label and pick rules, rules file, the app, its screens, `decisions/`, `questions/` | You build or change the app |
| [`development/`](development/README.md) | How to set up, build, run and test (`setup.md`, `cli.md`, `app.md`, `testing.md`) and `plan/`: only the unfinished work | You start or continue development |
| [`reviews/`](reviews/README.md) | The owner's feedback on the running app and ideas, one file per scope | You plan the next round of changes |
| [`user-guide/`](user-guide/README.md) | How the owner uses the finished app, step by step with pictures | You use the app or change what the owner sees |
| `archive/` | Superseded records (old review rounds, handoffs, setup logs, finished plans) and `id-map.md` (old ID → new ID). Never linked from the other documents | You look for the origin of an old ID or a finished task |

## Which folder does a statement belong to

| Test | Folder |
|---|---|
| It would be true even if no software existed | `business-process/` |
| It is a table, rule or screen of the target system | `software-design/` |
| It tells how to build or run, or what is left to build | `development/` |
| It is the owner's reaction to the running app | `reviews/` |
| It is finished or obsolete | `archive/`, or deleted |

## Scopes

A scope groups the documents of one part of the system. IDs carry it.

| Scope | Part | Design folder |
|---|---|---|
| LABEL | Reading labels and packing slips, courier deduction, picks, saved PDFs, the packing list | `software-design/labels/` |
| RULES | The picks groupings (`categories.toml` and the data folder's `categories/`) and the condition language | `software-design/rules-file/` |
| APP | One batch, the data folder, `state.json`, revert and amend, architecture, the program folder | `software-design/app/` |
| UI | The window and every screen | `software-design/ui/` |

| ID | Meaning | Lives in |
|---|---|---|
| `D-<SCOPE>-n` | A settled decision | `software-design/decisions/` |
| `Q-<SCOPE>-n` | An open question, with options and a default | `software-design/questions/` |
| `R-<SCOPE>-n` | A review item or idea of the owner | `reviews/<scope>.md` |
| `F-<SCOPE>-n` | A requirement (a feature of the target system) | `software-design/requirements.md` |

Numbers are never reused. A resolved question or review item is deleted from its file; its answer is
written into the design. Phase and task numbers appear only in `development/plan/`. Code comments may
still cite old IDs (`D2`, `D24`, …); `archive/id-map.md` maps them to the new ones.

## Writing rules

- The owner is "the owner", never he, she, they, it or you (the user guide, which instructs the owner, is
  the one place that addresses the reader).
- Live documents state the present only: no history, no changelog wording. Git keeps the past.
- `software-design/` holds the target only: no plans, phases, build tasks or build status.
- About 150 lines per file at most. Split a big topic into a folder with a `README.md` index.
- Each fact has one home; link instead of repeating.
- The dropship supplier's real name never appears: write "the supplier".
- `samples/`, `work/` and `tmp/` hold real data. They are confidential and never committed. Documents
  and committed test data use made-up IDs only (`JY0000001234`, `580000000000000001`).

## Words used everywhere

| Word | Meaning |
|---|---|
| **Order** | One platform order, identified by its `Order ID` — the number its label prints. Has one tracking ID and one or more item lines |
| **Tracking ID** | The courier's parcel number, e.g. `JY0000001234`. Printed on the label as text, 1D barcode and QR code |
| **Item line** | One product with a variation and a quantity inside an order, read from the packing slip |
| **Packing list** | The set of item lines of one order; also the A4 sheet that lists the runs. Two orders with the same set are packed the same way |
| **Pick** | One rule in a grouping; the app writes the orders it matches as one saved label PDF |
| **Saved PDF** | The label PDF of one pick, numbered through the day (1, 2, 3, …) |
| **Run** | Inside a saved PDF, consecutive orders with identical contents; numbered 01, 02, … and written `3-05` |
| **Category** | An owner-defined rule ("name contains sepatu"): a pick, shown as a section heading in the app; one entry of a grouping |
| **Grouping** | A named collection of categories; a batch is packed with the grouping chosen for it ([rules file](software-design/rules-file/README.md)) |
| **Batch** | One round of arrange shipment → download → prepare, numbered 1, 2, 3, … within the day: one label download prepared and saved together (`labels/<day>/batch 2/`) |
