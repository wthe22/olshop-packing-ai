# The rules file: `packing-workflows.toml`

One file beside the program holds the owner's rules: the couriers, the clues that name them, the images
that are not couriers, and the **sort rules** ([D-APP-13](../decisions/app-features.md)). It is written from the copy
bundled in the program when it is missing, and from then on it is the owner's. The app edits it from the
Sort rules screen and the logo prompt, keeping comments and order. The filter language is in
[conditions.md](conditions.md).

## Schema

```
Root {
  schema: String // X.Y version, starts at 1.0
  couriers: List<Courier>
  courier_classifier: List<CourierClassifier>
  not_courier: NotCourier
  label_sort_rules: List<LabelSortRule>
}

Courier {
  code: String // unique: lower-case letters, digits and -
  name: String // display name
}

CourierClassifier {
  courier_code: CourierCode // unique
  logos: List<String> // logo hash (XXH3-64 of the image's encoded bytes, 16 hex)
  shapes: List<String> // shape hash (XXH3-64 of the 64 x 64 appearance)
  tracking_id: Option<String> // tracking-ID regex, matched fully; none for a courier without one
  text: List<String> // printed text that stands in for the logo
  instant_tags: List<String> // service tags of an instant-service label (GKS)
}

NotCourier {
  logos: List<String> // hashes of header images that are not a courier, e.g. a channel strip
}

LabelSortRule {
  name: String // unique
  enabled: bool // default true; false: kept, not offered on New batch; at least one rule is on
  label_groups: List<LabelGroupRule>
}

LabelGroupRule {
  code: String // 1-3 letters or digits, unique in the rule
  name: String // unique in the rule
  filter: Option<String> // none: takes every order left; only the last group
}
```

## Couriers

```toml
schema = "1.0"

[[couriers]]
code = "jnt"
name = "J&T Express"

[[courier_classifier]]
courier_code = "jnt"
logos = ["39b9b7b9f61d0e7f"]     # 975x377 px, the same asset in every sample file
tracking_id = 'J[XY]\d{10}'
text = ["jet.co.id"]

[[courier_classifier]]
courier_code = "jnt-cargo"
shapes = ["753ccbd14749befa"]    # masked wordmark: its bytes name nothing
tracking_id = '57\d{10}'
text = ["FastTrack"]

[[courier_classifier]]
courier_code = "gojek"
instant_tags = ["GKS"]

[not_courier]
logos = []
```

The bundled copy holds J&T Express (`jnt`), SiCepat (`sicepat`), J&T Cargo (`jnt-cargo`), IDX (`idx`), SPX
(`spx`), Anteraja (`anteraja`), Gojek (`gojek`, tag `GKS`) and Grab (`grab`, tag `Sameday`) with the hashes,
forms and tags of [courier deduction](../labels/courier.md), and the buying channels' strips in
`not_courier`. A list field left out is empty.

## Sort rules

```toml
[[label_sort_rules]]
name = "default"

[[label_sort_rules.label_groups]]
code = "A"
name = "Sepatu"
filter = 'name contains "sepatu"'

[[label_sort_rules.label_groups]]
code = "B"
name = "Spion & Knalpot"
filter = '''
not name contains "sepatu"
and (name contains "spion" or name contains "knalpot")
'''

[[label_sort_rules.label_groups]]
code = "Z"
name = "Lainnya"

[[label_sort_rules]]
name = "promo 11.11"
enabled = false

[[label_sort_rules.label_groups]]
code = "Z"
name = "Lainnya"
```

- A sort rule's groups apply in file order; an order goes to the **first** group whose filter matches
  ([groups](../labels/groups.md)). `'…'` is a TOML literal string and `'''…'''` spans lines, so the
  filter's double quotes need no escaping.
- The bundled copy holds one rule, `default`, with one group `Z Lainnya` and no filter.
- **Which rule a batch uses** is chosen on [New batch](../ui/new-batch.md#the-sort-rule) among the rules
  that are on; the rule of the workspace's newest batch, on whichever date, is preselected
  ([D-RULES-1](../decisions/rules.md)).
- **A rule that is off** (`enabled = false`) is kept with its groups and is not offered on New batch; an
  amend of a batch saved with it still uses it. The Sort rules screen switches it; on is written as no key
  ([D-RULES-5](../decisions/rules.md)).
- **A saved batch keeps a snapshot** of its rule in its journal record, so its outputs stay traceable
  whatever the file says later. An **amend** uses the rule of the same name as it is now, so a corrected
  filter applies; when no rule has that name any more, it uses the snapshot ([D-RULES-2](../decisions/rules.md)).
- **Rename** changes the name here only; old batches keep theirs in the journal.

## What is in code, not in this file

The label formats — the Order ID captions, the slip's column headers, the page parts — are the platforms'
facts, not the owner's choices: a new platform needs new reading code and tests anyway. They stay in
`packing-engine` ([reading](../labels/reading.md)).

## Checks

Unknown key; a missing required field; another schema than `1.0`; a courier code that is not lower-case
letters, digits and `-`; a duplicate courier code, classifier, rule name, or group code or name within a
rule; a group code that is not 1–3 letters or digits; a classifier, a `courier equals` value or a logo
answer naming no courier; a hash that is not 16 lower-case hex characters; a tracking-ID regex that does
not compile; no sort rule, or a rule with no group; every rule off; a group with no `filter` that is
not last; a filter error. Each names its line and column of this file. A file that does not parse stops the
start; a check that fails names the rule and opens the Sort rules screen on it.
