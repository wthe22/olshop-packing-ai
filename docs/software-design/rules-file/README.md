# The rules files: `parser.toml` and `config.toml`

Two files in the data folder hold the owner's rules ([D-APP-30](../decisions/app-features.md),
[D-APP-30](../decisions/app-features.md)); the app's own preferences are a third file beside the program
([data](../data/README.md#the-files)).

| File | Holds | Edited by |
|---|---|---|
| [`parser.toml`](parser.md) | How labels are read: the couriers, the clues that name them, the header images that are not couriers | The logo prompt, the owner |
| [`config.toml`](config.md) | The accounts and the **sort rules** | The Accounts screen, the shop prompt, the Sort rules screen, the owner |

Each is written from the copy bundled in the program when it is missing, and from then on it is the
owner's. The app edits them keeping comments and order. The filter language is in
[conditions.md](conditions.md).

## Why two files

- A sort rule's `courier equals "jnt"` names a courier of `parser.toml`; the other direction never
  exists. `parser.toml` changes when a new logo is named; `config.toml` when the owner changes how the
  day is sorted, or a shop is listed. A broken edit of one never touches the other.
- Both are in the **data folder**, so a data folder moved to another PC, or shared by several PCs, carries
  its accounts, its sort rules and what the app has learned of labels with it. Switching the data folder
  in Settings switches them too; a folder without them is given the bundled copies.

## Loading

Both files are read together, so the checks across them hold. Each check names its **file**, line and
column; a file that does not parse stops the start.

Unknown key; a missing required field; another schema than `1.0`; a duplicate courier code,
classifier, account, rule name, or group code or name within a rule; a courier code that is not
lower-case letters, digits and `-`; a group code that is not 1–3 letters or digits; a classifier, a
`courier equals` value or a logo answer naming no courier; a hash that is not 16 lower-case hex
characters; a tracking-ID regex that does not compile; an account or shop name that
[the Accounts screen would refuse](../app/accounts.md#names); a rule with no group;
every rule off; a group with no `filter` that is not last; a filter error. A check that fails in
`config.toml` names the rule and opens the Sort rules screen on it.

**One write, one lock.** Every write of either file reads it again first and holds the data folder's lock
([data](../data/README.md#one-writer)), so another copy's change to the accounts or a rule is kept. A
write is made through a temporary file renamed into place.

## What is in code, not in these files

The label formats — the Order ID captions, the slip's column headers, the page parts — are the platforms'
facts, not the owner's choices: a new platform needs new reading code and tests anyway. They stay in
`packing-engine` ([reading](../labels/reading.md)).
