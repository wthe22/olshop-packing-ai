# Views

Three files of the owner's draft, written from the journal after every commit so a person or another
program can read a date without the app ([D-APP-8](../decisions/app.md)). Each is written to a temporary
file and renamed into place; `day.toml` is written last, so its `journal_seq` vouches for all three.
A view is never read for a decision. When one is missing or `day.toml`'s `journal_seq` is not the
journal's last `seq`, all three are written again ([journal › When a date opens](journal.md#when-a-date-opens));
when its `schema` is older than this version's, or the date still has v0.7's `session.toml`, only by the
migration the owner starts ([D-APP-33](../decisions/app.md)), which first copies them into the date's
update folder and then deletes `session.toml` ([D-APP-36](../decisions/versions.md)). An older copy writes no
view of a date whose journal holds a record it passed over, nor over a newer `day.toml` or `orders.csv`
([D-APP-34](../decisions/versions.md)). A view is the program's: each commit writes it whole, so a column or
a key added by hand is not kept; the owner edits a copy.

Only live batches appear; a reverted batch is gone from the views and stays in the journal. IDs that
the journal does not hold (`RuleID`, `OrderContentID`) are numbered by the fold in order of first
appearance, so the same journal always gives the same views.

The summaries made on request — `Day Summary.pdf`, `Day Summary - {account}.pdf` and `{batch} Summary -
{account}.pdf` — are views too, written only by **Day summary** and a batch card's account buttons on the
Day screen, and never checked for staleness: each says what the journal said when it was made
([day summary](../labels/day-summary.md), [summary PDF](../labels/summary-pdf.md#one-account)). A revert or
an amend deletes the batch's account summaries after its commit.

## `day.toml`

```
Root {
  schema: String                // "2.1": this file's own version (orders.csv has its own)
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
  accounts: List<AccountOrders> // the batch's accounts, most orders first
  warnings: List<String>
  errors: List<String>
  left_out: List<LeftOutCopy>   // as in the journal
  order_count: u16
  summary_pdf_path: Path        // {batch} Summary.pdf, in the date folder
}

LabelGroup {
  code: String                  // LabelGroupRule.code
  output_path: Path             // {batch}-{code} {name} x{count}.pdf, in the date folder
  is_marked_as_sent: bool
  is_marked_as_printed: bool
  order_contents: List<OrderContentSummary>   // in printing order
}

AccountOrders {
  account: String
  order_count: u16
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
an absent value is an empty cell. The last column's header is `schema:2.1`, its own version (`day.toml` has its own),
and its cells are empty: a reader finds a column by its header and passes over that one; a new column goes
before it ([D-APP-35](../decisions/versions.md)).

```
Order {
  pdf_num: PdfID                // the input its written copy is in
  page_start: u16
  page_end: u16
  order_id: String
  tracking_id: String
  courier_code: CourierCode
  shop: String                  // empty: no sender on the label
  province: String              // the receiver's area as printed (records.md, Area); empty: not printed or not read
  city: String
  district: String
  weight_grams: u32             // records.md, Weight; empty: not printed or not read
  content_id: OrderContentID
  total_quantity: u16           // the printed Qty Total
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
```

`OrderContent` and `ItemPick` are those of the journal ([records](records.md)). Two orders with the same
content share one ID (a content is its items with their SKUs), and the orders of one content ID in a group PDF are consecutive
([groups](../labels/groups.md#a-group-pdf)).
