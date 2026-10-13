# Phase 4 — App core (no camera)

Read: [01-requirements](../../design/01-requirements.md) A1, A3, A4, A6; [05-app-architecture](../../design/05-app-architecture.md) (database, import);
[06-app-ui](../../design/06-app-ui.md) (sessions, orders, import summary).

- [ ] **4.1** `core`: CSV import, display names, signatures, condition language, TOML
  read/write, batch diff, group numbering, status from events. JVM tests on `testdata/` must
  match `expected.json` and `conditions.json`; local test on `samples/` must match
  `samples/expected-06.json`.
- [ ] **4.2** Room database (tables of 05-app-architecture), repositories.
- [ ] **4.3** Sessions screen: create, rename, open, delete with confirmation.
- [ ] **4.4** Import CSV with summary and Apply / Cancel.
- [ ] **4.5** Orders list + detail, filters, search, grouped view.
- [ ] **4.6** Manual marks and undo; history screen.
- [ ] **4.7** Strings in English and Indonesian from the start (`values/`, `values-in/`).

**Owner review:** [review-1.md](review-1.md).
