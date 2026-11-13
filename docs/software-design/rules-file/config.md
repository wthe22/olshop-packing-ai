# The config file: `config.toml`

The accounts and the sort rules ([D-APP-30](../decisions/app-features.md),
[D-APP-27](../decisions/accounts.md)). In the data folder; see [the two files](README.md) for how it is
written, edited and checked. Its courier filters name the couriers of [`parser.toml`](parser.md).

## Schema

```
Root {
  schema: String // X.Y version, starts at 1.0
  accounts: List<Account> // in the order they were made; missing: none
  label_sort_rules: List<LabelSortRule>
}

Account {
  name: String // 1–40 characters, a file name; unique, case ignored (accounts.md, Names)
  shops: List<String> // each in one account only
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

## Accounts

```toml
[[accounts]]
name = "Toko Alfa"
shops = ["Toko Alfa", "Toko Alfa Official"]
```

The accounts screen and the shop prompt write them ([accounts](../app/accounts.md)); they are written
before the sort rules. The bundled copy lists none.

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
  that are on; the rule of the data folder's newest batch, on whichever date, is preselected
  ([D-RULES-1](../decisions/rules.md)).
- **A rule that is off** (`enabled = false`) is kept with its groups and is not offered on New batch; an
  amend of a batch saved with it still uses it. The Sort rules screen switches it; on is written as no key
  ([D-RULES-5](../decisions/rules.md)).
- **A saved batch keeps a snapshot** of its rule in its journal record, so its outputs stay traceable
  whatever the file says later. An **amend** uses the rule of the same name as it is now, so a corrected
  filter applies; when no rule has that name any more, it uses the snapshot ([D-RULES-2](../decisions/rules.md)).
- **Rename** changes the name here only; old batches keep theirs in the journal.
