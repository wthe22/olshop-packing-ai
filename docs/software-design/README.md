# Software design

The target system: what the PC app does, the rules it applies and how it is made. Only the target: no plans,
no build tasks, no build status. The work still to do is in [../development/plan/](../development/plan/README.md);
how the business works without the software is in [../business-process/](../business-process/README.md).
Where a design file disagrees with the business process, the business process wins and the design is fixed.

| File or folder | Content |
|---|---|
| [workflow/](workflow/README.md) | **Start here to check the system**: every step in order, its algorithm, the invariants it keeps, and the verification matrix |
| [requirements.md](requirements.md) | Concepts and the features of the app (`F-<SCOPE>-n`) |
| [conventions.md](conventions.md) | Naming and identification rules for every design, document and code line |
| [data/](data/README.md) | The data folder, the journal and its records, the views |
| [app/](app/README.md) | Scope APP: architecture, commands and start, namespaces, repeated orders, the verifier, search and reprint |
| [labels/](labels/README.md) | Scope LABEL: reading labels and slips, couriers, label groups and runs, the summary PDF |
| [rules-file/](rules-file/README.md) | Scope RULES: `packing-workflows.toml` and the filter language |
| [ui/](ui/README.md) | Scope UI: the window and every screen |
| [decisions/](decisions/README.md) | Settled choices of the owner, one record each (`D-<SCOPE>-n`) |
| [questions/](questions/README.md) | Open choices of the owner, each with options and a default (`Q-<SCOPE>-n`) |
