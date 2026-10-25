
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
the batch numbers run through that date of that namespace. There is no session name.

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
{namespace}/{date}/orders.csv
{namespace}/{date}/order-contents.toml
{namespace}/{date}/input/{some-random}.pdf
{namespace}/{date}/output/{batch_id}-{code} {name} x{count}.pdf
{namespace}/{date}/output/{batch_id} summary.pdf
```
- Default file names use no special symbols: only letters, digits, space, `-` and `_`. Any other character
  is replaced by a space and repeated spaces are collapsed. The owner may still rename a file.


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
  not_courier: NotCourier
  label_sort_rules: List<LabelSortRule>
}

CourierCode: Courier.code

Courier {
  code: String // Courier code, unique
  name: String // Display name
}

CourierClassifier {
  courier_code: CourierCode // unique
  logos: List<String> // Logo hash (XXH3-64)
  shapes: List<String> // Shape hash (XXH3-64)
  tracking_id: String // Tracking ID regex pattern
  text: List<String> // Any matching text
}

NotCourier {
  logos: List<String> // Logo hash (XXH3-64) of images that are not a courier, e.g. a channel strip. More fields can be added later
}

LabelSortRule {
  name: String // Unique
  label_groups: List<LabelGroupRule>
}

LabelGroupRule {
  code: String // unique per workflow
  name: String // unique per workflow
  filter: Option<String>
}
```

### `session.toml`
Scope: main working data

```
Root {
  schema: String // X.Y version, starts at 1.0
  namespace: String
  date: Date
  input_pdfs: Map<PdfID, LabelPdf>
  label_sort_rules: Map<RuleID, LabelSortRule>
  label_batches: List<LabelBatch>
}

PdfID: u8
RuleID: u8
BatchID: u8

LabelPdf {
  filename: String
  size: u64 // Size in bytes
  hash_sha256: String // Hexadecimal encoded
  create_time: Datetime // File creation time
  modify_time: Datetime // File modification time
  order_count: u16
  page_count: u16
  earliest_ship_by_date: Date
  latest_ship_by_date: Date
  couriers: List<CourierCode>
  warnings: List<String>
  errors: List<String>
}

LabelBatch {
  id: BatchID // 1, 2, 3, ... within the namespace and date
  create_time: Datetime
  modify_time: Datetime
  input_pdfs: List<PdfID>
  sort_rule_id: RuleID
  result_groups: List<LabelGroup>
  picking_list: List<ItemPick>
  earliest_ship_by_date: Date
  latest_ship_by_date: Date
  couriers: List<CourierCode>
  warnings: List<String>
  errors: List<String>
  order_count: u16
  summary_pdf_path: Path // Generated summary PDF path
}

LabelGroup {
  code: String // LabelGroupRule.code
  output_path: Path
  is_marked_as_printed: bool
  order_contents: List<OrderContentSummary>
}

OrderContentSummary {
  content_id: OrderContentID
  order_count: u16
}

OrderContentID: u16
```

### `orders.csv`
Scope: order info
Type: derived (written by the app from the PDFs; not rebuilt)
The field names are the CSV headers. `order_id` is unique: one row per order.

```
Order { 
  pdf_num: PdfID // PDF number (day wide)
  page_start: u16 // Starting page number
  page_end: u16 // Ending page number
  order_id: String
  tracking_id: String
  courier_code: CourierCode
  content_id: OrderContentID
  ship_by_date: Date
  batch_id: BatchID
  group_code: String // LabelGroup.code
}

```

### `order-contents.toml`
Scope: order content
Type: derived (written by the app from the PDFs; not rebuilt)

```
Root {
  schema: String // X.Y version, starts at 1.0
  auto_increment: OrderContentID
  order_content: Map<OrderContentID, OrderContent>
}

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
