# Software design

The target system: what the PC app does, the rules it applies and how it is made. Only the target:
no plans, no build tasks, no build status. The work still to do is in [../development/plan/](../development/plan/README.md);
how the business works without the software is in [../business-process/](../business-process/README.md).
Where a design file disagrees with the business process, the business process wins and the design is fixed.

| File or folder | Content |
|---|---|
| [requirements.md](requirements.md) | Concepts and the features of the app (`F-<SCOPE>-n`) |
| [conventions.md](conventions.md) | Naming and identification rules that apply to every design, document and code line |
| [labels/](labels/README.md) | Scope LABEL: reading labels and packing slips, courier deduction, picks, saved PDFs, the A4 packing list |
| [rules-file/](rules-file/README.md) | Scope RULES: the picks groupings (`categories.toml` and `categories/`) and the condition language |
| [app/](app/README.md) | Scope APP: one batch, the data folder, `state.json`, revert and amend, the check before saving, architecture, the portable program folder |
| [ui/](ui/README.md) | Scope UI: the window and every screen |
| [decisions/](decisions/README.md) | Settled choices of the owner, one record each (`D-<SCOPE>-n`) |
| [questions/](questions/README.md) | Open choices of the owner, each with options and a default (`Q-<SCOPE>-n`) |
