# RULES: the owner's feedback

The owner's findings on the picks sets and the condition language. Open items only; see [README](README.md).

## R-RULES-2 — A set of categories cannot be renamed

**Present.** The Categories screen creates a set of categories (**+ New set**) and deletes it (**Delete set**).
Its name cannot be changed.

**Wanted.** A rename control on the Categories screen. The new name shows on New batch and in the Plan
header (as *Grouping*, see R-UI-4).

**Open point for the design (not yet decided).** A batch records its set by name ([batch](../software-design/app/batch.md)),
and a set's file name is its name ([Q-RULES-1](../software-design/questions/rules.md), default (b)). A rename
must say which name an amend of an older batch reads: the new name, or the name the batch recorded. This goes
to the owner as a question with options and a default before the rename is built.
