# Conventions

Apply to every design, document and code line of the PC app.

- Timestamps are named `*_time` (`create_time`, `modify_time`) and stored as ISO 8601 with the UTC offset
  (`2026-10-06T14:26:00+07:00`). Date and time types are `Date` (`2026-10-07`), `Time` (`14:26`) and
  `Datetime`; a `*_date` field is a `Date` (`ship_by_date`).
- Orders are identified by `Order ID`. Items are identified by product name and variation; the platform
  `SKU ID` is not on the label PDFs, and the Seller SKU is a field of the line ([D-APP-20](decisions/app-features.md)).
- Namespaces, dates and batches are identified by their names and numbers in the data folder
  ([data](data/README.md)); an input by its SHA-256; a courier by its code.
- Every TOML file and the journal's first record carry `schema` as an `X.Y` version, starting at `1.0`. The
  schema version is separate from the app version; data of an older schema is not read and not migrated
  ([D-APP-3](decisions/app.md)).
- A saved fact has one home, the journal ([D-APP-7](decisions/app.md)). Anything else that holds it is a view or
  a cache, rebuilt from the journal and never read for a decision.
- Requirement IDs (`F-LABEL-3`) are stable references for documents; code, comments and tests do not cite them
  (`tools/check-comments.mjs`).
- Requirements, decisions, questions and review items carry scoped IDs (`F-UI-1`, `D-APP-2`, `Q-UI-1`,
  `R-UI-8`); a number is never reused. The scopes are listed in [../README.md](../README.md#scopes).
- Where a design disagrees with [../business-process/](../business-process/README.md), the business process
  wins and the design is fixed.
