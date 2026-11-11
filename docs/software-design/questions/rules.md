# Open questions: RULES

Each question gives the problem, the options, and the **Default**: what the work does until the owner decides.
Raised 2026-11-11 with [v0.5](../../development/plan/v0.5-accounts.md).

## Q-RULES-3 — Account and shop in the filter language

Search must filter by a shop or an account; the filter language is the one language of Search and of the sort
rules ([conditions](../rules-file/conditions.md#fields)).

- **A. Two fields of the filter language**: `account` (equals, contains) and `shop` (contains), usable in
  Search and in a sort rule, so a sort rule may also take one account's orders. *(Default)*
- **B. Search only.** Search gets its own account and shop choices; the sort rules never see them.
