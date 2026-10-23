
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

Namespace represents an owner.

Beside <app>.exe:
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
  schema: String // Version
  data_dir: Path // Path to the data directory
  workflow_file: Path // Path to the workflow file
}
```
- Relative and absolute paths must be supported

### `packing-workflows.toml`
Scope: PDF processing

Schema:
```
PackingWorkflows {
  schema: String // version
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
  date: String
  pdfs: Map<PdfID, PdfInfo>
  pdf_auto_increment: PdfID
}

PdfInfo {
  filename: String
  size: u64 // Size in bytes
  hash_sha256: String // SHA-256 hash of the PDF
  orders: u16 // Number of orders in the PDF
  pages: u16 // Number of pages in the PDF
  earliest_ship_by: Datetime
  latest_ship_by: Datetime
}

PdfID: u16
```

### `orders.csv`
Scope: order info

```
Order { 
  pdf_num: u16 // PDF number (day wide)
  page_start: u16 // Starting page number
  page_end: u16 // Ending page number
  order_id: String
  tracking_id: String
  courier_code: String
  content_id: u16
  ship_by: Datetime
}

```

### `order-content.toml`
Scope: order content

```
Root {
  auto_increment: ContentID
  order_content: Map<ContentID, OrderContent>
}

ContentID: u16

OrderContent {
  items: List<ItemPick>
  total_quantity: u16
}

ItemPick {
  name: String
  variant: String
  sku: Option<String>
  quantity: u16
}
```

### `batch-summary.toml`
Scope: batch summary, packing list info

This file alone can be used to generate the batch packing list PDF.

```
Root {
  batch_no: u16 // Batch number
  create_time: Datetime
  modified_time: Option<Datetime> // For amends
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
