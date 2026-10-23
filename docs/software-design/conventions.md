# Conventions

Apply to every design, document and code line of the PC app.

- Timestamps are named `*_time` (`create_time`, `save_time`) and stored as ISO 8601 with the UTC offset
  (`2026-10-06T14:26:00+07:00`).
- Items are identified by product name and variation (the packing key). The platform `SKU ID` is not on the
  label PDFs and is not used; the Seller SKU is a field of the item ([Q-APP-8](questions/app.md)).
- Orders are identified by `Order ID`.
- Date and time fields use the types `Date` (`2026-10-07`), `Time` (`14:26`) and `Datetime` (with the UTC offset).
  A `*_time` field is a `Datetime`.
- Every data file carries `schema` as an `X.Y` version, starting at `1.0`. The schema version is separate from the
  app version; a schema change raises the app version, and there is no migration, so older data is thrown away ([D-APP-3](decisions/app.md)).
- Requirement IDs (`F-LABEL-3`) are stable references for documents; code, comments and tests do not cite them
  (`tools/check-comments.mjs`).
- Requirements, decisions, questions and review items carry scoped IDs (`F-UI-1`, `D-APP-2`, `Q-UI-1`, `R-UI-8`); a number is
  never reused. The scopes are listed in [../README.md](../README.md#scopes).
- Where a design disagrees with [../business-process/](../business-process/README.md), the business
  process wins and the design is fixed.
