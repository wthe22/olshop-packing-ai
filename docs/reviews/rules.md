# RULES: the owner's feedback

The owner's findings on `categories.toml` and the condition language. Open items only; see [README](README.md).

## R-RULES-1 — Several sets of categories

**Now:** the app reads one `categories.toml` in the data folder, so there is one set of picks
([rules file](../software-design/rules-file/README.md)).

**Wanted:** the owner can define several named sets of categories, and use the set that fits. Which set a
batch uses, and where the sets are kept, are open choices: the next session puts them into
[software-design/questions/](../software-design/questions/README.md) with options and a default.
