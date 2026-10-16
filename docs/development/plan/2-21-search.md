# 2.21 Search: find a saved order by tracking ID, item, quantity and more

The Search panel, what a result row shows, its scope and how it reuses the condition boxes of
*Categories* are in [search.md](../../software-design/ui/search.md) (the owner's request).

- **Data:** each saved order keeps the searched fields in `state.json`'s `saved[]`
  ([data-folder.md](../../software-design/app/data-folder.md)), written at save. A day saved without
  them stays searchable by its IDs only (*items not stored*).
- **Code:** the matching reuses the condition code of `categories.toml` (engine, no PDF); the search
  command and the panel sit where the Categories editor's do
  [categories.md](../../software-design/ui/categories.md),
  [architecture.md](../../software-design/app/architecture.md),
  [program.md](../../software-design/app/program.md)).
- **Tests:** a tracking ID finds its one order; an item field matches when one line matches; `not`
  and *at least* behave as in a category; *This day* does not see another day's order; an order of an
  open draft is not found; an order saved before the change shows *items not stored*.
- **Result:** (not started)
