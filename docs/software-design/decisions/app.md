# Decisions: app

## D-APP-1 — PDFium does all three PDF jobs

**Decision.** One PDF library, PDFium, reads the text of a page with its positions, copies pages
unchanged into a saved PDF, and writes the A4 packing list.

**Why.** On the sample label downloads it read the Order ID on every page and gave the same slip
lines as an independent reading of the same files; copied pages render with no pixel difference;
reading 601 pages takes 1.6 s. The packing-list files are larger (about 1.4 MB for one page, 1.6 MB
for five) because PDFium writes whole fonts; they print the same, and the size is accepted.

**Applies to.** [architecture.md](../app/architecture.md), [packing-list.md](../labels/packing-list.md).

## D-APP-2 — The check before saving runs on every save

**Decision.** The saved PDFs and the packing lists are read back and compared with the plan on
every save, with no size condition.

**Why.** A wrong page order or a doubled page is otherwise found only at the printer; the check
costs 1–2 s on a large day.

**Applies to.** [saved-pdf-check.md](../app/saved-pdf-check.md), [batch.md](../app/batch.md).

## D-APP-3 — Data carries an X.Y schema version; no migration

**Decision.** Every data file carries `schema` as an `X.Y` version (the v0.2 schema is `0.2`). A change of
schema is a new app version (this schema ships as app `v0.2`). Data written by an older version is thrown
away; the app does not migrate it.

**Why.** The owner's choice: the data is a working set, and the labels can be read again.

**Applies to.** [conventions.md](../conventions.md); the owner's draft ([reviews](../../reviews/design+workflow.md)).

## D-APP-4 — Order data comes from the label PDFs only

**Decision.** The platform's order export is not read, by the app or as an input of any rule. Every order
field comes from the label and its packing slip. `orders.csv` in the data folder is written by the app
from the PDFs; it is an output.

**Why.** The owner's choice: the export is not part of this project any more.

**Applies to.** [requirements.md](../requirements.md) (F-APP-2), [app/README.md](../app/README.md), [rules-file/conditions.md](../rules-file/conditions.md).

## D-APP-5 — A namespace replaces the session

**Decision.** The session (a day plus an optional name) is replaced by the namespace: one namespace per
owner, one folder per date inside it. Saved-PDF and batch numbers run through a namespace and date. There is
no session name and no *Main*.

**Why.** The owner's choice: the session concept does not suit this workflow.

**Applies to.** The owner's draft ([reviews](../../reviews/design+workflow.md)). The design files that still
say session are listed in [plan/README.md](../../development/plan/README.md) until they are updated.

## D-APP-6 — The courier list is part of packing-workflows.toml

**Decision.** The courier list is merged into `packing-workflows.toml`; there is no separate `couriers.toml`.
Where that file lives is [Q-APP-11](../questions/app.md). This replaces the file of [D-LABEL-1](label.md).

**Why.** The owner's choice: a separate courier file has no purpose once the workflows file exists.

**Applies to.** The owner's draft; the design files that name `couriers.toml` are listed in [plan/README.md](../../development/plan/README.md).

