# Views

Three files of the owner's draft, written from the journal after every commit so a person or another
program can read a date without the app ([D-APP-8](../decisions/app.md)). Each is written to a temporary
file and renamed into place; `session.toml` is written last, so its `journal_seq` vouches for all three.
A view is never read for a decision. When one is missing, or `journal_seq` is not the journal's last
`seq`, all three are written again from the journal ([journal › When a date opens](journal.md#when-a-date-opens)).

Only live batches appear; a reverted batch is gone from the views and stays in the journal. IDs that
the journal does not hold (`RuleID`, `OrderContentID`) are numbered by the fold in order of first
appearance, so the same journal always gives the same views.

## `session.toml`

```
Root {
  schema: String                // "1.0"
  namespace: String
  date: Date
  journal_seq: u32              // the journal's last seq when this file was written
  input_pdfs: Map<PdfID, LabelPdf>
  label_sort_rules: Map<RuleID, LabelSortRule>   // each distinct rule snapshot once
  label_batches: List<LabelBatch>
  reprints: List<Reprint>
}

LabelPdf {
  filename: String
  size: u64
  hash_sha256: String
  create_time: Datetime         // of the chosen file
  modify_time: Datetime
  order_count: u16              // orders written from it
  page_count: u16
  earliest_ship_by_date: Option<Date>
  latest_ship_by_date: Option<Date>
  couriers: List<CourierCode>
  warnings: List<String>        // the findings that name this input
  errors: List<String>
}

LabelBatch {
  id: BatchID
  create_time: Datetime         // the first batch_saved of this number
  modify_time: Datetime         // the newest one (an amend)
  input_pdfs: List<PdfID>       // reading order
  sort_rule_id: RuleID
  result_groups: List<LabelGroup>
  picking_list: List<ItemPick>  // units per item over the batch, as the summary PDF prints them
  earliest_ship_by_date: Option<Date>
  latest_ship_by_date: Option<Date>
  couriers: List<CourierCode>
  warnings: List<String>
  errors: List<String>
  left_out: List<LeftOutCopy>   // as in the journal
  order_count: u16
  summary_pdf_path: Path        // output/{batch} summary.pdf
}

LabelGroup {
  code: String                  // LabelGroupRule.code
  output_path: Path             // output/{batch}-{code} {name} x{count}.pdf
  is_marked_as_printed: bool
  order_contents: List<OrderContentSummary>   // in printing order
}

OrderContentSummary {
  content_id: OrderContentID
  order_count: u16
}

Reprint {
  number: u8
  output_path: Path
  order_ids: List<String>
}
```

## `orders.csv`

One row per order written by a live batch; `order_id` is unique. The field names are the CSV headers;
an absent value is an empty cell.

```
Order {
  pdf_num: PdfID                // the input its written copy is in
  page_start: u16
  page_end: u16
  order_id: String
  tracking_id: String
  courier_code: CourierCode
  courier_clue: String          // logo, shape, text, form, tag or empty
  content_id: OrderContentID
  skus: String                  // its Seller SKUs in its content's item order, joined by "; "
  total_quantity: u16           // the printed Qty Total
  ship_by_date: Date
  pickup_code: String
  batch_id: BatchID
  group_code: String            // LabelGroup.code
}
```

## `order-contents.toml`

```
Root {
  schema: String                // "1.0"
  auto_increment: OrderContentID          // the next free ID
  order_content: Map<OrderContentID, OrderContent>
}
```

`OrderContent` and `ItemPick` are those of the journal ([records](records.md)). Two orders with the same
content share one ID, and the orders of one content ID in a group PDF are consecutive
([groups](../labels/groups.md#a-group-pdf)).
