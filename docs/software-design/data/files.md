# A date's files

`day.toml`, `orders.csv` and `order-contents.toml` are the truth of a date, for every account
([D-APP-39](../decisions/app.md)). Every decision (the already-saved guard, the next batch number, what may be
reverted or amended) reads them as they are on disk. They hold what is true now: a reverted batch is gone
from them, an amended one is replaced, and no history of either is kept. Every order traces to its source,
the pages of a stored input it was read from (`pdf_num`, `page_start`, `page_end`).

How they are written together is in [save](save.md). Each is written whole, to `.partial/` first; a key or a
column added by hand is not kept. `day.toml` holds the SHA-256 of `orders.csv` and `order-contents.toml`, and its
last line, a TOML comment `# sha256: <hex>`, the SHA-256 of the text before it.

## `day.toml`

```
Root {
  schema: String                // "2.1"
  date: Date                    // equals the folder's name
  orders_csv_sha256: Sha256     // of orders.csv as written
  order_contents_sha256: Sha256 // of order-contents.toml as written
  input_pdfs: Map<PdfID, LabelPdf>               // every input stored in the date folder, live or not
  label_sort_rules: Map<RuleID, LabelSortRule>   // each rule a live batch used, as it was (rules-file/README.md)
  label_batches: List<LabelBatch>                // the live batches
  reprints: List<Reprint>
}                               // then the line # sha256: <hex> of the text before it

LabelPdf {
  filename: String              // the name it had when it was chosen
  size: u64
  hash_sha256: String           // the file is {batch} Input {pdf} {first 12}.pdf
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
  create_time: Datetime         // its first save
  modify_time: Datetime         // its newest save (an amend)
  input_pdfs: List<PdfID>       // reading order
  sort_rule_id: RuleID
  result_groups: List<LabelGroup>
  picking_list: List<ItemPick>  // units per item over the batch, as the summary PDF prints them
  earliest_ship_by_date: Option<Date>
  latest_ship_by_date: Option<Date>
  couriers: List<CourierCode>
  warnings: List<String>
  errors: List<String>          // the verifier's, only on a batch saved anyway
  left_out: List<LeftOutCopy>
  order_count: u16
  summary_pdf_path: Path        // {batch} Summary.pdf, in the date folder
}

LabelGroup {
  code: String                  // LabelGroupRule.code
  output_path: Path             // {batch}-{code} {name} x{count}.pdf, in the date folder
  press_print_time: Option<Datetime>  // when Print last opened its preview; none: not since the save, or taken back
  is_printed: bool              // the owner's tick
  order_contents: List<OrderContentSummary>   // in printing order
}

OrderContentSummary { content_id: OrderContentID, order_count: u16 }

LeftOutCopy {                   // a copy of an order not written
  order_id: String
  pdf: PdfID
  page_start: u16
  page_end: u16
  reason: LeftOutReason         // repeat | saved_earlier
  kept: Kept                    // repeat: { pdf, page_start } of the copy written; saved_earlier: { batch }
}

Reprint {
  number: u8                    // 1, 2, … within the date
  output_path: Path             // Reprint {n} x{count}.pdf
  order_ids: List<String>       // in printing order
}
```

`PdfID`, `RuleID` and `OrderContentID` are never given twice within a date: the next one is one more than
the highest held. A sort rule or a content no live batch names is left out at the next write.

## `orders.csv`

One row per order of a live batch, in printing order; `order_id` is unique. The field names are the CSV
headers; an absent value is an empty cell. The last column's header is `schema:2.1` and its cells are
empty: a reader finds a column by its header and passes over that one; a new column goes before it
([D-APP-35](../decisions/versions.md)).

```
Order {
  pdf_num: PdfID                // the input its written copy is in
  page_start: u16
  page_end: u16
  order_id: String
  tracking_id: String           // an instant label's pickup code when it prints no tracking ID (D-LABEL-24)
  courier_code: CourierCode
  shop: String                  // empty: no sender on the label
  province: String              // the receiver's area as printed (reading.md); empty: not printed or not read
  city: String
  district: String
  weight_grams: u32             // empty: not printed or not read
  content_id: OrderContentID
  total_quantity: u16           // the printed Qty Total; empty on the Shopee-style slip
  ship_by_date: Date
  batch_id: BatchID
  group_code: String            // LabelGroup.code
}
```

## `order-contents.toml`

```
Root {
  schema: String                // "1.1"
  auto_increment: OrderContentID          // the next free ID
  order_content: Map<OrderContentID, OrderContent>
}

OrderContent {                  // what an order is packed with (D-LABEL-15)
  items: List<ItemPick>         // sorted by name, variant, quantity
}

ItemPick {
  name: String
  variant: String               // empty for `Default` or none
  quantity: u16
  sku: Option<String>           // the item's Seller SKU on the slip; none: the slip prints none
}
```

Two orders with the same content share one ID, and the orders of one content ID in a group PDF are
consecutive ([groups](../labels/groups.md#a-group-pdf)).

## The summaries made on request

`Day Summary.pdf`, `Day Summary - {account}.pdf` and `{batch} Summary - {account}.pdf` are written only by
**PDF** on the [Summary](../ui/summary.md) screen. Each says what the files said when it
was made ([day summary](../labels/day-summary.md)); a revert or an amend deletes the batch's account summaries.
