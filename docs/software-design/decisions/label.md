# Decisions: labels

## D-LABEL-1 — One courier file, in the data folder

**Decision.** The courier data (names, logo hashes, tracking-ID forms, text clues) is one file,
`couriers.toml`, in the data folder beside `categories.toml`. The app writes the bundled copy there
on first start; the file is then the owner's, and a broken file stops the batch with its line.

**Why.** A new courier or a corrected form is an edit the owner can make, not a new program
version. One file keeps the two clues of a courier together.

**Applies to.** [inputs.md](../app/inputs.md), [program.md](../app/program.md),
[start.md](../ui/start.md).

## D-LABEL-2 — A courier is named only when two clues agree

**Decision.** The courier of an order is named from two clues, and only when they agree: the
**logo** of the label (its image fingerprint) and the **tracking-ID form**. When they do not agree,
the courier stays empty and the batch warns.

**Why.** Some couriers print their name only as a logo image, and two couriers share a
tracking-ID length; neither clue alone is safe.

**Applies to.** [courier.md](../labels/courier.md), [inputs.md](../app/inputs.md).

## D-LABEL-4 — The app remembers the logos the owner names

**Decision.** A logo the owner names in the logo prompt, and an image answered *Not a courier*, is
remembered by the app in the data folder. The app never asks about the same logo again. Where it is kept
is [Q-LABEL-7](../questions/label.md).

**Why.** Asking again for a logo already answered is work the owner should do once.

**Applies to.** [courier.md](../labels/courier.md), [inputs.md](../app/inputs.md),
[new-batch.md](../ui/new-batch.md).
