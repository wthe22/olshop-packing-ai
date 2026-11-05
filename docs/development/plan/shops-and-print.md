# Print in the app

The owner's words in chat (2026-11-05), written into the design as
[D-APP-26](../../software-design/decisions/app-features.md); the shop check (D-APP-25) is built.
[code-rules.md](../code-rules.md) is binding for every step and every subagent; each report states *size
gate passed, no new lint suppression, coverage not lower*. Tests: known-answer fixtures through
read → plan → save → verify as the backbone, tables for fixed truths, broken outputs the verifier must catch.

## P1. Print, Sent and Printed

Read first: [Day](../../software-design/ui/day.md),
[05 After the save › 16](../../software-design/workflow/05-after-save.md#16-print-sent-and-printed),
[journal](../../software-design/data/journal.md), [records](../../software-design/data/records.md),
[views](../../software-design/data/views.md), [program](../../software-design/app/program.md).

| Step | What | Done when |
|---|---|---|
| P1.1 | Spike, kept out of `main`: a second `WebviewWindow` showing an output through the asset protocol (WebView2's PDF viewer), and `ICoreWebView2_16::ShowPrintUI(COREWEBVIEW2_PRINT_DIALOG_KIND_BROWSER)` through `with_webview` (`webview2-com`) once the PDF has loaded | The preview opens on one click with Chrome's options; it is known whether WebView2 reports Print versus Cancel (if it does, *Sent* follows the real print and the design is changed in the same commit) |
| P1.2 | The `print_sent` record: engine, the fold's rule, the views' `is_marked_as_sent`, `crates/batch` | Fold table test; save → mark sent → revert/amend clears it; reopen shows the same marks |
| P1.3 | `print_output` and `set_sent`; the Day screen's **Print** (group PDFs, summary PDFs, reprints), *Sent hh:mm* ✕ and the tick | App tests; checked on screenshots of the built program in both themes and at 125 % |

- An older copy of the program cannot read a journal that holds `print_sent`: the handoff tells the owner
  to update every copy on a shared data folder together. Check [D-APP-3](../../software-design/decisions/app.md)
  for what a new record type means for the journal's `schema`.
- After P1 the version becomes `0.4.0` (the owner's word), in its one place
  ([Settings › The version](../../software-design/ui/settings.md#the-version)).
