# Day folder

```
<data folder>/
  categories.toml          the grouping `default` ([rules file](../rules-file/README.md))
  categories/              every other grouping, one file each, the file name being the grouping name
    ramadhan.toml          the grouping `ramadhan`
  couriers.toml
  settings.json
  labels/
    2026-10-07/                  ← the session Main of that day
      state.json
      batch 1/
        1 A Sepatu ×172.pdf
        2 B Spion & Knalpot ×35.pdf
        3 Z Lainnya ×393.pdf
        packing-list.pdf
        summary.txt
        download/
          10-07_07-00-00_Shipping label+Packing slip_1.pdf
          10-07_07-00-09_Shipping label+Packing slip_2.pdf
          10-07_07-00-17_Shipping label+Packing slip_3.pdf
      batch 2/
        4 A Sepatu ×88.pdf
        5 Z Lainnya ×41.pdf
        packing-list.pdf
        summary.txt
        download/
          …
```

- Saved-PDF numbers run through the session (batch 2 starts at 4), so with the suggested names
  "sort by name" inside a batch folder is printing order, and a number on the packing list is
  unique for the whole session.
- Batch folders are `batch 1`, `batch 2`, …. Saved-PDF names are the owner's: suggested
  `<n> <code> <name> ×<orders>`, the whole name can be changed
  ([Plan screen](../ui/plan.md)); two saved PDFs of one batch cannot have the same name.
  Characters not allowed in Windows file names (`\ / : * ? " < > |`) become `-`; a name is cut
  at 100 characters (ending in `…`).
- Packing-list files: scope `whole` (the default) → one `packing-list.pdf` per batch; scope
  `per-pdf` → `packing-list-<n>.pdf` per saved PDF; with several layouts the layout is added
  (`packing-list-full.pdf`, `packing-list-3-pick.pdf`).
- `download/` holds the label files exactly as read (about 1.3 MB per 200 pages, ~4 MB for a
  day of 600 orders), so amend works even when the originals in Downloads are gone.
- **A session** is one folder of `labels/`: `<date>` for Main, `<date> <name>` for a named session
  ([sessions](sessions.md)); every file above lives inside it, so each session has its own numbers.
  The app opens on the PC's date, Main, and the [Sessions panel](../ui/sessions.md) opens any other
  session (e.g. a download just after midnight that belongs to the evening before).

## `state.json`

One per session folder. Everything needed to continue the session, revert or amend a batch.

```json
{
  "day": "2026-10-07",
  "batches": [
    {
      "batch": 1,
      "save_time": "2026-10-07T07:40:12+07:00",
      "set": "default",
      "files": [
        {"name": "10-07_07-00-00_Shipping label+Packing slip_1.pdf",
         "size": 1310720, "xxh3": "9f2c4a1d7be08356", "pages": 200}
      ],
      "pdfs": [
        {"number": 1, "code": "A", "name": "Sepatu", "file": "1 A Sepatu ×172.pdf",
         "orders": 172, "runs": 4}
      ],
      "packing_list": {"scope": "whole", "layouts": ["pick"]},
      "printed": [1],
      "warnings": ["Courier unknown for 1 order: JY0000000107. They are sorted normally; only a condition on courier cannot see them."],
      "errors": ["PDF 2 (2 B Spion & Knalpot ×35.pdf): the page of order 580000000000000001 is missing (0 of 1 written)."]
    }
  ],
  "saved": [
    {"order_id": "580000000000000001", "tracking_id": "JY0000001234",
     "batch": 1, "pdf": 1, "run": 5, "courier": "J&T Express",
     "total_quantity": 2, "distinct_items": 1, "ship_by": "2026-10-07T17:00:00",
     "items": [{"name": "Sepatu", "variation": "42",
     "seller_sku": "SP-0001", "quantity": 2}]}
  ]
}
```

| Field | Holds | Why |
|---|---|---|
| `batches[].files` | Name, size, XXH3-64 (16 hex characters of the whole file), page count of each label file read, in reading order | Recognise a file used twice ("used in batch 1"); amend reads `download/` in this order |
| `batches[].pdfs` | Each saved PDF: number, pick code/name, file name, counts | Day screen without re-reading the PDFs; revert knows which files are the batch's |
| `batches[].packing_list` | Scope and layouts chosen | Amend starts with the same choice |
| `batches[].set` | The name of the grouping the batch was packed with ([rules file](../rules-file/README.md)); absent loads as `default` | An **Amend** reads the batch's own grouping again, whatever grouping was used last; the [Plan screen](../ui/plan.md) header and **Amend**'s confirmation ([Day screen](../ui/day.md)) name it |
| `batches[].printed` | The saved-PDF numbers of the batch whose **Printed** tick is set ([Day screen](../ui/day.md)); written when a tick is clicked; absent loads as empty | The tick is a note only, and it belongs to the batch: a revert removes it with the batch, and a batch written again by an amend starts unticked |
| `batches[].warnings` | The read warnings of the batch, as shown on the Plan screen, plus the check before saving's warnings ([saved-pdf-check.md](saved-pdf-check.md)); absent loads as empty | Day screen *Warnings ▸* without re-reading the files |
| `batches[].left_out` | The copies read but not written, each with its order, file, page, reason (`repeat`, `saved_earlier`) and the copy that stays ([duplicates](duplicates.md#what-is-stored)); absent loads as empty | Day screen *Left out ▸*; the page ledger of the check ([saved-pdf-check.md](saved-pdf-check.md)); an amend preselects the kept copies |
| `batches[].errors` | The check before saving's errors, kept apart from the warnings; written only when the batch was saved anyway (*Save anyway*), absent loads as empty | Day screen *Saved with `<n>` errors ▸* without re-reading the files |
| `saved[]` | Each saved order: IDs, batch, saved PDF, run, and the fields the Search panel finds it by (`courier`, `total_quantity`, `distinct_items`, `ship_by`, `items` with each line's `name`, `variation`, `seller_sku`, `quantity`) | Duplicate guard; Search ([Search screen](../ui/search.md)). Older orders keep only their IDs ([Q-UI-3](../questions/ui.md)) |

The file is written whole each time (write to `state.json.tmp`, then replace), so a crash
never leaves half a file. A file entry may instead carry `sha256` (64 hex characters): it
loads and is never matched — the *used in batch n* hint only matters inside one session — and a save
writes `xxh3` only. The app does not read the analysis helpers' `work/<day>/state.json`.
