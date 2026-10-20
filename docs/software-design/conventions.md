# Conventions

Apply to every design, document and code line of the PC app.

- Timestamps are named `*_time` (`create_time`, `save_time`) and stored as ISO 8601 with the UTC offset
  (`2026-10-06T14:26:00+07:00`).
- Items are identified only by the platform `SKU ID`. Product name, variation and Seller SKU are display
  values that can change between two exports of the same day.
- Orders are identified by `Order ID`.
- Requirement IDs (`F-LABEL-3`) are stable references for documents; code, comments and tests do not cite them
  (`tools/check-comments.mjs`).
- Requirements, decisions, questions and review items carry scoped IDs (`F-UI-1`, `D-APP-2`, `Q-UI-1`, `R-UI-8`); a number is
  never reused. The scopes are listed in [../README.md](../README.md#scopes).
- Where a design disagrees with [../business-process/](../business-process/README.md), the business
  process wins and the design is fixed.
