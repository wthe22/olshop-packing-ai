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

## R-APP-12 — One folder per date, no input and output folders

**Present.** A date folder holds its journal, an `input/` folder (the inputs) and an `output/` folder (the group
PDFs and the summary PDF). The day summary sits directly in the date folder.

**Wanted.** No `input/` and no `output/` folder: every file of a date sits directly in the date folder, the
journal included. Where the files being written first go, and how a file is kept safe while it is written, is
for the next dev to settle.

**For the next dev.** The next dev also builds a Python script that converts an existing data folder from the
`input/` and `output/` layout to the flat one (the owner's existing dates must keep working after the change; the
script is run once by the owner, like `tools/rename-inputs.py`). The flat layout ships as **v0.8 alpha**.

## R-APP-13 — Inputs named like the outputs

**Present.** An input is named `{date}_{NN}_{hash}.pdf`, for example `2026-11-23_03_61459e8a8203.pdf`.

**Wanted.** An input is named `{batch_no} Input {pdf_id} {short_hash}.pdf`, for example
`2 Input 03 0a1b2c3d4e5f.pdf` (the hash made up here): the batch's number, the word *Input* in capital case, the
PDF's number as today, and the short hash of its SHA-256 as today.

## R-APP-14 — The batch summary named in capital case

**Present.** The batch's summary PDF is named `{batch} summary.pdf`, lower case.

**Wanted.** `{batch_no} Summary.pdf`, or `{batch_no} Summary - {shop_name}.pdf` for one account's summary, with
*Summary* in capital case. `{shop_name}` is the account's name.

## R-APP-15 — The day summary named in capital case

**Present.** The day summary is named `day summary.pdf`, or `day summary {account}.pdf` for one account, lower case.

**Wanted.** `Day Summary.pdf`, or `Day Summary - {shop_name}.pdf` for one account, with *Day Summary* in capital
case. `{shop_name}` is the account's name.
