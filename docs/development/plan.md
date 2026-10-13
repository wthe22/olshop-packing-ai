# Build plan

Phases in order, for both tools. Each phase has a folder with its task list (`plan.md`) and its
owner reviews (`review-N.md`). Each task ends with a commit. Tick the box (`[x]`) in the phase's
`plan.md` in the same commit that finishes the task. A phase is done when all its tasks are
ticked **and** its owner review is noted as passed in the phase's review file.

| Phase | Tool | Tasks | Owner review |
|---|---|---|---|
| 1 — PC script (done; now the analysis tools in `tools/analysis`) | desktop | [p01-pc-script](../desktop/phases/p01-pc-script/plan.md) | none |
| 2 — PC app (Rust + Tauri) | desktop | [p02-pc-app](../desktop/phases/p02-pc-app/plan.md) | [round 1](../desktop/phases/p02-pc-app/review-1.md), [round 2](../desktop/phases/p02-pc-app/review-2.md) |
| 3 — Android camera spike | mobile | [p03-camera-spike](../mobile/phases/p03-camera-spike/plan.md) | [review-1](../mobile/phases/p03-camera-spike/review-1.md) |
| 4 — App core (no camera) | mobile | [p04-app-core](../mobile/phases/p04-app-core/plan.md) | [review-1](../mobile/phases/p04-app-core/review-1.md) |
| 5 — Scanning | mobile | [p05-scanning](../mobile/phases/p05-scanning/plan.md) | [review-1](../mobile/phases/p05-scanning/review-1.md) |
| 6 — Files and settings | mobile | [p06-files-settings](../mobile/phases/p06-files-settings/plan.md) | [review-1](../mobile/phases/p06-files-settings/review-1.md) |
| 7 — Layouts and polish | mobile | [p07-layouts-polish](../mobile/phases/p07-layouts-polish/plan.md) | [review-1](../mobile/phases/p07-layouts-polish/review-1.md) |

## Status

Status: **Phase 2 (PC app): tasks 2.1–2.15 and 2.17 done — the packing-list display defects and the
*Open folder* defect of the owner's review round 1 (2026-10-11, his own day's data) are fixed;
2.16 (the courier) is next, and 2.18 (the check of the saved PDFs, designed at the
owner's request on 2026-10-11) waits behind it.**
The owner decided that the PC gets a desktop app written in Rust (a
command line is not good for daily use), so phase 1 stops with its code done (270 tests); the script is kept,
frozen, as the analysis tools in `tools/analysis/` (not part of the product); its two open tasks
move into phase 2. Design:
[07-pc-app](../desktop/design/07-pc-app.md), [08-pc-app-ui](../desktop/design/08-pc-app-ui.md).
Open in [desktop/design/02-design-questions.md](../desktop/design/02-design-questions.md): D17 (whether 2.18's check runs on every batch), D18 (how much of a *masked* courier logo to read — measured 2026-10-13: its appearance names J&T Cargo
where its encoded bytes cannot, so the question is how much of that clue to use), D20 (how the app writes a taught logo into `couriers.toml`), **D21** (an instant-service label names no courier — the name comes from the order export; the owner reported **Gojek** on 2026-10-13) and **D22** (where the pickup code shows during packing); in
[mobile/design/02-design-questions.md](../mobile/design/02-design-questions.md): D1 (Android, open
until phase 3).
**D15 is answered**
(2026-10-11, the owner: two clues that must agree — the tracking-ID form and the service tag; the
rule is written into 03 › *Courier deduction*, so D15 is removed), **D16 is answered** (2026-10-11): the courier data lives in the data folder, so a courier change is a file
edit the owner can make, not a new program version — task 2.19, and **D19 is answered**
(2026-10-11, the owner: one file — the taught logos go into `couriers.toml`, no second file). D2 (the
PDF library) is answered: PDFium does all three PDF
jobs, and the bigger packing-list files (about 1.4 MB, 2.0 MB with `full`/`summary`) are accepted.

**Next step.** 2.16 (the courier: the form-and-tag rule, no owner answer needed), then
2.19 (the courier data into the data folder, D16's answer — no owner answer needed, so it can be
built at any time) and 2.18 (the saved-PDF check; its four choices are answered and D17 has a
"built for now" default, so it waits on nothing); 2.20 (one hash for everything — the owner's decision of 2026-10-13,
no answer needed) can be built at any time. The 2026-10-13 sample download (3 files, 604 pages) was
measured against the courier rule: J&T Express 500 · SiCepat 96 · iD Express (IDX) 3 · **J&T Cargo 1**
(the first J&T Cargo and iD Express labels in a *slip* export) · 4 slip continuations — so it holds
no new courier, form or tag, and J&T Cargo's masked logo is readable by
appearance (one value for both its rasters): 2.16 gives it a logo clue like the others. **The new
courier the owner reported (2026-10-13) is Gojek**, and it is *not* in that download: Gojek serves
the platform's *Instant / Same Day* service, whose label is the `TikTok Shop`-branded page measured
earlier (`2026-10-07_07-16-53_labels+slips_2of3_TIKTOK-INSTANT.pdf` p162, the only such page in the samples) — the one style with **no courier logo and no tracking ID**,
printing `Kode Pengambilan` + a `WX` pickup code + the tag `GKS`, with a pickup *time* instead of
`23:59`. That download was checked page by page (images, captions, the pickup-code line, the
`In transit by` value, the 600 Order IDs): no 2026-10-13 page is that style, and per Tokopedia's
guide a Same Day label cannot be printed before the delivery is arranged — so the day's Gojek order
has no printable label yet. The page never names Gojek (the service is Gojek, Grab or Blitz), so
**D21 is new** (where an instant label's courier name comes from; its "Built for now" default is the
export's `Shipping Provider Name`), with **D22** (where the pickup code shows during packing).
**D18 is the other thing waiting on the owner** (how much of that appearance clue to use); both have
a "Built for now" default, so nothing waits.
Then the round-2 review at the end of Phase 2 — open the same day's packing
list (the header must stand clear, `×` must print), press *Open folder* (the batch folder must open,
not Documents), check the Day screen for a *Courier unknown* warning, and save one batch to watch
the check run (the progress line; a clean day must report nothing). The
answers belong in
[02-design-questions.md](../desktop/design/02-design-questions.md) — each question has a
"Built for now" default, nothing waits on them — or under *Answers* in
[review-2.md](../desktop/phases/p02-pc-app/review-2.md), or as a new session here; the phase-3 Android
work waits for this phase's review to pass.
