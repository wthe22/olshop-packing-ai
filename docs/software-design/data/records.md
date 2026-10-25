# Journal records

The records of [the journal](journal.md), in the notation of the owner's draft. Every record also carries
`seq: u32`, `time: Datetime`, `app: String` and `type`. A field may be added to a record without a new
schema version; a field that changes meaning may not.

```
Started {                       // type "started", seq 1
  schema: String                // "1.0"
  namespace: String
  date: Date
}

BatchSaved {                    // type "batch_saved"
  batch: BatchID
  amends: bool                  // true: replaces the newest live batch, same number
  inputs: List<InputPdf>        // in reading order
  sort_rule: LabelSortRule      // the rule used, as it was (rules-file/README.md)
  contents: List<OrderContent>  // the distinct contents of this batch; an order names one by index
  orders: List<SavedOrder>      // in printing order: group by group, run by run, page order in a run
  left_out: List<LeftOutCopy>
  outputs: List<OutputFile>     // the group PDFs in group order, then the summary PDF
  warnings: List<Finding>
  errors: List<Finding>         // the verifier's errors; only on a batch saved anyway
}

InputPdf {
  pdf: PdfID
  sha256: String                // 64 hex characters; the file is input/{sha256}.pdf
  filename: String              // the name it had when it was chosen
  size: u64
  page_count: u16
  create_time: Datetime         // of the chosen file
  modify_time: Datetime
}

OrderContent {
  items: List<ItemPick>         // sorted by name, variant, quantity
  total_quantity: Option<u16>   // Qty Total printed on the slip; none on the Shopee-style slip
}

ItemPick {
  name: String
  variant: String               // empty for `Default` or none
  sku: Option<String>           // Seller SKU
  quantity: u16
}

SavedOrder {
  order_id: String
  tracking_id: Option<String>
  courier: Option<CourierCode>
  courier_clue: CourierClue
  content: u16                  // index into contents
  ship_by_date: Option<Date>
  pickup_code: Option<String>   // the instant-service label's Kode Pengambilan
  pdf: PdfID                    // the copy that is written
  page_start: u16
  page_end: u16
  group: String                 // LabelGroupRule.code
  run: u16                      // 1, 2, … within the group
}

CourierClue {
  kind: CourierClueKind         // logo | shape | text | form | tag | none
  value: Option<String>         // the hash, text or tag that named the courier
  form: Option<String>          // the tracking-ID form that agreed with it
}

LeftOutCopy {
  order_id: String
  pdf: PdfID
  page_start: u16
  page_end: u16
  reason: LeftOutReason         // repeat | saved_earlier
  kept: Kept                    // repeat: { pdf, page_start } of the copy written; saved_earlier: { batch }
}

OutputFile {
  file: String                  // the name in output/
  group: Option<String>         // the label group's code; none for the summary PDF
  pages: u16
  size: u64
  sha256: String
}

Finding {
  kind: FindingKind             // e.g. slip_total_differs, courier_unknown, page_doubled
  pdf: Option<PdfID>
  page: Option<u16>
  order_id: Option<String>
  text: String                  // the message as shown
}

BatchReverted { batch: BatchID }                              // type "batch_reverted"
PrintedMarked { batch: BatchID, group: String, printed: bool } // type "printed_marked"
OutputRestored { batch: BatchID, file: String, pages: u16, size: u64, sha256: String }

ReprintMade {                   // type "reprint_made"
  number: u8                    // 1, 2, … within the date
  file: String
  pages: u16
  size: u64
  sha256: String
  orders: List<ReprintedOrder>  // in printing order
}

ReprintedOrder {
  order_id: String
  date: Date                    // the date the order was saved
  batch: BatchID
  sha256: String                // its input, {namespace}/{date}/input/{sha256}.pdf
  page_start: u16
  page_end: u16
}

Recovered {                     // type "recovered"
  dropped_bytes: u64
  dropped_text: String          // the cut-off bytes, as UTF-8 with invalid bytes replaced
}
```

## Why each record holds what it holds

- A `batch_saved` is complete on its own: the verifier reads that one record (and the earlier live batches,
  for the already-saved guard) into its own types ([verification](../app/verification.md)).
- `inputs` repeats the facts of an input a batch reuses; the fold checks they agree.
- `contents` stores each distinct content once per batch: most orders share a few contents.
- The ledger (pages read, written, left out) is not stored: it follows from `inputs`, `orders` and
  `left_out`, and the verifier computes it.
- `courier_clue` and the page range make every order traceable to the page and the clue it came from.
