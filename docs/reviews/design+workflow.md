
# Workflow
## Operational Workflow
This warehouse handles orders from multiple owners

1. Shop owners send their shipping labels to the admin
2. The admin will handle the work for them

- An owner MAY have multiple shops.

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
{namespace}/{date}/session.toml
{namespace}/{date}/order-content.toml
{namespace}/{date}/orders.csv
{namespace}/{date}/batch-{n}/batch-summary.toml
```


## Data Schemas

### `packing-settings.toml`
Scope: settings / preferences

Schema:
```
AppSettings {
  schema: String // X.Y version, starts at 1.0
  data_dir: Path // Path to the data directory
}
```
- Relative and absolute paths must be supported

### `packing-workflows.toml`
Scope: PDF processing. Holds the courier list (merged from the former `couriers.toml`) and the groupings.

Schema:
```
PackingWorkflows {
  schema: String // X.Y version, starts at 1.0
  couriers: List<Courier>
  courier_deduction: List<CourierDeduction>
  workflows: List<Workflow>
}

Courier {
  code: String // Courier code, unique
  name: String // Display name
}

CourierDeduction {
  courier_code: String // Courier code, unique
  logos: List<String> // Logo hash (XXH3-64)
  shapes: List<String> // Shape hash (XXH3-64)
  tracking_id: String // Tracking ID regex pattern
  text: List<String> // Any matching text
}

Workflow {
  name: String
  sort_groups: List<SortGroup>
}

SortGroup {
  code: String // unique
  name: String
  match: String // Condition for matching
}
```

### `session.toml`
Scope: session state

```
Session {
  schema: String // X.Y version, starts at 1.0
  date: Date
  pdfs: Map<PdfID, PdfInfo>
  pdf_auto_increment: PdfID
  batch: List<PrintBatch>
}

PdfInfo {
  filename: String
  size: u64 // Size in bytes
  hash_sha256: String
  order_count: u16
  page_count: u16
  earliest_ship_by_time: Datetime
  latest_ship_by_time: Datetime
}

PdfID: u16

PrintBatch {
  batch_number: u16
  create_time: Datetime
  modified_time: Datetime
  pdfs: List<PdfID>
  workflow: Option<String> // Workflow name, if custom then none
}
```

### `orders.csv`
Scope: order info. Written by the app from the label PDFs; it is not an input.

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

### `order-content.toml`
Scope: order content

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

### `batch-summary.toml`
Scope: batch summary, packing list info

A cache. Every field is reproduced from the top-level files; the app may rewrite it at any time and
rebuilds it when it is missing.

This file alone can be used to generate the batch packing list PDF.

```
Root {
  schema: String // X.Y version, starts at 1.0
  batch_number: u16 // Batch number
  create_time: Datetime
  modified_time: Datetime // Same as the batch's modified_time in session.toml
  pdf_list: List<PdfInfo>
  print_groups: List<PrintGroup>
  picking_list: List<ItemPick>
  warnings: List<String>
  order_count: u16
}

PrintGroup {
  number: GroupNumber
  name: String
  contents: List<OrderContentSummary>
}

OrderContentSummary {
  items: List<ItemPick>
  order_count: u16
}

GroupNumber: u16
```



## First time setup
- Configure a folder to store the data
