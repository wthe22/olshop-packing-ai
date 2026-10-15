# Day folder

```
<data folder>/
  categories.toml
  couriers.toml
  settings.json
  labels/
    2026-10-07/
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

- Saved-PDF numbers run through the day (batch 2 starts at 4), so with the suggested names
  "sort by name" inside a batch folder is printing order, and a number on the packing list is
  unique for the whole day.
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
- **The day** is the PC's date when the app is opened; the Day screen can switch to another day
  (e.g. a download just after midnight that belongs to the evening before).

## `state.json`

One per day folder. Everything needed to continue the day, revert or amend a batch.

```json
{
  "day": "2026-10-07",
  "batches": [
    {
      "batch": 1,
      "save_time": "2026-10-07T07:40:12+07:00",
      "files": [
        {"name": "10-07_07-00-00_Shipping label+Packing slip_1.pdf",
         "size": 1310720, "xxh3": "9f2c4a1d7be08356", "pages": 200}
      ],
      "pdfs": [
        {"number": 1, "code": "A", "name": "Sepatu", "file": "1 A Sepatu ×172.pdf",
         "orders": 172, "runs": 4}
      ],
      "packing_list": {"scope": "whole", "layouts": ["pick"]},
      "warnings": ["Courier unknown for 1 order: JY0000000107. They are sorted normally; only a condition on courier cannot see them."]
    }
  ],
  "saved": [
    {"order_id": "580000000000000001", "tracking_id": "JY0000001234",
     "batch": 1, "pdf": 1, "run": 5, "courier": "J&T Express",
     "total_quantity": 2, "items": [{"name": "Sepatu", "variation": "42",
     "seller_sku": "SP-0001", "quantity": 2}]}
  ]
}
```

| Field | Holds | Why |
|---|---|---|
| `batches[].files` | Name, size, XXH3-64 (16 hex characters of the whole file), page count of each label file read, in reading order | Recognise a file used twice ("used in batch 1"); amend reads `download/` in this order |
| `batches[].pdfs` | Each saved PDF: number, pick code/name, file name, counts | Day screen without re-reading the PDFs; revert knows which files are the batch's |
| `batches[].packing_list` | Scope and layouts chosen | Amend starts with the same choice |
| `batches[].warnings` | The read warnings of the batch, as shown on the Plan screen | Day screen *Warnings ▸* without re-reading the files |
| `saved[]` | Each saved order: IDs, batch, saved PDF, run, and the fields the Search panel finds it by (`courier`, `total_quantity`, `distinct_items`, `ship_by`, `items` with each line's `name`, `variation`, `seller_sku`, `quantity`) | Duplicate guard; Search ([Search screen](../ui/search.md)). Older orders keep only their IDs ([Q-UI-3](../questions/ui.md)) |

The file is written whole each time (write to `state.json.tmp`, then replace), so a crash
never leaves half a file. The app does not read the analysis helpers' `work/<day>/state.json`.
