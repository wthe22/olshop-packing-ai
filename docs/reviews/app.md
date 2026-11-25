# APP: the owner's feedback

The owner's findings on batches, the data folder, revert and amend, and the program folder. Open items
only; see [README](README.md).

## R-APP-2 — A full workflow of the program, to verify

**Present.** The steps are written in one place: [the program's workflow](../software-design/workflow/README.md),
every step with its algorithm, the invariants I1–I12 it keeps and the verification matrix.

**Wanted.** The whole workflow in one place — steps, processing logic and algorithms — so the owner can verify
that they are correct.

**Waits for the owner's reading.** A step the owner finds wrong is written here as an item, one each; the
points R1 had to settle are listed in [the plan's status](../development/plan/README.md).

## R-APP-16 — Strict read-only mode, and migrations that ask, with progress

**Present.** Opening a date writes without asking: the views of an older version are written again, a
stopped save is settled. A long step shows little of what it does.

**Wanted.** At its start, when the data folder needs a migration, the app asks: what would change, with
**Update** and **Open read-only** (the only way into read-only). Read-only writes nothing at all. Update
shows its progress, step by step, so the program never looks frozen. The rule is
[D-APP-33](../software-design/decisions/app.md); this item builds it, with the migrations of R-LABEL-17
and the views' `schema` 2.1.

