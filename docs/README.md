# Packing Checker (PC app) — Documentation

The PC app turns the shipping-label PDFs with packing slip of one or more shop owners' daily orders (TikTok
Shop, Tokopedia, Shopee-style; shipped by J&T, SiCepat, IDX, SPX, …) into label groups — one PDF per group —
and an A4 summary per batch, each verified before it is saved. It is a Windows program (Rust + Tauri).

| Folder | Contains | Read when |
|---|---|---|
| [`business-process/`](business-process/README.md) | How the packing work is done today and the files it uses. True even without the software | You need to know *why*, or what the real data looks like |
| [`software-design/`](software-design/README.md) | The target system only: the workflow and its invariants, requirements, data and journal, label and sort rules, the rules file, the app, its screens, `decisions/`, `questions/` | You build, change or check the app |
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
| LABEL | Reading labels and packing slips, courier deduction, label groups and order contents, the summary PDF | `software-design/labels/` |
| RULES | `packing-workflows.toml` (couriers, sort rules) and the filter language | `software-design/rules-file/` |
| APP | Architecture, namespaces, the data folder and the journal, save, revert and amend, the verifier, reprint, the program folder | `software-design/app/`, `software-design/data/`, `software-design/workflow/` |
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
| **Item line** | One product with a variation, a Seller SKU and a quantity inside an order, read from the packing slip. An item is its name and variation |
| **Order content** | The item lines of one order as (name, variation, quantity). Orders with the same order content are packed the same way and are printed next to each other in their group PDF |
| **Namespace** | One shop owner whose labels the app prepares (an **owner** on screen): a folder with one folder per date |
| **Sort rule** | A named list of label groups in `packing-workflows.toml`; a batch is sorted by the rule chosen for it ([rules file](software-design/rules-file/README.md)) |
| **Label group** | One entry of a sort rule: a code, a name and a filter (`name contains "sepatu"`); the orders it takes are written as one **group PDF**, `2-A Sepatu x88.pdf` |
| **Summary PDF** | The one A4 sheet set of a batch: the units to take from stock and the batch's facts |
| **Batch** | One round of arrange shipment → download → prepare, numbered 1, 2, 3, … within a namespace and date: one label download prepared and saved together |
| **Journal** | The one file of a namespace and date that holds every saved fact; every other data file is an input, an output or a view of it |
| **Copy** | One appearance of an order's label in the files. An order with two or more copies in a batch is a repeat |
