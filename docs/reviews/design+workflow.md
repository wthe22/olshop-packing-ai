
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


## Data Schemas
- app.exe


### Data folder

Namespace represents an owner.


```
{namespace}/{date}/
{namespace}/{date}/
```

### `packing-app.toml`
Scope: user preferences

Schema:
```
AppPreferences {
  schema: String // Version
  data_dir: Path // Path to the data directory
  workflow_file: Path // Path to the workflow file
}
```
- Relative and absolute paths must be supported

### `packing-workflows.toml`
Scope: workflows

Schema:
```
PackingWorkflows {
  schema: String // version
  couriers: List<Courier>
  courier_detection: List<CourierDetection>
  workflows: List<Workflow>
}

Courier {
  code: String // Courier code, unique
  name: String // Display name
}

CourierDetection {
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



## First time setup
- Configure a folder to store the data
- For 
couriers: 

```



## First time setup
- Configure a folder to store the data
- For
