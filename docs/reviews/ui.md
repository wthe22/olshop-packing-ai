# UI: the owner's feedback

The owner's findings on the window and the screens. Open items only; see [README](README.md) for the form.

## R-UI-4 — Sessions: every day and session in a side panel

**Present.** The window shows one day; ◀ ▶ **Today** in the title bar step through days. A day is one
folder, so a second shop or a test run on the same day shares the batches, the numbers and the
already-saved guard of the first.

**Wanted.** The days listed as **sessions** in a side panel that can be shown or hidden, all of them
at once. More than one session in a day — one per shop, or to test a different setup.

**Design written, waits for the owner.** [Sessions](../software-design/app/sessions.md) and the
[Sessions panel](../software-design/ui/sessions.md); choices with defaults in
[Q-APP-4, Q-APP-5, Q-UI-5 to Q-UI-7](../software-design/questions/sessions.md).

## R-UI-5 — Repeated orders: which PDFs hold them, how many, in groups

**Present.** Nothing on screen says that a batch holds the same order twice.

**Wanted.** The PDFs that contain a duplicate highlighted; the number of duplicates visible; the
duplicate PDFs grouped, so that several duplicates can be identified and solved one group at a time.

**Reading.** *PDFs* is read as the label files of the batch being prepared
([Q-UI-8](../software-design/questions/duplicates.md) asks whether saved PDFs are meant too).

**Design written, waits for the owner.** [Repeated orders](../software-design/ui/duplicates.md);
choices in [Q-APP-6, Q-APP-7, Q-UI-8](../software-design/questions/duplicates.md).
