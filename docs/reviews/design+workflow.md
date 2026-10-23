
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

- At config 
- 



### Data folder

Namespace represents an owner.

Beside <app>.exe:
```
packing-settings.toml
packing-workflows.toml
```


Inside data directory:
```
{namespace}/{date}/session.toml
{namespace}/{date}/orders.toml
{namespace}/{date}/batch-{n}/batch.toml
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
SessionInfo {
  date: String
  pdfs: List<PdfInfo>
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
```

### `orders.toml`
Scope: order info

```
Order { 
  pdf: u16 // PDF number (day wide)
  order_id: String // Order ID
  tracking_id: String
  courier_code: String
  profile_id: String
  total_quantity: u16
  ship_by: Option<Datetime>
}
```

### `order-profiles.toml`
Scope: order info

```
OrderProfile { 
  id: String // Profile ID
  items: List<Item>
}

Item {
  name: String
  variant: String
  sku: String
  quantity: u16
}
```

### `batch.toml`
Scope: batch info

```
BatchInfo {
  create_time: Datetime
  modified_time: Option<Datetime>
}

```



## First time setup
- Configure a folder to store the data
- For 
couriers: 

```



## First time setup
- Configure a folder to store the data
- For
