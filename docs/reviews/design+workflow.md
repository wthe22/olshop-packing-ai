
# Workflow
## Operational Workflow
This warehouse handles orders from multiple owners

1. Shop owners send their shipping labels to the admin
2. The admin will handle the work for them

- An owner MAY have multiple shops (and multiple platforms).

## Label Preparation Workflow

1. Download all the PDFs of the shipping labels
2. Print the shipping labels
3. Sort shipping labels by packing list
4. Give the sorted shipping labels to the packing teams

- Shipping labels can come from multiple sources/owners
- The admin needs to keep track of which shipping labels are already printed to prevent double-printing
- Sometimes a label may need re-printing due to damage or labeling mistake

## Packing Workflow
- Each packing team handles items of a specific category
- Packing lists that appears often are pre-packed

1. Pack all the items in the packing list
2. Stick the label to the package
3. Put the package in the sorting bin (sorted by courier)

# Software Workflow
This app will be used as a portable app.

## Data folder

A namespace represents an owner. It is the top folder of the data: one folder per date sits inside it, and
the saved-PDF and batch numbers run through that date of that namespace. There is no session name.

Types: `Date` is `2026-10-07`, `Time` is `14:26`, `Datetime` is ISO 8601 with the UTC offset
(`2026-10-07T14:26:00+07:00`).

Versions: every file carries `schema` as an `X.Y` version, starting at `1.0`. The schema version is not the
app version. The app version of this build is `v0.2`, and a schema change raises it. There is no migration:
data written by an older version is thrown away.

Beside `<app>.exe`:
```
packing-settings.toml
packing-workflows.toml
```


Inside data directory:
```
{namespace}/{date}/session-replay.toml
{namespace}/{date}/order-content.toml
{namespace}/{date}/orders.csv
{namespace}/{date}/session.toml
{namespace}/{date}/input/{some-random}.pdf
{namespace}/{date}/output/{batch_number}-{code} {name} x{count}.pdf
```


## Data Schemas

### `packing-settings.toml`
Scope: settings / preferences

Schema:
```
Root {
  schema: String // X.Y version, starts at 1.0
  data_dir: Path // Path to the data directory
}
```
- Relative and absolute paths must be supported

### `packing-workflows.toml`
Scope: global PDF processing infos
Type: Input

Holds the courier list (merged from the former `couriers.toml`) and the groupings.

Schema:
```
Root {
  schema: String // X.Y version, starts at 1.0
  couriers: List<Courier>
  courier_classifier: List<CourierClassifier>
  pdf_sort_rules: List<PdfSortRules>
}

Courier {
  code: String // Courier code, unique
  name: String // Display name
}

CourierClassifier {
  courier_code: String // Courier code, unique
  logos: List<String> // Logo hash (XXH3-64)
  shapes: List<String> // Shape hash (XXH3-64)
  tracking_id: String // Tracking ID regex pattern
  text: List<String> // Any matching text
}

PdfSortRules {
  name: String // Unique
  groups: List<PdfGrouping>
}

PdfGroupRule {
  code: String // unique per workflow
  name: String // unique per workflow
  filter: Option<String>
}
```

### `session-replay.toml`
Scope: session inputs

E2E input. All outputs can be derived from this.

```
Session {
  schema: String // X.Y version, starts at 1.0
  input_pdfs: Map<PdfID, PdfFile>
  sort_workflows: Map<WorkflowID, SortWorkflow>
  sort_batches: List<SortBatch>
}

PdfID: u16 // day-wide, auto-incrementing
WorkflowID: u16 // day-wide, auto-incrementing

PdfFile {
  file_path: String
  size: u64 // Size in bytes
  hash_sha256: String // Hexadecimal encoded
}

SortBatch {
  batch_number: u16 // day-wide
  create_time: Datetime
  modified_time: Datetime
  input_pdfs: List<PdfID>
  workflow_used: WorkflowID
  sort_group: List<SortGroup>
}

SortGroup {
  code: String // PrintGrouping.code
  output_filename: String
  is_marked_as_printed: bool
}
```

### `session.toml`
Scope: main working data

```
Root {
  schema: String // X.Y version, starts at 1.0
  namespace: String
  date: Date
  pdfs: Map<PdfID, PdfInfo>
  batch_info: List<BatchInfo>
  day_info: BatchInfo
}

Courier: Courier.code

PdfInfo {
  filename: String
  size: u64 // Size in bytes
  hash_sha256: String // Hexadecimal encoded
  create_time: Datetime // File creation time
  modify_time: Datetime // File modification time
  order_count: u16
  page_count: u16
  earliest_ship_by_date: Date
  earliest_ship_by_time: Option<Time>
  latest_ship_by_date: Date
  latest_ship_by_time: Option<Time>
  couriers: List<Courier>
}

BatchInfo {
  create_time: Datetime
  modify_time: Datetime
  input_pdfs: List<PdfID>
  sort_groups: List<SortGroupSummary>
  picking_list: List<ItemPick>
  earliest_ship_by_date: Date
  earliest_ship_by_time: Option<Time>
  latest_ship_by_date: Date
  latest_ship_by_time: Option<Time>
  couriers: List<Courier>
  warnings: List<String>
  order_count: u16
}

PickSummary {
  print_groups: List<PrintGroupSummary>
  picking_list: List<ItemPick>
  earliest_ship_by_date: Date
  earliest_ship_by_time: Option<Time>
  latest_ship_by_date: Date
  latest_ship_by_time: Option<Time>
  couriers: List<Courier>
  warnings: List<String>
  order_count: u16
}

OrderGroupSummary {
  code: String // PrintGrouping.code
  name: String // PrintGrouping.name
  contents: List<OrderContentSummary>
}

OrderContentSummary {
  items: List<ItemPick>
  order_count: u16
}

GroupNumber: u16
```

### `orders.csv`
Scope: order info
Type: cache (from parsing PDFs)

```
Order { 
  pdf_num: u16 // PDF number (day wide)
  page_start: u16 // Starting page number
  page_end: u16 // Ending page number
  order_id: String
  tracking_id: String
  courier_code: String
  content_id: u16
  ship_by_time: Datetime
}

```

### `order-contents.toml`
Scope: order content
Type: cache (from parsing PDFs)

```
Root {
  schema: String // X.Y version, starts at 1.0
  auto_increment: ContentID
  order_content: Map<ContentID, OrderContent>
}

ContentID: u16

OrderContent {
  items: List<ItemPick>
  total_quantity: Option<u16> // Qty Total printed on the slip; none on the Shopee-style slip. A check against the lines, not a sum
}

ItemPick {
  name: String
  variant: String
  sku: Option<String> // Seller SKU; the platform SKU ID is never on the PDFs
  quantity: u16
}
```

## First time setup
- Configure a folder to store the data
